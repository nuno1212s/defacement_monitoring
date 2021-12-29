use std::sync::Arc;
use log::error;
use log::info;
use log::warn;
use log::{debug, LevelFilter};

use crate::communication::CommunicationMethod;
use crate::communication::email::EmailCommunicator;
use crate::comparators::checksum_comparator::ChecksumComparator;
use crate::comparators::Comparator;
use crate::comparators::diff_comparator::DiffComparator;
use crate::databases::sqlitedb::SQLLiteDefacementDB;
use crate::page_management::page_management::PageManager;
use crate::parsers::chromium_parser::ChromiumParser;

pub mod parsers;
pub mod comparators;

pub mod page_management {
    pub mod page_management;
}

pub mod communication;
pub mod databases;

#[tokio::main]
async fn main() {

    env_logger::init();
    debug!("Initializing DB");

    let database = SQLLiteDefacementDB::new();

    debug!("Initializing chromium parser");

    let parser = ChromiumParser::new();

    debug!("Init comparators");
    let comparators: Vec<Box<dyn Comparator>> = vec![Box::new(ChecksumComparator::new()),
                                                     Box::new(DiffComparator::new())];

    debug!("Init email communication");

    let config_file = include_str!("../resources/email.toml");

    let communicators: Vec<Box<dyn CommunicationMethod>> = vec![Box::new(EmailCommunicator::new(config_file))];

    debug!("Initializing program....");

    let page_manager = Arc::new(PageManager::new(database.clone(), database,
                                                 parser, comparators, communicators));

    page_manager.start().await;
}
