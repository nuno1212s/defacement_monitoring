use std::process::Command;
use crate::databases::TrackedPage;
use crate::parsers::Parser;

const CHROME_HEADLESS: &str = "chromium";
const HEADLESS: &str = "--headless";
const DUMP_TO_DOM: &str = "--dump-dom";

/**
Runs chromium headless to render and then obtain the websites we want
This allows our program to get the full website after expanding CSS, running JS and other things
 */
pub fn read_website_to_dom(website: &str) -> Result<String, String> {
    let result = Command::new(CHROME_HEADLESS)
        .arg(HEADLESS)
        .arg(DUMP_TO_DOM)
        .arg(website).output();

    return match result {
        Ok(dom) => {
            let parsed_text_result = String::from_utf8(dom.stdout);

            match parsed_text_result {
                Ok(dom_str) => {
                    Ok(dom_str)
                }
                Err(e) => { Err(e.to_string()) }
            }
        }
        Err(error) => {
            Err(error.to_string())
        }
    };
}

pub fn read_website_to_pdf(_website: &str) {}

pub struct ChromiumParser {}

impl ChromiumParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl Parser<String> for ChromiumParser {
    fn parse_page(&self, page: &TrackedPage) -> Result<String, String> {
        read_website_to_dom(page.page_url())
    }
}


#[cfg(test)]
mod parser_tests {
    use crate::parsers::chromium_parser::read_website_to_dom;

    #[test]
    fn test_parser() {
        let string = read_website_to_dom("https://jekil.sexy/blog/2009/website-defacement-detection-techniques.html");

        println!("{}", string.unwrap());
    }
}