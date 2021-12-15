use std::os::linux::raw::stat;
use sqlite::{Connection, State};
use sqlite::State::Done;

use crate::databases::database::WebsiteDefacementDB;

const TRACKED_PAGES_TABLE: String = String::from("TRACKED_PAGES");

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
}

impl WebsiteDefacementDB for SQLLiteDefacementDB {
    fn insert_tracked_page(&self, page: &str) {
        let mut statement = self.get_sql_conn().prepare(
            format!("INSERT INTO {}(PAGE_URL) values(?)", TRACKED_PAGES_TABLE)).unwrap();

        statement.bind(1, page);

        match statement.next() {
            Ok(state) => {
                if state != Done {
                    println!("Failed to insert into DB")
                }
            }
            Err(_) => {
                println!("Failed to insert into DB")
            }
        }
    }

    fn list_all_tracked_pages(&self) -> Vec<String> {
        let mut result = self.get_sql_conn().prepare(
            format!("SELECT * FROM {}", TRACKED_PAGES_TABLE)
        ).unwrap();

        let mut return_vec = Vec::new();

        while let State::Row = result.next().unwrap() {
            return_vec.push(result.read::<String>(0).unwrap());
        }

        return_vec
    }

    fn del_tracked_page(&self, page: &str) -> bool {
        let mut statement = self.get_sql_conn().prepare(
            format!("DELETE FROM {} WHERE LOWER(PAGE_URL)=LOWER(?)", TRACKED_PAGES_TABLE)).unwrap();

        statement.bind(1, page);

        match statement.next() {
            Ok(state) => {
                if state == State::Done {
                    return true;
                }

                return false;
            }
            Err(_) => {
                false
            }
        }
    }

    fn read_dom_for_page(&self, page: &str) -> String {
        todo!()
    }

    fn update_dom_for_page(&self, page: &str, page_dom: &str) {
        todo!()
    }
}