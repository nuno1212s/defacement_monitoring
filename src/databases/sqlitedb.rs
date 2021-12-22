extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use std::fmt::format;
use std::ptr::write;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params};

use crate::databases::database::WebsiteDefacementDB;

const TRACKED_PAGES_TABLE: &str = "TRACKED_PAGES";
const TRACKED_PAGES_DOMS: &str = "PAGES";
const IN_MEMORY: &str = ":memory:";
const PAGE_STORAGE: &str = "pages_db";

/*
By using a connection pool we are able to use multiple threads effectively
 */
pub struct SQLLiteDefacementDB {
    sql_conn: Pool<SqliteConnectionManager>,
}

impl SQLLiteDefacementDB {
    fn new() -> Self {
        let manager = SqliteConnectionManager::file(PAGE_STORAGE);

        let pool = r2d2::Pool::new(manager).unwrap();

        let result = Self {
            sql_conn: pool
        };

        result.create_tables();

        result
    }

    fn get_sql_conn(&self) -> PooledConnection<SqliteConnectionManager> {
        self.sql_conn.get().unwrap()
    }

    fn write_sql_conn(&self) -> PooledConnection<SqliteConnectionManager> {
        self.get_sql_conn()
    }

    fn create_tables(&self) {
        let connection = self.get_sql_conn();

        connection.execute(format!("CREATE TABLE IF NOT EXISTS {} (rowid INTEGER PRIMARY KEY, PAGE_URL varchar(2048) NOT NULL)",
                                   TRACKED_PAGES_TABLE).as_str(), []).unwrap();

        connection.execute(format!("CREATE UNIQUE INDEX IF NOT EXISTS PAGE_URL_IND ON {}(PAGE_URL)",
                                   TRACKED_PAGES_TABLE).as_str(), params![]).unwrap();

        connection.execute(format!("CREATE TABLE IF NOT EXISTS {} (rowid INTEGER PRIMARY KEY, PAGE_ID INTEGER, DOM TEXT NOT NULL)",
                                   TRACKED_PAGES_DOMS).as_str(), []).unwrap();

        connection.execute(format!("CREATE INDEX IF NOT EXISTS PAGE_ID_IND ON {}(PAGE_ID)",
                                   TRACKED_PAGES_DOMS).as_str(), params![]).unwrap();
    }

    fn read_doms_for_page_id(&self, page_id: u32) -> Result<Vec<String>, String> {
        let read_guard = self.get_sql_conn();

        let mut statement = read_guard.prepare(
            format!("SELECT DOM FROM {} WHERE PAGE_ID=?", TRACKED_PAGES_DOMS).as_str()).unwrap();

        return {
            match statement.query(params![page_id]) {
                Ok(mut state) => {

                    let mut doms = Vec::new();

                    loop {
                        match state.next() {
                            Ok(row) => {

                                match row {
                                    Some(row_) => {
                                        doms.push(row_.get(0).unwrap());
                                    }
                                    None => {
                                        return Ok(doms);
                                    }
                                }

                            }
                            Err(e) => {
                                return Err(e.to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    Err(e.to_string())
                }
            }
        };
    }

    fn read_page_id_for_page(&self, page: &str) -> Result<u32, String> {
        let read_guard = self.get_sql_conn();

        let mut statement
            = read_guard.prepare(format!("SELECT ROWID FROM {} WHERE PAGE_URL=?", TRACKED_PAGES_TABLE).as_str()).unwrap();

        return match statement.query(params![page]) {
            Ok(mut rows) => {
                match rows.next() {
                    Ok(row) => {
                        match row {
                            Some(row_) => {
                                let x: i64 = row_.get(0).unwrap();

                                Ok(x as u32)
                            }
                            None => Err(String::from("Failed to find page"))
                        }
                    }
                    Err(e) => {
                        Err(e.to_string())
                    }
                }
            }
            Err(e) => {
                Err(e.to_string())
            }
        };
    }
}

impl WebsiteDefacementDB for SQLLiteDefacementDB {
    fn insert_tracked_page(&self, page: &str) -> Result<u32, String> {
        let write_guard = self.write_sql_conn();

        let mut statement = write_guard.prepare(
            format!("INSERT INTO {}(PAGE_URL) values(?)", TRACKED_PAGES_TABLE).as_str()).unwrap();

        match statement.execute(params![page]) {
            Ok(state) => {
                if state > 0 {
                    let i = write_guard.last_insert_rowid();

                    Ok(i as u32)
                } else {
                    Err(String::from("Failed to insert"))
                }
            }
            Err(e) => {
                // println!("Failed to insert into DB")
                Err(e.to_string())
            }
        }
    }

    fn list_all_tracked_pages(&self) -> Result<Vec<String>, String> {
        let read_guard = self.get_sql_conn();

        let mut result = read_guard.prepare(
            format!("SELECT * FROM {}", TRACKED_PAGES_TABLE).as_str()
        ).unwrap();

        let mut return_vec = Vec::new();

        let mut rows = result.query([]).unwrap();

        while let Some(row) = rows.next().unwrap() {
            return_vec.push(row.get(1).unwrap());
        }

        Ok(return_vec)
    }

    fn del_tracked_page(&self, page: &str) -> Result<bool, String> {
        let write_guard = self.get_sql_conn();

        let mut statement = write_guard.prepare(
            format!("DELETE FROM {} WHERE LOWER(PAGE_URL)=LOWER(?)", TRACKED_PAGES_TABLE).as_str()).unwrap();

        match statement.execute(params![page]) {
            Ok(state) => {
                if state > 0 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => {
                Err(e.to_string())
            }
        }
    }


    fn read_doms_for_page(&self, page: &str) -> Result<Vec<String>, String> {
        let read_guard = self.get_sql_conn();

        let mut statement = read_guard
            .prepare(format!("SELECT rowid FROM {} WHERE PAGE_URL=?", TRACKED_PAGES_TABLE).as_str()).unwrap();

        return match statement.query(params![page]) {
            Ok(mut e) => {
                let result = e.next().unwrap().unwrap();

                let id: u32 = result.get(0).unwrap();

                self.read_doms_for_page_id(id)
            }
            Err(e) => {
                Err(e.to_string())
            }
        };
    }

    fn insert_dom_for_page(&self, page: &str, page_dom: &str) -> Result<u32, String> {
        match self.read_page_id_for_page(page) {
            Ok(page_id) => {
                let write_guard = self.write_sql_conn();

                let mut update = write_guard
                    .prepare(format!("INSERT INTO {}(PAGE_ID, DOM) values(?, ?)", TRACKED_PAGES_DOMS).as_str()).unwrap();


                return match update.execute(params![page_id, page_dom]) {
                    Ok(count) => {
                        if count > 0 {
                            Ok(write_guard.last_insert_rowid() as u32)
                        } else {
                            Err(String::from("Failed to insert into the DB"))
                        }
                    }
                    Err(e) => {
                        Err(e.to_string())
                    }
                };
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    fn update_dom_for_page(&self, _page: &str, dom_id: u32, page_dom: &str) -> Result<(), String> {
        let guard = self.get_sql_conn();

        let mut update = guard
            .prepare(format!("UPDATE {} SET DOM=? WHERE rowid=?", TRACKED_PAGES_DOMS).as_str())
            .unwrap();

        match update.execute(params![page_dom, dom_id]) {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                Err(e.to_string())
            }
        }
    }

    fn delete_dom_for_page(&self, _page: &str, dom_id: u32) -> Result<bool, String> {
        let read_guard = self.get_sql_conn();

        let mut execute = read_guard.prepare(
            format!("DELETE FROM {} WHERE rowid=?", TRACKED_PAGES_DOMS).as_str()).unwrap();

        match execute.execute(params![dom_id]) {
            Ok(size) => {
                if size > 0 { Ok(true) } else { Ok(false) }
            }
            Err(e) => { Err(e.to_string()) }
        }
    }
}

#[cfg(test)]
mod sqlite_tests {
    use crate::databases::database::WebsiteDefacementDB;
    use crate::databases::sqlitedb::SQLLiteDefacementDB;

    #[test]
    fn test_sqlite_tracked_page() {
        let db = SQLLiteDefacementDB::new();

        let page = "https://google.com";

        let result = db.insert_tracked_page(page);

        assert!(result.is_ok());

        let result2 = db.insert_tracked_page(page);

        assert!(result2.is_err());

        let page_id = result.unwrap();

        assert_eq!(db.read_page_id_for_page(page).unwrap(), page_id);

        let x = db.del_tracked_page(page).unwrap();

        assert_eq!(x, true);

        assert!(db.read_page_id_for_page(page).is_err())
    }

    #[test]
    fn test_sqlite_store_dom() {
        let db = SQLLiteDefacementDB::new();

        let page = "https://google.com";

        let dom = "<>";

        let result = db.insert_tracked_page(page);

        assert!(result.is_ok());

        let result_insert_dom = db.insert_dom_for_page(page, dom);

        assert!(result_insert_dom.is_ok());

        let doms = db.read_doms_for_page(page);

        assert!(doms.is_ok());

        assert_eq!(doms.unwrap(), vec![dom]);

        let delete_result = db.delete_dom_for_page(page, result_insert_dom.unwrap());

        assert!(delete_result.is_ok());

        let doms_2 = db.read_doms_for_page(page);

        assert!(doms_2.is_ok() && doms_2.unwrap().is_empty());

        assert!(db.del_tracked_page(page).is_ok());
    }
}