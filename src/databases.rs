use crate::communication::{CommData, UserCommunication};

mod sqlitedb;

#[derive(PartialEq, Debug, Clone)]
pub struct TrackedPage {
    page_id: u32,
    page_url: String,
    owning_user_id: u32,
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
    user: String
}

pub trait WebsiteDefacementDB {
    fn insert_tracked_page(&self, page: &str, user_id: u32) -> Result<TrackedPage, String>;

    fn list_all_tracked_pages(&self) -> Result<Vec<TrackedPage>, String>;

    fn get_information_for_page(&self, page: &str) -> Result<TrackedPage, String>;

    fn get_information_for_tracked_page(&self, page_id: u32) -> Result<TrackedPage, String>;

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

    fn register_contact(&self, user: &User, comm: CommData) -> Result<UserCommunication, String>;

    fn list_contacts_for(&self, user: &User) -> Result<Vec<UserCommunication>, String>;

    fn delete_contact(&self, comm: UserCommunication) -> Result<bool, String>;
}

impl TrackedPage {
    pub fn new(page_id: u32, page_url: String, owning_user_id: u32) -> Self {
        Self {
            page_id,
            page_url: page_url,
            owning_user_id,
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