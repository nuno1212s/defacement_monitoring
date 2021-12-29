use std::cmp::Ordering;
use std::time::Duration;
use crate::communication::{CommData, UserCommunication};

pub mod sqlitedb;

const DEFAULT_DEFACEMENT_THRESHOLD: u32 = 5;
pub const DEFAULT_INDEXING_INTERVAL: u64 = Duration::from_secs(60 * 30).as_millis() as u64;

#[derive(PartialEq, Debug, Clone)]
pub struct TrackedPage {
    page_id: u32,
    page_url: String,
    owning_user_id: u32,
    last_time_checked: u128,
    last_time_indexed: u128,
    index_interval: u128,
    tracked_page_type: TrackedPageType,
    defacement_count: u32,
    defacement_threshold: u32,
    notified_of_current_breach: bool,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TrackedPageType {
    Static,
    /*
    Stores the threshold at which we start to notify that the page has been defaced
     */
    Dynamic(f64),
}

#[derive(PartialEq, Debug, Clone)]
pub struct StoredDom {
    dom_id: u32,
    owning_page_id: u32,
    dom: String,
}

#[derive(PartialEq, Debug, Clone)]
pub struct User {
    user_id: u32,
    user: String,
}

pub trait WebsiteDefacementDB: Send + Sync {
    fn insert_tracked_page(&self, page: &str, user_id: u32) -> Result<TrackedPage, String>;

    fn list_all_tracked_pages(&self) -> Result<Vec<TrackedPage>, String>;

    fn list_all_pages_not_checked_for(&self, time_since_last_check: u128) -> Result<Vec<TrackedPage>, String>;

    fn list_all_pages_not_indexed_for(&self) -> Result<Vec<TrackedPage>, String>;

    fn get_information_for_page(&self, page: &str) -> Result<TrackedPage, String>;

    fn get_information_for_tracked_page(&self, page_id: u32) -> Result<TrackedPage, String>;

    fn update_tracking_type_for_page(&self, page: &TrackedPage) -> Result<bool, String>;

    ///Should also set the value of the object we were passed as the correct
    /// Value that is stored in the database
    ///notified is whether the system notified the user (If he had already been notified, this will be false)
    fn increment_defacement_count(&self, page: &mut TrackedPage, notified: bool) -> Result<(), String>;

    ///Should also reset the value of the object we were passed
    fn reset_defacement_count(&self, page: &mut TrackedPage) -> Result<(), String>;

    fn del_tracked_page(&self, page: TrackedPage) -> Result<bool, String>;

    fn read_doms_for_page(&self, page: &TrackedPage) -> Result<Vec<StoredDom>, String>;

    fn read_latest_dom_for_page(&self, page: &TrackedPage) -> Result<StoredDom, String>;

    fn insert_dom_for_page(&self, page: &TrackedPage, page_dom: &str) -> Result<StoredDom, String>;

    fn update_dom_for_page(&self, page: &TrackedPage, dom: &mut StoredDom, page_dom: &str) -> Result<(), String>;

    fn delete_dom_for_page(&self, page: &TrackedPage, dom: StoredDom) -> Result<bool, String>;
}

pub trait UserDB: Send + Sync {
    fn create_user(&self, user_name: &str) -> Result<User, String>;

    fn get_user_info_for(&self, user_name: &str) -> Result<User, String>;

    fn get_user_info_for_id(&self, user_id: u32) -> Result<User, String>;

    fn delete_user(&self, user: User) -> Result<bool, String>;

    fn insert_contact_for(&self, user: &User, comm: CommData) -> Result<UserCommunication, String>;

    fn list_contacts_for(&self, user: &User) -> Result<Vec<UserCommunication>, String>;

    fn get_contact_for_id(&self, contact_id: u32) -> Result<UserCommunication, String>;

    fn delete_contact(&self, comm: UserCommunication) -> Result<bool, String>;
}

impl TrackedPage {
    pub fn new(page_id: u32, page_url: String, owning_user_id: u32, last_time_checked: u128,
               last_time_indexed: u128, index_interval: u128, defacement_count: u32, defacement_threshold: u32,
               notified_of_current: bool,
               tracked_type: TrackedPageType) -> Self {
        Self {
            page_id,
            page_url,
            owning_user_id,
            last_time_checked,
            last_time_indexed,
            index_interval,
            defacement_count,
            defacement_threshold,
            notified_of_current_breach: notified_of_current,
            tracked_page_type: tracked_type,
        }
    }

    pub fn page_id(&self) -> u32 {
        self.page_id
    }
    pub fn page_url(&self) -> &str {
        &self.page_url
    }
    pub fn owning_user_id(&self) -> u32 {
        self.owning_user_id
    }
    pub fn last_time_checked(&self) -> u128 {
        self.last_time_checked
    }
    pub fn tracked_page_type(&self) -> &TrackedPageType {
        &self.tracked_page_type
    }
    pub fn set_tracked_page_type(&mut self, tracked_page_type: TrackedPageType) {
        self.tracked_page_type = tracked_page_type;
    }
    pub fn last_time_indexed(&self) -> u128 {
        self.last_time_indexed
    }
    pub fn defacement_count(&self) -> u32 {
        self.defacement_count
    }
    pub fn defacement_threshold(&self) -> u32 {
        self.defacement_threshold
    }
    pub fn notified_of_current_breach(&self) -> bool {
        self.notified_of_current_breach
    }
    pub fn set_defacement_count(&mut self, defacement_count: u32) {
        self.defacement_count = defacement_count;
    }
    pub fn set_notified_of_current_breach(&mut self, notified_of_current_breach: bool) {
        self.notified_of_current_breach = notified_of_current_breach;
    }
    pub fn index_interval(&self) -> u128 { self.index_interval }
    pub fn set_index_interval(&mut self, index_interval: u128) {
        self.index_interval = index_interval;
    }
}

impl StoredDom {
    pub fn new(dom_id: u32, owning_page_id: u32, dom: String) -> Self {
        Self { dom_id, owning_page_id, dom }
    }

    pub fn dom_id(&self) -> u32 {
        self.dom_id
    }
    pub fn owning_page_id(&self) -> u32 {
        self.owning_page_id
    }
    pub fn dom(&self) -> &str {
        &self.dom
    }

    pub fn set_dom(&mut self, dom: String) {
        self.dom = dom;
    }
}

impl User {
    pub fn user_id(&self) -> u32 {
        self.user_id
    }
    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn new(user_id: u32, user: String) -> Self {
        Self { user_id, user }
    }
}

pub fn tracked_page_type_to_str(page_type: &TrackedPageType) -> &str {
    match page_type {
        TrackedPageType::Static => {
            "Static"
        }
        TrackedPageType::Dynamic(_) => {
            "Dynamic"
        }
    }
}