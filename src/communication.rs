use std::fmt::Debug;
use crate::databases::{StoredDom, TrackedPage, User};

pub mod email;

pub trait CommunicationMethod: Send + Sync {
    fn matches(&self, comm: &CommData) -> bool;
    fn send_report_to(&self, user: &User, comm_method: &UserCommunication, tracked_pape: &TrackedPage,
    stored_dom: &StoredDom, latest_dom: &str)
                      -> Result<String, String>;
}

#[derive(PartialEq, Debug, Clone)]
pub struct UserCommunication {
    comm_id: u32,
    user_id: u32,
    communication: CommData,
}

#[derive(PartialEq, Debug, Clone)]
pub enum CommData {
    Email(String)
}

impl UserCommunication {
    pub fn new(comm_id: u32, user_id: u32, communication: CommData) -> Self {
        Self { comm_id, user_id, communication }
    }

    pub fn user_id(&self) -> u32 {
        self.user_id
    }

    pub fn communication(&self) -> &CommData {
        &self.communication
    }
    pub fn comm_id(&self) -> u32 {
        self.comm_id
    }
}