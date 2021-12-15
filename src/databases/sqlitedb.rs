use std::fmt::format;
use std::os::linux::raw::stat;
use sqlite::{Connection, State};
use sqlite::State::Done;

use crate::databases::database::WebsiteDefacementDB;

const TRACKED_PAGES_TABLE: String = String::from("TRACKED_PAGES");
const TRACKED_PAGES_DOMS: String = String::from("PAGES");

pub struct SQLLiteDefacementDB {
    sql_conn: Connection,
}

impl SQLLiteDefacementDB {
    fn new() -> Self {
        let connection = sqlite::open("").unwrap();

        Self {
            sql_conn: connection
        }
    }

    fn get_sql_conn(&self) -> &Connection {
        &self.sql_conn
    }

    fn read_dom_for_page_id(&self, page_id: u32) -> Result<String, String> {
        match self.get_sql_conn().prepare(format!("SELECT * FROM {} WHERE PAGE_ID=?", TRACKED_PAGES_DOMS)) {
            Ok(mut statement) => {

                statement.bind(1, page_id);

                match statement.next() {
                    Ok(state) => {
                        match state {
                            State::Row => {
                                Ok(statement.read::<String>(0).unwrap())
                            }
                            Done => {
                                Err(String::from("Could not find dom for page"))
                            }
                        }
                    }
                    Err(E) => {
                        Err(E.message.unwrap())
                    }
                }
            }
            Err(E) => { Err(E.message.unwrap()) }
        }
    }
}

impl WebsiteDefacementDB for SQLLiteDefacementDB {
    fn insert_tracked_page(&self, page: &str) -> Result<u32, String> {
        let mut statement = self.get_sql_conn().prepare(
            format!("INSERT INTO {}(PAGE_URL) values(?)", TRACKED_PAGES_TABLE)).unwrap();

        statement.bind(1, page);

        match statement.next() {
            Ok(state) => {
                if state != Done {
                    Err(String::from("Failed to insert into DB"))
                } else {
                    let statement1 = self.get_sql_conn().prepare("last_insert_id()").unwrap();

                    let result = statement1.read::<u32>(0);

                    match result {
                        Ok(id) => {
                            Ok(id)
                        }
                        Err(E) => {
                            Err(E.message.unwrap())
                        }
                    }
                }
            }
            Err(E) => {
                // println!("Failed to insert into DB")
                Err(E.message.unwrap())
            }
        }
    }

    fn list_all_tracked_pages(&self) -> Result<Vec<String>, String> {
        let mut result = self.get_sql_conn().prepare(
            format!("SELECT * FROM {}", TRACKED_PAGES_TABLE)
        ).unwrap();

        let mut return_vec = Vec::new();

        while let State::Row = result.next().unwrap() {
            return_vec.push(result.read::<String>(0).unwrap());
        }

        Ok(return_vec)
    }

    fn del_tracked_page(&self, page: &str) -> Result<bool, String> {
        let mut statement = self.get_sql_conn().prepare(
            format!("DELETE FROM {} WHERE LOWER(PAGE_URL)=LOWER(?)", TRACKED_PAGES_TABLE)).unwrap();

        statement.bind(1, page);

        match statement.next() {
            Ok(state) => {
                if state == State::Done {
                    Ok(true)
                }

                Ok(false)
            }
            Err(E) => {
                Err(E.message.unwrap())
            }
        }
    }


    fn read_dom_for_page(&self, page: &str) -> Result<String, String> {
        let mut statement = self.get_sql_conn()
            .prepare(format!("SELECT ROWID FROM {} WHERE page=?", TRACKED_PAGES_TABLE)).unwrap();

        statement.bind(1, page);

        match statement.next() {
            Ok(e) => {
                match e {
                    State::Row => {
                        let id = statement.read::<u32>(0).unwrap();

                        self.read_dom_for_page_id(id)
                    }
                    Done => {
                        Err(String::from("Could not find page"))
                    }
                }
            }
            Err(E) => {
                Err(E.message.unwrap())
            }
        }
    }

    fn update_dom_for_page(&self, page: &str, page_dom: &str) -> Result<(), String>{

        self.get_sql_conn().prepare(format!("INSERT OR REPLACE INTO {}(PAGE_ID, DOM)", TRACKED_PAGES_DOMS));

        Err(String::new())
    }
}