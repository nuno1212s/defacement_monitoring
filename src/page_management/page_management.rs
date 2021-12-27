use std::io::{BufRead, StdinLock};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::time;
use crate::comparators::{Comparator, CompareResult};

use crate::databases::{StoredDom, TrackedPage, TrackedPageType, User, UserDB, WebsiteDefacementDB};
use crate::databases::TrackedPageType::Dynamic;
use crate::parsers::Parser;

/*
60 minutes between checks
 */
const TIME_BETWEEN_CHECKS: u128 = 60 * 60 * 1000;
///60 seconds between attempting checks
const TIME_INTERVAL: Duration = Duration::from_millis(60 * 1000);

pub struct PageManager<T, V, K> where
    T: WebsiteDefacementDB + Send + Sync,
    V: UserDB + Send + Sync,
    K: Parser + Send + Sync {
    currently_analysing_pages: Mutex<Vec<TrackedPage>>,
    tracked_page_db: T,
    user_db: V,
    parser: K,
    comparators: Vec<Box<dyn Comparator>>
}

impl<T, V, K> PageManager<T, V, K>
    where T: WebsiteDefacementDB + Send + Sync + 'static,
          V: UserDB + Send + Sync + 'static,
          K: Parser + Send + Sync + 'static {
    pub fn new(tracked_page_db: T, user_db: V, parser: K, comparators: Vec<Box<dyn Comparator>>) -> Self {

        Self {
            currently_analysing_pages: Mutex::new(Vec::new()),
            tracked_page_db,
            user_db,
            parser,
            comparators
        }
    }

    pub async fn start(self: Arc<Self>) {
        let page_man = self.clone();

        tokio::spawn(async move {
            let mut duration = time::interval(TIME_INTERVAL);

            loop {
                page_man.check_pages();

                duration.tick().await;
            }
        });

        self.show_menu();
    }

    fn show_menu(&self) {
        loop {
            println!("=============================================");
            println!("1- List all currently tracked pages.");
            println!("2- Register a tracked page.");
            println!("3- Remove a tracked page.");
            println!("4- Edit tracked page.");
            println!("5- Force rescan of tracked page. \
            (This is useful for when you make changes to the page and want to update the stored dom.)");
            println!("5- Get user ID for username.");
            println!("6- Register new user.");
            println!("7- Delete user.");
            println!("=============================================");

            let stdin1 = std::io::stdin();

            let mut stdin = stdin1.lock();

            let mut line = String::new();

            stdin.read_line(&mut line).expect("Failed to read line");

            line.pop();

            let user_inp = line.parse::<u32>();

            if !user_inp.is_ok() {
                println!("Your input is not correct. {} with input {}", user_inp.unwrap_err(), line);

                self.show_menu();
                return;
            }

            match user_inp.unwrap() {
                1 => {
                    self.display_all_tracked_pages()
                }
                2 => {
                    match self.insert_tracked_page(&mut stdin) {
                        Ok(mut tracked_page) => {
                            println!("You have successfully inserted the page. The page ID is {}", tracked_page.page_id());

                            self.alter_tracked_page(&mut stdin, &mut tracked_page);

                            match tracked_page.tracked_page_type() {
                                Dynamic(_) => {
                                    todo!("Analyse the page and get a good idea of the diff threshold")
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            println!("Failed to insert the page because {}", e);
                        }
                    }
                }
                3 => {}
                5 => {
                    self.display_user_id(&mut stdin)
                }
                6 => {
                    match self.insert_new_user(&mut stdin) {
                        Ok(user) => {
                            println!("The user with the username {} has been created succesfully and has the id {}",
                                     user.user(), user.user_id())
                        }
                        Err(e) => { println!("Failed to create user because {}", e) }
                    }
                }
                _ => { println!("Could not find that option!") }
            }
        }
    }

    fn insert_tracked_page(&self, stdin: &mut StdinLock) -> Result<TrackedPage, String> {
        println!("Insert the user ID of the owner of the page");

        let mut line = String::new();

        let read_size = stdin.read_line(&mut line).expect("Failed to read input");

        //Pop the trailing \n
        line.pop();

        let user_res = line.parse::<u32>();

        return match user_res {
            Ok(id) => {
                println!("Insert the page URL you want to track.");

                let mut page_url = String::new();

                stdin.read_line(&mut page_url);

                page_url.pop();

                self.tracked_page_db().insert_tracked_page(page_url.as_str(), id)
            }
            Err(e) => Err(e.to_string())
        };
    }

    fn alter_tracked_page(&self, stdin: &mut StdinLock, tracked_page: &mut TrackedPage) {
        println!("The page is:");
        println!("1- Static");
        println!("2- Dynamic");

        let mut line = String::new();

        let read_int = stdin.read_line(&mut line);

        line.pop();

        let read_input = line.parse::<u32>();

        match read_input {
            Ok(choice) => {
                match choice {
                    1 => {
                        tracked_page.set_tracked_page_type(TrackedPageType::Static)
                    }
                    2 => {
                        tracked_page.set_tracked_page_type(TrackedPageType::Dynamic(-1.0))
                    }
                    _ => {}
                }
            }
            Err(e) => { println!("Failed to read your choice. {:?}", e) }
        }
    }

    fn display_all_tracked_pages(&self) {
        let pages_res = self.tracked_page_db().list_all_tracked_pages();

        match pages_res {
            Ok(pages) => {
                for page in pages {
                    println!("Page with ID {} tracking url {}", page.page_id(), page.page_url());

                    match page.tracked_page_type() {
                        TrackedPageType::Static => {
                            println!("Static page.");
                        }
                        TrackedPageType::Dynamic(diff_tolerance) => {
                            println!("Dynamic page with diff threshold of {}", diff_tolerance);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to read pages because {}", e);
            }
        }
    }

    fn insert_new_user(&self, stdin: &mut StdinLock) -> Result<User, String> {
        println!("Insert the username of the user");

        let mut username = String::new();

        stdin.read_line(&mut username);

        username.pop();

        self.user_db().create_user(username.as_str())
    }

    fn display_user_id(&self, stdin: &mut StdinLock) {
        let mut username = String::new();

        stdin.read_line(&mut username);

        username.pop();

        let user_result = self.user_db.get_user_info_for(username.as_str());

        match user_result {
            Ok(user) => {
                println!("That username corresponds to the user ID {}", user.user_id());
            }
            Err(e) => { println!("Could not find a user by that name? {}", e); }
        }
    }

    fn delete_user(&self, stdin: &mut StdinLock) {
        let mut username = String::new();

        stdin.read_line(&mut username);

        username.pop();

        let user_result = self.user_db.get_user_info_for(username.as_str());

        match user_result {
            Ok(user) => {
                match self.user_db.delete_user(user) {
                    Ok(deleted) => {
                        if deleted {
                            println!("The user has been successfully deleted.");
                        } else {
                            println!("Failed to delete the user?");
                        }
                    }
                    Err(e) => {
                        println!("{:?}", e);
                    }
                };
            }
            Err(e) => { println!("Could not find a user by that name? {}", e); }
        }
    }

    ///Check which pages need haven't been checked in a while and checks them
    fn check_pages(self: &Arc<Self>) {
        let result = self.tracked_page_db()
            .list_all_pages_not_checked_for(TIME_BETWEEN_CHECKS);

        match result {
            Ok(mut pages_not_checked) => {
                let mut lock_guard = self.currently_analysing_pages.lock().unwrap();

                for page in pages_not_checked {
                    let page_clone = page.clone();

                    let self_res = self.clone();

                    tokio::spawn(async move {
                        let result_doms = self_res.tracked_page_db()
                            .read_doms_for_page(&page_clone);

                        let doms = result_doms.unwrap();

                        if doms.is_empty()
                        {
                            println!("Tried to check page {} with ID {} but there are no stored doms, please take a look at this.",
                                     page_clone.page_url(), page_clone.page_id());

                            return;
                        }

                        let dom_result = self_res.read_current_page_for(&page_clone);

                        let current_dom = dom_result.expect("Failed to read current dom.");

                        let mut latest_dom = &doms[0];

                        for dom in &doms {
                            if dom.dom_id() > latest_dom.dom_id() {
                                latest_dom = dom;
                            }
                        }

                        if !self_res.verify_page(&page_clone, latest_dom, current_dom) {
                            //TODO: Send notification to the owner
                        }
                    });

                    lock_guard.push(page);
                }
            }
            Err(e) => {
                println!("Failed to check pages because of error: {}", e)
            }
        }
    }

    fn read_current_page_for(&self, page: &TrackedPage) -> Result<String, String> {
        self.parser().parse_page(page)
    }

    ///Verifies if the page is as it's suposed to be.
    ///Returns true if the page is good (not defaced)
    ///Returns false if the page is not good (defaced)
    fn verify_page(&self, page: &TrackedPage, stored_dom: &StoredDom, current_dom: String) -> bool {
        for comparator in &self.comparators {
            let result = comparator.compare_between(page, stored_dom.dom(),
                                                    current_dom.as_str());

            match result {
                CompareResult::NotDefaced => {
                    return true;
                }
                CompareResult::MaybeDefaced => {
                    println!("Inconclusive result for page {} with ID {},\
                     could not determine if page was defaced or not with comparator {}", page.page_url(), page.page_id(),
                    comparator.name())
                }
                CompareResult::Defaced => {
                    return false;
                }
            }

        }

        true
    }

    pub fn tracked_page_db(&self) -> &T {
        &self.tracked_page_db
    }
    pub fn user_db(&self) -> &V {
        &self.user_db
    }
    pub fn parser(&self) -> &K {
        &self.parser
    }
}