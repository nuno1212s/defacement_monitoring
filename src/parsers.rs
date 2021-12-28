use crate::databases::TrackedPage;

pub mod chromium_parser;

pub trait Parser: Send + Sync {

    fn parse_page(&self, page: &TrackedPage) -> Result<String, String>;

}