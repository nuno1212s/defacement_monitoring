use std::collections::BTreeSet;
use std::io::{BufRead, StdinLock};
use std::num::ParseIntError;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::{debug, error, trace, warn};

use tokio::time;

use crate::communication::{CommData, CommunicationMethod, UserCommunication};
use crate::comparators::{Comparator, CompareResult};
use crate::comparators::diff_comparator::{analyse_dynamic_page, compare_dom_with_diff};
use crate::databases::{DEFAULT_INDEXING_INTERVAL, StoredDom, TrackedPage, TrackedPageType, User, UserDB, WebsiteDefacementDB};
use crate::databases::TrackedPageType::Dynamic;
use crate::DiffComparator;
use crate::parsers::Parser;

/*
60 minutes between checks
 */
const TIME_BETWEEN_CHECKS: u128 = 1 * 1 * 1000;
const TIME_BETWEEN_INDEX_CHECKS: u128 = 30 * TIME_BETWEEN_CHECKS;
///60 seconds between attempting checks
const TIME_INTERVAL: Duration = Duration::from_millis(1 * 1000);

pub struct PageManager<T, V, K> where
    T: WebsiteDefacementDB,
    V: UserDB,
    K: Parser {
    //A set of all page_id that are currently being indexed
    currently_indexing: Mutex<BTreeSet<u32>>,
    tracked_page_db: T,
    user_db: V,
    parser: K,
    comparators: Vec<Box<dyn Comparator>>,
    communications: Vec<Box<dyn CommunicationMethod>>,
}

impl<T, V, K> PageManager<T, V, K>
    where T: WebsiteDefacementDB + 'static,
          V: UserDB + 'static,
          K: Parser + 'static {
    pub fn new(tracked_page_db: T, user_db: V, parser: K,
               comparators: Vec<Box<dyn Comparator>>,
               communications: Vec<Box<dyn CommunicationMethod>>) -> Self {
        Self {
            currently_indexing: Mutex::new(BTreeSet::new()),
            tracked_page_db,
            user_db,
            parser,
            comparators,
            communications,
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

    fn show_menu(self: &Arc<Self>) {
        let stdin1 = std::io::stdin();

        let mut stdin = stdin1.lock();

        loop {
            println!("=============================================");
            println!("1- List all currently tracked pages.");
            println!("2- Register a tracked page.");
            println!("3- Remove a tracked page.");
            println!("4- Edit tracked page.");
            println!("5- Force rescan of tracked page. \
            (This is useful for when you make changes to the page and want to update the stored dom.)");
            println!("6- Get user info for username.");
            println!("7- Register new user.");
            println!("8- Delete user.");
            println!("9- Register contact for user.");
            println!("10- Delete contact for user.");
            println!("=============================================");

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
                            println!("You have successfully inserted the page.\
                             The page ID is {}", tracked_page.page_id());

                            self.alter_tracked_page(&mut stdin, tracked_page);
                        }
                        Err(e) => {
                            println!("Failed to insert the page because {}", e);
                        }
                    }
                }
                3 => {
                    self.remove_tracked_page(&mut stdin);
                }
                4 => {
                    match self.read_page_from_stdin(&mut stdin) {
                        Ok(mut page) => {
                            self.alter_tracked_page(&mut stdin, page);
                        }
                        Err(e) => {
                            println!("{}", e);
                        }
                    }
                }
                5 => {
                    match self.read_page_from_stdin(&mut stdin) {
                        Ok(mut page) => {
                            tokio::task::spawn(self.clone().analyse_page(page));
                        }
                        Err(e) => { println!("FAILED {}", e); }
                    }
                }
                6 => {
                    self.display_user_id(&mut stdin);
                }
                7 => {
                    match self.insert_new_user(&mut stdin) {
                        Ok(user) => {
                            println!("The user with the username {} has been created succesfully and has the id {}",
                                     user.user(), user.user_id())
                        }
                        Err(e) => { println!("Failed to create user because {}", e) }
                    }
                }
                8 => {
                    self.delete_user(&mut stdin);
                }
                9 => {
                    match self.read_user_info(&mut stdin) {
                        Ok(user) => {
                            self.insert_contact_for(&mut stdin, &user);
                        }
                        Err(error) => {
                            println!("{}", error)
                        }
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

    fn read_page_from_stdin(&self, stdin: &mut StdinLock) -> Result<TrackedPage, String> {
        println!("Please enter the page ID.");

        let mut line = String::new();

        match stdin.read_line(&mut line) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("Failed to read page ID. {:?}", e));
            }
        }

        line.pop();

        let parsed_number_result = line.parse::<u32>();

        return match parsed_number_result {
            Ok(page_id) => {
                Ok(self.tracked_page_db().get_information_for_tracked_page(page_id)?)
            }
            Err(e) => {
                Err(format!("Failed to read page ID {:?}", e))
            }
        };
    }

    fn alter_tracked_page(self: &Arc<Self>, stdin: &mut StdinLock, mut tracked_page: TrackedPage) {
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
                        tracked_page.set_tracked_page_type(TrackedPageType::Static);
                    }
                    2 => {
                        tracked_page.set_tracked_page_type(TrackedPageType::Dynamic(-1.0));
                    }
                    _ => {}
                }
            }
            Err(e) => {
                println!("Failed to read your choice. {:?}, {}", e, line);
                return;
            }
        }

        println!("How regularly do you wish your page to be re indexed (Choose this depending on the amount of cumulative changes you think your page will have over that period of time)");
        println!("Please insert time in minutes");
        println!("Press ENTER for DEFAULT ({} minutes)", Duration::from_millis(DEFAULT_INDEXING_INTERVAL).as_secs() / 60);

        let read_time = stdin.read_line(&mut line);

        line.pop();

        let mut time_in_millis: u128;

        if line.is_empty() {
            time_in_millis = DEFAULT_INDEXING_INTERVAL as u128
        } else {
            time_in_millis = Duration::from_secs((line.parse::<u32>().unwrap() * 60) as u64).as_millis();
        }

        tracked_page.set_index_interval(time_in_millis);

        tokio::task::spawn(self.clone().analyse_page(tracked_page));
    }

    fn display_all_tracked_pages(&self) {
        let pages_res = self.tracked_page_db().list_all_tracked_pages();

        match pages_res {
            Ok(pages) => {
                for page in &pages {
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

                if pages.is_empty() {
                    println!("There are no tracked pages!");
                }
            }
            Err(e) => {
                println!("Failed to read pages because {}", e);
            }
        }
    }

    fn remove_tracked_page(&self, stdin: &mut StdinLock) {
        println!("Insert the ID of the page you want to stop tracking.");

        let mut line: String = String::new();

        match stdin.read_line(&mut line) {
            Ok(_) => { line.pop(); }
            Err(e) => {
                println!("Failed to read input! {:?}", e);

                return;
            }
        }

        let page_id_res = line.parse::<u32>();

        match page_id_res {
            Ok(page_id) => {
                match self.tracked_page_db().get_information_for_tracked_page(page_id) {
                    Ok(page) => {
                        self.tracked_page_db().del_tracked_page(page);
                    }
                    Err(e) => {
                        println!("There is no page with that ID {}! {}", page_id, e);

                        return;
                    }
                }
            }
            Err(e) => {
                println!("Failed to parse page ID {:?}", e);

                return;
            }
        }
    }

    ///Fetch which pages need haven't been checked in a while and checks them
    fn check_pages(self: &Arc<Self>) {
        {
            let result = self.tracked_page_db()
                .list_all_pages_not_indexed_for();

            match result {
                Ok(pages_not_indexed) => {
                    for page_to_index in pages_not_indexed {
                        if page_to_index.defacement_count() > 0 {
                            ///We don't want to automatically reindex a page that has an even
                            /// tiny chance of currently being defaced, as that would mean we would
                            /// not be able to detect it at all
                            ///This reindexing has to be done by hand when the changes
                            ///Are sufficient to trigger a defacement warning
                            continue;
                        }

                        let self_cpy = self.clone();

                        tokio::task::spawn_blocking(move || { self_cpy.analyse_page(page_to_index) });
                    }
                }
                Err(e) => {
                    error!("Failed to check pages because of error: {}", e)
                }
            }
        }
        {
            let result = self.tracked_page_db()
                .list_all_pages_not_checked_for(TIME_BETWEEN_CHECKS);

            match result {
                Ok(mut pages_not_checked) => {
                    for page in pages_not_checked {
                        let self_res = self.clone();

                        tokio::task::spawn_blocking(move || { self_res.check_singular_page(page) });
                    }
                }
                Err(e) => {
                    error!("Failed to check pages because of error: {}", e)
                }
            }
        }
    }

    ///Analyse a given page and check if it has been defaced
    ///Runs all the comparison algorithms provided in PageManager initialization
    fn check_singular_page(self: Arc<Self>, mut page: TrackedPage) {
        {
            let currently_index = self.currently_indexing.lock().unwrap();

            if currently_index.contains(&page.page_id()) {
                //We do not want to check the page if it is currently being indexed.
                return;
            }
        }

        let result_doms = self.tracked_page_db()
            .read_doms_for_page(&page);

        let doms = result_doms.unwrap();

        if doms.is_empty()
        {
            debug!("Tried to check page {} with ID {} but there are no stored doms, please take a look at this.",
                     page.page_url(), page.page_id());

            return;
        }

        //Allow the scheduler to take over a different task
        //while we are reading the dom from the parser, which might take a while
        //As it has to fetch the result
        let dom_result = self.read_current_page_for(&page);

        let current_dom = dom_result.expect("Failed to read current dom.");

        let mut latest_dom = &doms[0];

        for dom in &doms {
            if dom.dom_id() > latest_dom.dom_id() {
                latest_dom = dom;
            }
        }

        if !self.verify_page(&page, latest_dom, &current_dom) {
            let mut notify = false;

            if page.defacement_count() + 1 >= page.defacement_threshold() && !page.notified_of_current_breach() {
                notify = true;
            }

            match self.tracked_page_db().increment_defacement_count(&mut page, notify) {
                Ok(_) => {}
                Err(error) => {
                    error!("Failed to increment defacement count of {} because {}", page.page_id(), error);
                }
            }

            debug!("Page now has {} defacements out of {} possible ones", page.defacement_count(), page.defacement_threshold());

            if !notify {
                return;
            }

            let owning_user = self.user_db()
                .get_user_info_for_id(page.owning_user_id());

            match owning_user {
                Ok(user) => {
                    let contacts_res = self.user_db().list_contacts_for(&user);

                    match contacts_res {
                        Ok(contacts) => {
                            if contacts.is_empty() {
                                warn!("Found defacement in tracked page {} with id {} \
                                but owner {} with id {} has not registered contacts.",
                                page.page_url(), page.page_id(), user.user(), user.user_id());

                                return;
                            }

                            for contact in &contacts {
                                for comm_method in &self.communications {
                                    match (*comm_method).send_report_to(&user, contact, &page,
                                                                        latest_dom, current_dom.as_str()) {
                                        Ok(_) => {
                                            debug!("Sent notification to user {} with ID {} about defacement on page {} with id {}",
                                                     user.user(), user.user_id(), page.page_url(),
                                                     page.page_id());
                                        }
                                        Err(e) => {
                                            error!("Failed to contact user {} with ID {}\
                                                             on communication method {:?} for \
                                                             tracked page defacement {} with page ID {}. {}",
                                                     user.user(), user.user_id(), contact, page.page_url(),
                                                     page.page_id(), e);
                                        }
                                    };

                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to load contacts for user {}, {}", page.owning_user_id(), e);
                        }
                    }
                }
                Err(e) => {
                    error!("DETECTED DEFACEMENT IN PAGE {} BUT COULD NOT FIND \
                                    USER INFO FOR OWNER {}, {}", page.page_url(),
                             page.owning_user_id(), e);
                }
            }
        } else {
            match self.tracked_page_db().reset_defacement_count(&mut page) {
                Ok(_) => {}
                Err(error) => {
                    error!("Failed to reset defacement count {}", error);
                }
            }
        }
    }

    ///Verifies if the page is as it's supposed to be.
    ///Returns true if the page is good (not defaced)
    ///Returns false if the page is not good (defaced)
    fn verify_page(&self, page: &TrackedPage, stored_dom: &StoredDom, current_dom: &String) -> bool {
        for comparator in &self.comparators {
            let result = comparator.compare_between(page, stored_dom.dom(),
                                                    current_dom.as_str());

            match result {
                CompareResult::NotDefaced => {
                    trace!("Not defaced, page {}, comparator {}", page.page_url(),
                    comparator.name());
                    return true;
                }
                CompareResult::MaybeDefaced => {
                    trace!("Inconclusive result for page {} with ID {},\
                     could not determine if page was defaced or not with comparator {}", page.page_url(), page.page_id(),
                             comparator.name())
                }
                CompareResult::Defaced => {
                    trace!("Defaced, page {}, comparator {}", page.page_url(), comparator.name());
                    return false;
                }
            }
        }

        true
    }

    ///Fetch the latest version doms, insert it into the DB
    ///When the page is registered as a dynamic page, also performs
    ///An analysis for 5 minutes, taking samples every 30 seconds and does an average of the diff between
    ///them so we can calculate a threshold that will trigger a defacement alarm
    async fn analyse_page(self: Arc<Self>, mut page: TrackedPage) {
        debug!("Analysing page {} with ID {}", page.page_url(), page.page_id());

        //We insert this scope here as the compiler is still not capable of detecting a drop(),
        //So when we do the await further up ahead, which gives the tokio runtime permission
        //To put this task to sleep, the mutex guard isn't part of the data that needs to be
        //Stored (Mutex guards are not Send)
        {
            let mut currently_indexing = self.currently_indexing.lock().unwrap();

            if currently_indexing.contains(&page.page_id()) {
                return;
            }

            currently_indexing.insert(page.page_id());
        }

        match page.tracked_page_type() {
            TrackedPageType::Static => {
                let dom_res = self.read_current_page_for(&page);

                match dom_res {
                    Ok(dom) => {
                        self.tracked_page_db().insert_dom_for_page(&page, dom.as_str()).unwrap();
                        debug!("Inserted DOM for page {} with ID {}", page.page_url(), page.page_id());
                    }
                    Err(e) => {
                        error!("FAILED TO ANALYSE PAGE {}, PLEASE FIX WHAT IS WRONG. {}", page.page_url(), e);
                    }
                }
            }
            Dynamic(_) => {
                let result = analyse_dynamic_page(self.parser(), &page).await;

                page.set_tracked_page_type(Dynamic(result.unwrap()));

                let dom_res = self.read_current_page_for(&page);

                match dom_res {
                    Ok(dom) => {
                        self.tracked_page_db().insert_dom_for_page(&page, dom.as_str()).unwrap();
                        debug!("Inserted DOM for page {} with ID {}", page.page_url(), page.page_id());
                    }
                    Err(e) => {
                        error!("FAILED TO ANALYSE PAGE {}, PLEASE FIX WHAT IS WRONG. {}", page.page_url(), e);
                    }
                }
            }
        }

        self.tracked_page_db().update_tracking_type_for_page(&page).unwrap();

        {
            let mut currently_indexing = self.currently_indexing.lock().unwrap();

            currently_indexing.remove(&page.page_id());
        }
    }

    fn read_current_page_for(&self, page: &TrackedPage) -> Result<String, String> {
        self.parser().parse_page(page)
    }

    fn insert_new_user(&self, stdin: &mut StdinLock) -> Result<User, String> {
        println!("Insert the username of the user");

        let mut username = String::new();

        match stdin.read_line(&mut username) {
            Ok(_) => {}
            Err(e) => { return Err(e.to_string()); }
        };

        username.pop();

        self.user_db().create_user(username.as_str())
    }

    fn display_user_id(&self, stdin: &mut StdinLock) {
        println!("Please insert the username.");

        let mut username = String::new();

        match stdin.read_line(&mut username) {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to read username because {:?}.", e);
                return;
            }
        };

        username.pop();

        let user_result = self.user_db().get_user_info_for(username.as_str());

        match user_result {
            Ok(user) => {
                println!("That username corresponds to the user ID {}.", user.user_id());

                let contacts_result = self.user_db().list_contacts_for(&user);

                match contacts_result {
                    Ok(contacts) => {
                        if contacts.is_empty() {
                            println!("This user has no contacts associated.");
                            return;
                        }

                        for contact in &contacts {
                            println!("{} - {} - {}", contact.comm_id(),
                                     match contact.communication() { CommData::Email(_) => { "Email" } },
                                     match contact.communication() { CommData::Email(lettre_email) => { lettre_email } })
                        }
                    }
                    Err(error) => {
                        println!("Failed to get contacts for user {}. {}", user.user(), error);
                    }
                }
            }
            Err(e) => { println!("Could not find a user by that name? {}", e); }
        }
    }

    fn delete_user(&self, stdin: &mut StdinLock) {
        println!("Insert the username.");

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

    fn read_user_info(&self, stdin: &mut StdinLock) -> Result<User, String> {
        println!("Please insert the user ID.");

        let mut line = String::new();

        match stdin.read_line(&mut line) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("Failed to read username because {:?}.", e));
            }
        };

        line.pop();

        let parsed_id = line.parse::<u32>();

        return match parsed_id {
            Ok(page_id) => {
                self.user_db().get_user_info_for_id(page_id)
            }
            Err(err) => {
                Err(format!("Failed to read user id {:?}.", err))
            }
        };
    }

    fn insert_contact_for(&self, stdin: &mut StdinLock, user: &User) {
        println!("Insert contact email.");

        let mut line = String::new();

        match stdin.read_line(&mut line) {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to read email because {:?}.", e);
            }
        };

        line.pop();

        match self.user_db().insert_contact_for(user, CommData::Email(line.clone())) {
            Ok(res) => {
                println!("Inserted contact {} with ID {}", line, res.comm_id());
            }
            Err(error) => {
                println!("Failed to insert contact because {}", error);
            }
        }
    }

    fn delete_contact_for(&self, stdin: &mut StdinLock, user: &User) {
        println!("Insert contact id.");

        let mut line = String::new();

        match stdin.read_line(&mut line) {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to read email because {:?}.", e);
            }
        };

        line.pop();

        let comm_id_res = line.parse::<u32>();

        match comm_id_res {
            Ok(comm_id) => {
                match self.user_db().get_contact_for_id(comm_id) {
                    Ok(contact) => {
                        match self.user_db().delete_contact(contact) {
                            Ok(res) => {
                                if res {
                                    println!("Deleted contact successfully");
                                } else {
                                    println!("Failed to delete contact");
                                }
                            }
                            Err(error) => {
                                println!("Failed to delete contact because {}", error);
                            }
                        }
                    }
                    Err(err) => {
                        println!("There is no contact with that ID {}", err);
                    }
                }
            }
            Err(error) => {
                println!("Failed to read comm ID. {}", error);
            }
        }
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