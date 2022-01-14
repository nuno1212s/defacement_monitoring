extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use std::fmt::{Display, format};
use std::string::ToString;
use std::time::{SystemTime, UNIX_EPOCH};

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Error, params, Row, Rows, ToSql};
use rusqlite::types::FromSql;

use crate::communication::CommData::Email;
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
#[derive(Clone)]
pub struct SQLLiteDefacementDB<T> where T: Display + FromSql + ToSql {
    sql_conn: Pool<SqliteConnectionManager>,
    _phantom: Option<T>,
}

impl<T> SQLLiteDefacementDB<T> where T: Display + FromSql + ToSql {
    pub(crate) fn new() -> Self {
        let manager = SqliteConnectionManager::file(PAGE_STORAGE);

        let pool = r2d2::Pool::new(manager).unwrap();

        let result = Self {
            sql_conn: pool,
            _phantom: None,
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

        connection.execute(format!("CREATE TABLE IF NOT EXISTS {} (rowid INTEGER PRIMARY KEY, \
        PAGE_URL varchar(2048) NOT NULL, USER_ID INTEGER NOT NULL, LAST_TIME_CHECKED INTEGER,\
        LAST_TIME_INDEXED INTEGER, INDEX_INTERVAL INTEGER,DEFACEMENT_COUNT INTEGER DEFAULT 0, DEFACEMENT_THRESHOLD INTEGER NOT NULL, \
         NOTIFIED_OF_CURRENT INTEGER DEFAULT 0, PAGE_TYPE varchar(25) NOT NULL, PAGE_TRACKING_DATA TEXT)",
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

        connection.execute(format!("CREATE TABLE IF NOT EXISTS {} (rowid INTEGER PRIMARY KEY, USER_ID INTEGER NOT NULL, CONTACT_TYPE varchar(50) NOT NULL, CONTACT TEXT NOT NULL)",
                                   USER_CONTACTS).as_str(), params![]).unwrap();

        connection.execute(format!("CREATE INDEX IF NOT EXISTS USER_IND ON {}(USER_ID)",
                                   USER_CONTACTS).as_str(), params![]).unwrap();
    }

    fn read_doms_for_page_id(&self, page_id: u32) -> Result<Vec<StoredDom<T>>, String> {
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

    fn parse_tracked_page_from_row(&self, row: &Row) -> Result<TrackedPage, Error> {
        let page_id: u32 = row.get(0)?;
        let page_url: String = row.get(1)?;
        let owning_user_id: u32 = row.get(2)?;
        let last_time_checked: u64 = row.get(3)?;
        let last_time_indexed: u64 = row.get(4)?;
        let index_interval: u64 = row.get(5)?;
        let defacement_count: u32 = row.get(6)?;
        let defacement_threshold: u32 = row.get(7)?;
        let notified_current: u32 = row.get(8)?;

        let mut tracked_page_type = TrackedPageType::Static;

        let tracked_type: String = row.get(9)?;

        if tracked_type.eq_ignore_ascii_case("Dynamic") {
            let diff_str: String = row.get(10)?;

            let diff: f64 = diff_str.parse::<f64>().unwrap();

            tracked_page_type = TrackedPageType::Dynamic(diff);
        }

        Ok(TrackedPage::new(page_id, page_url, owning_user_id, last_time_checked as u128,
                            last_time_indexed as u128, index_interval as u128,
                            defacement_count, defacement_threshold, notified_current != 0,
                            tracked_page_type))
    }

    fn crawl_all_pages_in_result_set(&self, rows: &mut Rows) -> Result<Vec<TrackedPage>, Error> {
        let mut return_vec = Vec::new();

        while let Some(row) = rows.next().unwrap() {
            let parsed_page = self.parse_tracked_page_from_row(row)?;

            return_vec.push(parsed_page);
        }

        Ok(return_vec)
    }

    fn list_all_pages_not_actioned_for(&self, time_since_last_check: Option<u128>, check_interval_col: Option<&str>, check_col: &str) -> Result<Vec<TrackedPage>, String> {
        let read_guard = self.get_sql_conn();

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap().as_millis();

        let mut final_str;

        match time_since_last_check {
            None => {
                final_str = format!("UPDATE {} SET {}=? WHERE {}<(?-{})", TRACKED_PAGES_TABLE,
                                    check_col, check_col, check_interval_col.unwrap());
            }
            Some(time_interval) => {
                final_str = format!("UPDATE {} SET {}=? WHERE {}<(?-{})", TRACKED_PAGES_TABLE,
                                    check_col, check_col, time_interval);
            }
        }

        /// This is basically a poor man's lock on SQLite without actually locking anything
        /// By performing this update, we are basically saying that we will take responsibility of checking these pages
        /// Because the time set is correspondent to our time and the odds of another thread attempting to
        /// do this at the EXACT same time as us is basically 0, so we get a form of locking
        /// to assure each page only gets verified once every time_since_last_check to improve performance
        let mut update_query = read_guard.prepare(final_str.as_str()).unwrap();

        match update_query.execute(params![current_time as u64, current_time as u64]) {
            Ok(_) => {}
            Err(e) => {
                return Err(e.to_string());
            }
        };

        let mut result = read_guard.prepare(
            format!("SELECT * FROM {} WHERE {}=?", TRACKED_PAGES_TABLE, check_col).as_str()
        ).unwrap();

        let mut rows = result.query(params![current_time as u64]).unwrap();

        return match self.crawl_all_pages_in_result_set(&mut rows) {
            Ok(results) => { Ok(results) }
            Err(e) => { Err(e.to_string()) }
        };
    }

    fn parse_contact_from_row(&self, row: &Row) -> Result<UserCommunication, String> {
        let comm_type: String = row.get(2).unwrap();

        let mut comm: Option<CommData> = Option::None;

        if comm_type.eq("EMAIL") {
            comm = Some(Email(row.get(3).unwrap()))
        }

        return match comm {
            Some(comm_) => {
                Ok(UserCommunication::new(row.get(0).unwrap(),
                                          row.get(1).unwrap(),
                                          comm_))
            }
            None => { Err(format!("Failed to load comm from row.")) }
        };
    }
}

impl<T> WebsiteDefacementDB<T> for SQLLiteDefacementDB<T> where T: Display + Debug + FromSql + ToSql + Send + Sync {
    fn insert_tracked_page(&self, page: &str, user_id: u32) -> Result<TrackedPage, String> {
        let write_guard = self.write_sql_conn();

        let mut statement = write_guard.prepare(
            format!("INSERT INTO {}(PAGE_URL, USER_ID, LAST_TIME_CHECKED, LAST_TIME_INDEXED, INDEX_INTERVAL, DEFACEMENT_THRESHOLD, PAGE_TYPE) values(?, ?, ?, ?,?, ?, ?)", TRACKED_PAGES_TABLE).as_str()).unwrap();

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        match statement.execute(params![page, user_id, current_time as u64,
            0,DEFAULT_INDEXING_INTERVAL, DEFAULT_DEFACEMENT_THRESHOLD, "Static"]) {
            Ok(state) => {
                if state > 0 {
                    let i = write_guard.last_insert_rowid();

                    Ok(TrackedPage::new(i as u32, String::from(page), user_id,
                                        current_time, 0,
                                        DEFAULT_INDEXING_INTERVAL as u128, 0,
                                        DEFAULT_DEFACEMENT_THRESHOLD, false,
                                        TrackedPageType::Static))
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
            format!("SELECT * FROM {}", TRACKED_PAGES_TABLE).as_str()
        ).unwrap();

        let mut rows = result.query([]).unwrap();

        return match self.crawl_all_pages_in_result_set(&mut rows) {
            Ok(results) => { Ok(results) }
            Err(e) => { Err(e.to_string()) }
        };
    }

    fn list_all_pages_not_checked_for(&self, time_since_last_check: u128) -> Result<Vec<TrackedPage>, String> {
        self.list_all_pages_not_actioned_for(Option::Some(time_since_last_check), None, "LAST_TIME_CHECKED")
    }

    fn list_all_pages_not_indexed_for(&self) -> Result<Vec<TrackedPage>, String> {
        self.list_all_pages_not_actioned_for(None, Some("INDEX_INTERVAL"), "LAST_TIME_INDEXED")
    }

    fn get_information_for_page(&self, page: &str) -> Result<TrackedPage, String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.prepare(format!("SELECT * FROM {} WHERE LOWER(PAGE_URL)=LOWER(?)", TRACKED_PAGES_TABLE).as_str()).unwrap();

        return match statement.query(params![page]) {
            Ok(mut rows) => {
                if let Some(row) = rows.next().unwrap() {
                    let parse_result = self.parse_tracked_page_from_row(row);

                    return match parse_result {
                        Ok(page) => Ok(page),
                        Err(e) => Err(e.to_string())
                    };
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
                    let parse_result = self.parse_tracked_page_from_row(row);

                    return match parse_result {
                        Ok(page) => Ok(page),
                        Err(e) => Err(e.to_string())
                    };
                }

                Err(format!("Could not find the required page with id {}", page_id))
            }
            Err(e) => { Err(e.to_string()) }
        };
    }

    fn update_tracking_type_for_page(&self, page: &TrackedPage) -> Result<bool, String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.prepare(format!("UPDATE {} SET PAGE_TYPE=?,\
         PAGE_TRACKING_DATA=?,LAST_TIME_INDEXED=?,INDEX_INTERVAL=? WHERE rowid=?", TRACKED_PAGES_TABLE).as_str()).unwrap();

        let mut page_type_data = String::from("NULL");

        match page.tracked_page_type() {
            TrackedPageType::Static => {}
            TrackedPageType::Dynamic(diff_threshold) => {
                page_type_data = format!("{}", diff_threshold);
            }
        }

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        return match statement.execute(params![tracked_page_type_to_str(page.tracked_page_type()),
        page_type_data,current_time as u64, page.index_interval() as u64, page.page_id()]) {
            Ok(changed) => {
                if changed > 0 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(e) => {
                Err(e.to_string())
            }
        };
    }

    fn increment_defacement_count(&self, page: &mut TrackedPage, notified: bool) -> Result<(), String> {
        let connection = self.get_sql_conn();

        ///If he has already been notified then we don't want to set that to no
        let mut statement = connection.prepare(format!("UPDATE {} SET DEFACEMENT_COUNT=DEFACEMENT_COUNT+1,\
         NOTIFIED_OF_CURRENT=(? OR NOTIFIED_OF_CURRENT) WHERE rowid=?", TRACKED_PAGES_TABLE).as_str()).unwrap();

        return match statement.execute(params![notified, page.page_id()]) {
            Ok(size) => {
                if size > 0 {
                    let mut fetch_count = connection.prepare(format!("SELECT DEFACEMENT_COUNT FROM {} WHERE rowid=?", TRACKED_PAGES_TABLE).as_str()).unwrap();

                    return match fetch_count.query(params![page.page_id()]) {
                        Ok(mut rows) => {
                            if let Some(row) = rows.next().unwrap() {
                                page.set_defacement_count(row.get(0).unwrap());

                                if notified {
                                    page.set_notified_of_current_breach(true);
                                }

                                Ok(())
                            } else {
                                Err(String::from("Something went very wrong"))
                            }
                        }
                        Err(err) => { Err(err.to_string()) }
                    };
                }
                Ok(())
            }
            Err(err) => { Err(err.to_string()) }
        };
    }

    fn reset_defacement_count(&self, page: &mut TrackedPage) -> Result<(), String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.prepare(format!("UPDATE {} SET DEFACEMENT_COUNT=0,NOTIFIED_OF_CURRENT=0 WHERE rowid=?", TRACKED_PAGES_TABLE).as_str()).unwrap();

        return match statement.execute(params![page.page_id()]) {
            Ok(edited) => {
                if edited > 0 {
                    page.set_defacement_count(0);
                    page.set_notified_of_current_breach(false);
                    Ok(())
                } else {
                    Err(String::from("Could not edit"))
                }
            }
            Err(err) => {
                Err(err.to_string())
            }
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


    fn read_doms_for_page(&self, page: &TrackedPage) -> Result<Vec<StoredDom<T>>, String> {
        self.read_doms_for_page_id(page.page_id())
    }

    fn read_latest_dom_for_page(&self, page: &TrackedPage) -> Result<StoredDom<T>, String> {
        let conn = self.get_sql_conn();

        let mut statement = conn.prepare(format!("SELECT * FROM {} WHERE PAGE_ID=? ORDER BY rowid DESC LIMIT 1", TRACKED_PAGES_DOMS).as_str()).unwrap();

        return match statement.query(params![page.page_id()]) {
            Ok(mut rows) => {
                match rows.next() {
                    Ok(row) => {
                        if let Some(row_i) = row {
                            Ok(StoredDom::new(row_i.get(0).unwrap(),
                                              row_i.get(1).unwrap(),
                                              row_i.get(2).unwrap()))
                        } else {
                            Err(String::from("Could not find dom for page"))
                        }
                    }
                    Err(e) => { Err(e.to_string()) }
                }
            }
            Err(e) => { Err(e.to_string()) }
        };
    }

    fn insert_dom_for_page(&self, page: &TrackedPage, page_dom: T) -> Result<StoredDom<T>, String> {
        let write_guard = self.write_sql_conn();

        let mut update = write_guard
            .prepare(format!("INSERT INTO {}(PAGE_ID, DOM) values(?, ?)", TRACKED_PAGES_DOMS).as_str()).unwrap();

        return match update.execute(params![page.page_id(), page_dom]) {
            Ok(count) => {
                if count > 0 {
                    Ok(StoredDom::new(write_guard.last_insert_rowid() as u32,
                                      page.page_id(), page_dom))
                } else {
                    Err(String::from("Failed to insert into the DB"))
                }
            }
            Err(e) => {
                Err(e.to_string())
            }
        };
    }

    fn update_dom_for_page(&self, _page: &TrackedPage, dom: &mut StoredDom<T>, page_dom: T) -> Result<(), String> {
        let guard = self.get_sql_conn();

        let mut update = guard
            .prepare(format!("UPDATE {} SET DOM=? WHERE rowid=?", TRACKED_PAGES_DOMS).as_str())
            .unwrap();

        match update.execute(params![page_dom, dom.dom_id()]) {
            Ok(_) => {
                dom.set_dom(page_dom);

                Ok(())
            }
            Err(e) => {
                Err(e.to_string())
            }
        }
    }

    fn delete_dom_for_page(&self, _page: &TrackedPage, dom: StoredDom<T>) -> Result<bool, String> {
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

impl<T> UserDB for SQLLiteDefacementDB<T> where T: Display + FromSql + ToSql + Send + Sync {
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
                    return Ok(User::new(row.get(0).unwrap(), row.get(1).unwrap()));
                }

                Err(String::from("Could not find user."))
            }

            Err(e) => {
                Err(e.to_string())
            }
        };
    }

    fn get_user_info_for_id(&self, user_id: u32) -> Result<User, String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.prepare(format!("SELECT * FROM {} WHERE rowid=?", USERS)
            .as_str()).unwrap();

        return match statement.query(params![user_id]) {
            Ok(mut rows) => {
                if let Some(row) = rows.next().unwrap() {
                    return Ok(User::new(row.get(0).unwrap(), row.get(1).unwrap()));
                }

                Err(String::from("Could not find user."))
            }
            Err(e) => {
                Err(e.to_string())
            }
        };
    }

    fn delete_user(&self, user: User) -> Result<bool, String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.prepare(format!("DELETE FROM {} WHERE rowid=?", USERS).as_str()).unwrap();

        return match statement.execute(params![user.user_id()]) {
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

    fn insert_contact_for(&self, user: &User, comm: CommData) -> Result<UserCommunication, String> {
        let connection = self.get_sql_conn();

        let mut statement = connection.prepare(format!("INSERT INTO {} (USER_ID, CONTACT_TYPE, CONTACT) values(?, ?, ?)", USER_CONTACTS).as_str()).unwrap();

        let mut final_result: Option<Result<usize, Error>> = Option::None;

        match &comm {
            CommData::Email(mail) => {
                final_result = Some(statement.execute(params![user.user_id(), "EMAIL", mail]));
            }
        }

        match final_result.unwrap() {
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
                    match self.parse_contact_from_row(row) {
                        Ok(contact) => {
                            contacts.push(contact);
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }

        Ok(contacts)
    }

    fn get_contact_for_id(&self, contact_id: u32) -> Result<UserCommunication, String> {
        let conn = self.get_sql_conn();

        let mut statement = conn.prepare(format!("SELECT * FROM {} WHERE rowid=?", contact_id).as_str()).unwrap();

        return match statement.query(params![contact_id]) {
            Ok(mut rows) => {
                if let Some(row) = rows.next().unwrap() {
                    return self.parse_contact_from_row(row);
                }

                Err(String::from("There is no communication by that ID."))
            }
            Err(err) => { Err(err.to_string()) }
        };
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
    use crate::communication::CommData::Email;
    use crate::databases::{UserDB, WebsiteDefacementDB};
    use crate::databases::sqlitedb::SQLLiteDefacementDB;

    #[test]
    fn test_sqlite_tracked_page() {
        let db: SQLLiteDefacementDB<String> = SQLLiteDefacementDB::new();

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
        let db: SQLLiteDefacementDB<String> = SQLLiteDefacementDB::new();

        let page = "https://google.com";

        let dom = "<>";

        let result = db.insert_tracked_page(page, 0);

        assert!(result.is_ok());

        let tracked_page = result.unwrap();

        let result_insert_dom = db.insert_dom_for_page(&tracked_page, String::from(dom));

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

    #[test]
    fn test_user_db() {
        let db: SQLLiteDefacementDB<String> = SQLLiteDefacementDB::new();

        let username = "teste";

        let result_created_user = db.create_user(username);

        assert!(result_created_user.is_ok());

        let created_user = result_created_user.unwrap();

        let result_user_info = db.get_user_info_for(username);

        assert!(result_user_info.is_ok());

        let user_info = result_user_info.unwrap();

        assert_eq!(created_user, user_info);

        let contact = db.insert_contact_for(&user_info, Email(String::from("nunonuninho2@gmail.com"))).unwrap();

        let result_contact_list = db.list_contacts_for(&user_info);

        assert!(result_contact_list.is_ok());

        let contact_list = result_contact_list.unwrap();

        assert_eq!(vec![contact.clone()], contact_list);

        let result_delete_contact = db.delete_contact(contact);

        assert!(result_delete_contact.is_ok());

        let result_delete_user = db.delete_user(user_info);

        assert!(result_delete_user.is_ok() && result_delete_user.unwrap());

        let result_user_info_after_delete = db.get_user_info_for(username);

        assert!(result_user_info_after_delete.is_err());
    }
}