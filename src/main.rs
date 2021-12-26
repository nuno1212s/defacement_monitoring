use std::sync::Arc;
use crate::databases::sqlitedb::SQLLiteDefacementDB;
use crate::parsers::chromium_parser::ChromiumParser;
use crate::page_management::page_management::PageManager;

pub mod parsers;
pub mod comparators;

pub mod page_management {
    pub mod page_management;
}

pub mod communication;
pub mod databases;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let database = SQLLiteDefacementDB::new();

    let parser = ChromiumParser::new();

    let page_manager = Arc::new(PageManager::new(database.clone(), database,
                                                 parser));

    page_manager.start().await;

}
