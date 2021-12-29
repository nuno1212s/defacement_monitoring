use crate::databases::TrackedPage;

pub mod chromium_parser;

pub trait Parser<T>: Send + Sync {

    fn parse_page(&self, page: &TrackedPage) -> Result<T, String>;

}