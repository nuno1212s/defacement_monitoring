extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use std::fmt::format;
use std::os::linux::raw::stat;
use std::ptr::write;
use std::slice::SliceIndex;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params};
use crate::databases::*;


const TRACKED_PAGES_TABLE: &str = "TRACKED_PAGES";
const TRACKED_PAGES_DOMS: &str = "PAGES";
const USERS: &str = "USERS";
const USER_CONTACTS: &str = "CONTACTS";
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

        connection.execute(format!("CREATE TABLE IF NOT EXISTS {} (rowid INTEGER PRIMARY KEY, PAGE_URL varchar(2048) NOT NULL, USER_ID INTEGER)",
                                   TRACKED_PAGES_TABLE).as_str(), []).unwrap();

        connection.execute(format!("CREATE UNIQUE INDEX IF NOT EXISTS PAGE_URL_IND ON {}(PAGE_URL)",
                                   TRACKED_PAGES_TABLE).as_str(), params![]).unwrap();

        connection.execute(format!("CREATE TABLE IF NOT EXISTS {} (rowid INTEGER PRIMARY KEY, PAGE_ID INTEGER, DOM TEXT NOT NULL)",
                                   TRACKED_PAGES_DOMS).as_str(), []).unwrap();

        connection.execute(format!("CREATE INDEX IF NOT EXISTS PAGE_ID_IND ON {}(PAGE_ID)",
                                   TRACKED_PAGES_DOMS).as_str(), params![]).unwrap();

        connection.execute(format!("CREATE TABLE IF NOT EXISTS {} (rowid INTEGER PRIMARY KEY, USERNAME varchar(50) NOT NULL)",
                                   USERS).as_str(), params![]).unwrap();

        connection.execute(format!("CREATE UNIQUE INDEX IF NOT EXISTS USERNAME_ID ON {}(USERNAME)",
                                   USERS).as_str(), params![]).unwrap();

        connection.execute(format!("CREATE TABLE IF NOT EXISTS {} (rowid INTEGER PRIMARY KEY, USER_ID INTEGER NOT NULL, CONTACT TEXT NOT NULL)",
                                   USER_CONTACTS).as_str(), params![]).unwrap();

        connection.execute(format!("CREATE UNIQUE INDEX IF NOT EXISTS USER_IND ON {}(USER_ID)",
                                   USER_CONTACTS).as_str(), params![]).unwrap();
    }

    fn read_doms_for_page_id(&self, page_id: u32) -> Result<Vec<StoredDom>, String> {
        let read_guard = self.get_sql_conn();

        let mut statement = read_guard.prepare(
            format!("SELECT * FROM {} WHERE PAGE_ID=?", TRACKED_PAGES_DOMS).as_str()).unwrap();

        return {
            match statement.query(params![page_id]) {
                Ok(mut state) => {
                    let mut doms = Vec::new();

                    while let Some(row) = state.next().unwrap() {
                        doms.push(StoredDom::new(row.get(0).unwrap(),
                                                 row.get(1).unwrap(),
                                                 row.get(2).unwrap()));
                    }

                    Ok(doms)
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
    fn insert_tracked_page(&self, page: &str, user_id: u32) -> Result<TrackedPage, String> {
        let write_guard = self.write_sql_conn();

        let mut statement = write_guard.prepare(
            format!("INSERT INTO {}(PAGE_URL, USER_ID) values(?, ?)", TRACKED_PAGES_TABLE).as_str()).unwrap();

        match statement.execute(params![page, user_id]) {
            Ok(state) => {
                if state > 0 {
                    let i = write_guard.last_insert_rowid();

                    Ok(TrackedPage::new(i as u32, String::from(page), user_id))
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

    fn list_all_tracked_pages(&self) -> Result<Vec<TrackedPage>, String> {
        let read_guard = self.get_sql_conn();

        let mut result = read_guard.prepare(
            format!("SELECT rowid, PAGE_URL, USER_ID FROM {}", TRACKED_PAGES_TABLE).as_str()
        ).unwrap();

        let mut return_vec = Vec::new();

        let mut rows = result.query([]).unwrap();

        while let Some(row) = rows.next().unwrap() {
            return_vec.push(TrackedPage::new(row.get(1).unwrap(), row.get(2).unwrap(),
                                             row.get(3).unwrap()));
        }

        Ok(return_vec)
    }

    fn get_information_for_page(&self, page: &str) -> Result<TrackedPage, String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.prepare(format!("SELECT * FROM {} WHERE LOWER(PAGE_URL)=LOWER(?)", TRACKED_PAGES_TABLE).as_str()).unwrap();

        return match statement.query(params![page]) {
            Ok(mut rows) => {
                if let Some(row) = rows.next().unwrap() {
                    return Ok(TrackedPage::new(row.get(0).unwrap(),
                                               row.get(1).unwrap(), row.get(2).unwrap()));
                }

                Err(format!("Could not find the required page with url {}", page))
            }
            Err(e) => { Err(e.to_string()) }
        };
    }

    fn get_information_for_tracked_page(&self, page_id: u32) -> Result<TrackedPage, String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.prepare(format!("SELECT * FROM {} WHERE rowid=?", TRACKED_PAGES_TABLE).as_str()).unwrap();

        return match statement.query(params![page_id]) {
            Ok(mut rows) => {
                if let Some(row) = rows.next().unwrap() {
                    return Ok(TrackedPage::new(row.get(0).unwrap(),
                                               row.get(1).unwrap(), row.get(2).unwrap()));
                }

                Err(format!("Could not find the required page with id {}", page_id))
            }
            Err(e) => { Err(e.to_string()) }
        };
    }

    fn del_tracked_page(&self, page: TrackedPage) -> Result<bool, String> {
        let write_guard = self.get_sql_conn();

        let mut statement = write_guard.prepare(
            format!("DELETE FROM {} WHERE rowid=?", TRACKED_PAGES_TABLE).as_str()).unwrap();

        match statement.execute(params![page.page_id()]) {
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


    fn read_doms_for_page(&self, page: &TrackedPage) -> Result<Vec<StoredDom>, String> {
        self.read_doms_for_page_id(page.page_id())
    }

    fn insert_dom_for_page(&self, page: &TrackedPage, page_dom: &str) -> Result<StoredDom, String> {
        let write_guard = self.write_sql_conn();

        let mut update = write_guard
            .prepare(format!("INSERT INTO {}(PAGE_ID, DOM) values(?, ?)", TRACKED_PAGES_DOMS).as_str()).unwrap();

        return match update.execute(params![page.page_id(), page_dom]) {
            Ok(count) => {
                if count > 0 {
                    Ok(StoredDom::new(write_guard.last_insert_rowid() as u32,
                                      page.page_id(), String::from(page_dom)))
                } else {
                    Err(String::from("Failed to insert into the DB"))
                }
            }
            Err(e) => {
                Err(e.to_string())
            }
        };
    }

    fn update_dom_for_page(&self, page: &TrackedPage, dom: &mut StoredDom, page_dom: &str) -> Result<(), String> {
        let guard = self.get_sql_conn();

        let mut update = guard
            .prepare(format!("UPDATE {} SET DOM=? WHERE rowid=?", TRACKED_PAGES_DOMS).as_str())
            .unwrap();

        match update.execute(params![page_dom, dom.dom_id()]) {
            Ok(_) => {
                dom.set_dom(String::from(page_dom));

                Ok(())
            }
            Err(e) => {
                Err(e.to_string())
            }
        }
    }

    fn delete_dom_for_page(&self, _page: &TrackedPage, dom: StoredDom) -> Result<bool, String> {
        let read_guard = self.get_sql_conn();

        let mut execute = read_guard.prepare(
            format!("DELETE FROM {} WHERE rowid=?", TRACKED_PAGES_DOMS).as_str()).unwrap();

        match execute.execute(params![dom.dom_id()]) {
            Ok(size) => {
                if size > 0 { Ok(true) } else { Ok(false) }
            }
            Err(e) => { Err(e.to_string()) }
        }
    }
}

impl UserDB for SQLLiteDefacementDB {
    fn create_user(&self, user_name: &str) -> Result<User, String> {
        let connection = self.get_sql_conn();
        let statement = connection.execute(format!("INSERT INTO {}(USERNAME) values(LOWER(?))", USERS).as_str(),
                                           params![user_name]);

        return match statement {
            Ok(changed_count) => {
                if changed_count > 0 {
                    let last_id = connection.last_insert_rowid();

                    Ok(User::new(last_id as u32, String::from(user_name)))
                } else {
                    Err(String::from("Failed to insert user, maybe it's duplicated?"))
                }
            }
            Err(e) => {
                Err(e.to_string())
            }
        };
    }

    fn get_user_info_for(&self, user_name: &str) -> Result<User, String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.prepare(format!("SELECT * FROM {} WHERE LOWER(USERNAME)=LOWER(?)", USERS)
            .as_str()).unwrap();

        return match statement.query(params![user_name]) {
            Ok(mut rows) => {
                if let Some(row) = rows.next().unwrap() {
                    Ok(User::new(row.get(0).unwrap(), row.get(1).unwrap()))
                }

                Err(String::from("Could not find user."))
            }

            Err(e) => {
                Err(e.to_string())
            }
        };
    }

    fn delete_user(&self, user_id: u32) -> Result<bool, String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.prepare(format!("DELETE FROM {} WHERE rowid=?", USERS).as_str()).unwrap();

        return match statement.execute(params![user_id]) {
            Ok(count) => {
                if count > 0 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => { Err(e.to_string()) }
        };
    }

    fn register_contact(&self, user: &User, comm: CommData) -> Result<UserCommunication, String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.execute(format!("INSERT INTO {} (USER_ID, CONTACT) values(?, ?)", USER_CONTACTS).as_str(),
                                               params![user.user_id(), ]);

        match statement {
            Ok(changed_rows) => {
                if changed_rows > 0 {
                    let id = connection.last_insert_rowid();

                    Ok(UserCommunication::new(id as u32, user.user_id(), comm))
                } else {
                    Err(String::from("Failed to add contact"))
                }
            }
            Err(e) => {
                Err(e.to_string())
            }
        }
    }

    fn list_contacts_for(&self, user: &User) -> Result<Vec<UserCommunication>, String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.prepare(format!("SELECT * FROM {} WHERE USER_ID=?", USER_CONTACTS).as_str()).unwrap();

        let mut contacts = Vec::new();

        match statement.query(params![user.user_id()]) {
            Ok(mut rows) => {
                while let Some(row) = rows.next().unwrap() {
                    contacts.push(UserCommunication::new(row.get(0).unwrap(),
                                                         row.get(1).unwrap(), row.get(2).unwrap()));
                }
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }

        Ok(contacts)
    }

    fn delete_contact(&self, comm: UserCommunication) -> Result<bool, String> {
        let connection = self.get_sql_conn();

        return match connection.execute(format!("DELETE FROM {} WHERE rowid=?", USER_CONTACTS).as_str(),
                                        params![comm.comm_id()]) {
            Ok(count) => {
                if count > 0 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => { Err(e.to_string()) }
        };
    }
}

#[cfg(test)]
mod sqlite_tests {
    use crate::databases::WebsiteDefacementDB;
    use crate::databases::sqlitedb::SQLLiteDefacementDB;

    #[test]
    fn test_sqlite_tracked_page() {
        let db = SQLLiteDefacementDB::new();

        let page = "https://google.com";

        let result = db.insert_tracked_page(page, 0);

        assert!(result.is_ok());

        let result2 = db.insert_tracked_page(page, 0);

        assert!(result2.is_err());

        let page_id = result.unwrap();

        assert_eq!(db.get_information_for_page(page).unwrap(), page_id);

        let x = db.del_tracked_page(page_id).unwrap();

        assert_eq!(x, true);

        assert!(db.read_page_id_for_page(page).is_err())
    }

    #[test]
    fn test_sqlite_store_dom() {
        let db = SQLLiteDefacementDB::new();

        let page = "https://google.com";

        let dom = "<>";

        let result = db.insert_tracked_page(page, 0);

        assert!(result.is_ok());

        let tracked_page = result.unwrap();

        let result_insert_dom = db.insert_dom_for_page(&tracked_page, dom);

        assert!(result_insert_dom.is_ok());

        let inserted_dom = result_insert_dom.unwrap();

        let doms = db.read_doms_for_page(&tracked_page);

        assert!(doms.is_ok());

        assert_eq!(doms.unwrap(), vec![inserted_dom.clone()]);

        let delete_result = db.delete_dom_for_page(&tracked_page, inserted_dom);

        assert!(delete_result.is_ok());

        let doms_2 = db.read_doms_for_page(&tracked_page);

        assert!(doms_2.is_ok() && doms_2.unwrap().is_empty());

        assert!(db.del_tracked_page(tracked_page).is_ok());
    }
}