use std::process::Command;

const CHROME_HEADLESS: String = String::from("chromium");
const HEADLESS: String = String::from("--headless");
const DUMP_TO_DOM: String = String::from("--dump-dom");

/**
Runs chromium headless to render and then obtain the websites we want
This allows our program to get the full website after
*/
pub fn read_website_to_dom(website: &str) -> String {
    let command = Command::new(CHROME_HEADLESS)
        .arg(HEADLESS)
        .arg(DUMP_TO_DOM)
        .arg(website);

    let result = command.output()
        .expect("Failed to execute chrome to fetch website");

    String::from_utf8(result.stdout).expect("Failed to parse return text")
}

pub fn read_website_to_pdf(website: &str) {

}