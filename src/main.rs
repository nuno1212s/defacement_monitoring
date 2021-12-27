use std::sync::Arc;
use crate::comparators::checksum_comparator::ChecksumComparator;
use crate::comparators::Comparator;
use crate::comparators::diff_comparator::DiffComparator;
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

    let comparators: Vec<Box<dyn Comparator>> = vec![Box::new(ChecksumComparator::new()),
                                                     Box::new(DiffComparator::new())];

    let page_manager = Arc::new(PageManager::new(database.clone(), database,
                                                 parser, comparators));

    page_manager.start().await;
}
