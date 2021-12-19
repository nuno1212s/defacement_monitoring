use std::process::Command;

const CHROME_HEADLESS: &str = "chromium";
const HEADLESS: &str = "--headless";
const DUMP_TO_DOM: &str = "--dump-dom";

/**
Runs chromium headless to render and then obtain the websites we want
This allows our program to get the full website after
 */
pub fn read_website_to_dom(website: &str) -> String {
    let result = Command::new(CHROME_HEADLESS)
        .arg(HEADLESS)
        .arg(DUMP_TO_DOM)
        .arg(website).output();

    let final_res = result.expect("Failed to execute chrome to fetch website");

    String::from_utf8(final_res.stdout).expect("Failed to parse return text")
}

pub fn read_website_to_pdf(website: &str) {}