use std::fmt::{Debug, Display};
use crate::databases::TrackedPage;

pub mod chromium_parser;

pub trait Parser<T>: Send + Sync where T: Debug + Display + Send + Sync {
    fn parse_page(&self, page: &TrackedPage) -> Result<T, String>;
}