use crate::communication::{CommData, UserCommunication};

pub mod sqlitedb;

#[derive(PartialEq, Debug, Clone)]
pub struct TrackedPage {
    page_id: u32,
    page_url: String,
    owning_user_id: u32,
    last_time_checked: u128,
    tracked_page_type: TrackedPageType,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TrackedPageType {
    Static,
    /*
    Stores the threshold at which we start to notify that the page has been defaced
     */
    Dynamic(f32),
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

pub trait WebsiteDefacementDB {
    fn insert_tracked_page(&self, page: &str, user_id: u32) -> Result<TrackedPage, String>;

    fn list_all_tracked_pages(&self) -> Result<Vec<TrackedPage>, String>;

    fn list_all_pages_not_checked_for(&self, time_since_last_check: u128) -> Result<Vec<TrackedPage>, String>;

    fn get_information_for_page(&self, page: &str) -> Result<TrackedPage, String>;

    fn get_information_for_tracked_page(&self, page_id: u32) -> Result<TrackedPage, String>;

    fn update_tracking_type_for_page(&self, page: &TrackedPage) -> Result<bool, String>;

    fn del_tracked_page(&self, page: TrackedPage) -> Result<bool, String>;

    fn read_doms_for_page(&self, page: &TrackedPage) -> Result<Vec<StoredDom>, String>;

    fn insert_dom_for_page(&self, page: &TrackedPage, page_dom: &str) -> Result<StoredDom, String>;

    fn update_dom_for_page(&self, page: &TrackedPage, dom: &mut StoredDom, page_dom: &str) -> Result<(), String>;

    fn delete_dom_for_page(&self, page: &TrackedPage, dom: StoredDom) -> Result<bool, String>;
}

pub trait UserDB {
    fn create_user(&self, user_name: &str) -> Result<User, String>;

    fn get_user_info_for(&self, user_name: &str) -> Result<User, String>;

    fn delete_user(&self, user: User) -> Result<bool, String>;

    fn insert_contact_for(&self, user: &User, comm: CommData) -> Result<UserCommunication, String>;

    fn list_contacts_for(&self, user: &User) -> Result<Vec<UserCommunication>, String>;

    fn delete_contact(&self, comm: UserCommunication) -> Result<bool, String>;
}

impl TrackedPage {
    pub fn new(page_id: u32, page_url: String, owning_user_id: u32, last_time_checked: u128,
               tracked_type: TrackedPageType) -> Self {
        Self {
            page_id,
            page_url,
            owning_user_id,
            last_time_checked,
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