extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

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

        Self {
            sql_conn: pool
        }
    }

    fn get_sql_conn(&self) -> PooledConnection<SqliteConnectionManager> {
        self.sql_conn.get().unwrap()
    }

    fn write_sql_conn(&self) -> PooledConnection<SqliteConnectionManager> {
        self.get_sql_conn()
    }

    fn read_dom_for_page_id(&self, page_id: u32) -> Result<String, String> {
        let read_guard = self.get_sql_conn();

        let mut statement = read_guard.prepare(
            format!("SELECT * FROM {} WHERE PAGE_ID=?", TRACKED_PAGES_DOMS).as_str()).unwrap();

        return {
            match statement.query(params![page_id]) {
                Ok(mut state) => {
                    match state.next() {
                        Ok(pos_row) => {
                            match pos_row {
                                Some(row) => {
                                    Ok(row.get(0).unwrap())
                                }
                                None => {
                                    Err(String::from("Could not find dom for page"))
                                }
                            }
                        }
                        Err(e) => { Err(e.to_string()) }
                    }
                }
                Err(E) => {
                    Err(E.to_string())
                }
            }
        };
    }

    fn read_page_id_for_page(&self, page: &str) -> Result<u32, String> {
        let read_guard = self.get_sql_conn();

        let mut statement
            = read_guard.prepare(format!("SELECT ROWID FROM {} WHERE PAGE=?", TRACKED_PAGES_TABLE).as_str()).unwrap();

        return match statement.query(params![page]) {
            Ok(mut rows) => {
                match rows.next() {
                    Ok(row) => {
                        match row {
                            Some(row_) => {
                                Ok(row_.get(0).unwrap())
                            }
                            None => Err(String::from("Failed to find page"))
                        }
                    }
                    Err(E) => {
                        Err(E.to_string())
                    }
                }
            }
            Err(E) => {
                Err(E.to_string())
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
                    let mut read_id_stmt = write_guard.prepare("last_insert_id()").unwrap();

                    return match read_id_stmt.query([]) {
                        Ok(mut rows) => {
                            match rows.next() {
                                Ok(row) => {
                                    match row {
                                        Some(row_obj) => {
                                            Ok(row_obj.get(0).unwrap())
                                        }
                                        None => {
                                            Err(String::from("Failed to retrieve error"))
                                        }
                                    }
                                }
                                Err(E) => { Err(E.to_string()) }
                            }
                        }
                        Err(E) => {
                            Err(E.to_string())
                        }
                    };
                } else {
                    Err(String::from("Failed to insert"))
                }
            }
            Err(E) => {
                // println!("Failed to insert into DB")
                Err(E.to_string())
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
            Err(E) => {
                Err(E.to_string())
            }
        }
    }


    fn read_dom_for_page(&self, page: &str) -> Result<String, String> {
        let read_guard = self.get_sql_conn();

        let mut statement = read_guard
            .prepare(format!("SELECT ROWID FROM {} WHERE page=?", TRACKED_PAGES_TABLE).as_str()).unwrap();

        return match statement.query(params![page]) {
            Ok(mut e) => {
                let result = e.next().unwrap().unwrap();

                let id: u32 = result.get(0).unwrap();

                self.read_dom_for_page_id(id)
            }
            Err(E) => {
                Err(E.to_string())
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
                            let mut statement = write_guard.prepare("last_inserted_id()").unwrap();

                            return match statement.query([]) {
                                Ok(mut rows) => {
                                    match rows.next() {
                                        Ok(row) => {
                                            match row {
                                                Some(row_obj) => Ok(row_obj.get(0).unwrap()),
                                                None => Err(String::from("Failed to insert"))
                                            }
                                        }
                                        Err(e) => { Err(e.to_string()) }
                                    }
                                }
                                Err(e) => { Err(e.to_string()) }
                            };
                        } else {
                            Err(String::from("Failed to insert into the DB"))
                        }
                    }
                    Err(e) => {
                        Err(e.to_string())
                    }
                };
            }
            Err(E) => {
                Err(E)
            }
        }
    }

    fn update_dom_for_page(&self, page: &str, dom_id: u32, page_dom: &str) -> Result<(), String> {
        let guard = self.get_sql_conn();

        let mut update = guard
            .prepare(format!("UPDATE {} SET DOM=? WHERE ROWID=?", TRACKED_PAGES_DOMS).as_str())
            .unwrap();

        match update.execute(params![page_dom, dom_id]) {
            Ok(_) => {
                Ok(())
            }
            Err(E) => {
                Err(E.to_string())
            }
        }
    }

    fn delete_dom_for_page(&self, page: &str, dom_id: u32) -> Result<bool, String> {
        let read_guard = self.get_sql_conn();

        let mut execute = read_guard.prepare(
            format!("DELETE FROM {} WHERE ROWID=?", TRACKED_PAGES_DOMS).as_str()).unwrap();

        match execute.execute(params![dom_id]) {
            Ok(size) => {
                if size > 0 { Ok(true) } else { Ok(false) }
            }
            Err(E) => { Err(E.to_string()) }
        }
    }
}

#[cfg(test)]
mod sqlite_tests {
    #[test]
    fn test_sqlite() {}
}