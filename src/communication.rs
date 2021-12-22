use crate::communication::email::EmailCommunicator;
use crate::databases::User;

pub mod email;

pub trait CommunicationMethod {
    fn send_report_to(&self, user: &UserCommunication, reason: &str);
}

pub struct UserCommunication {
    comm_id: u32,
    user_id: u32,
    communication: CommData,
}

pub enum CommData {
    Email(String)
}

impl UserCommunication {
    pub fn new(comm_id: u32, user_id: u32, communication: UserCommData) -> Self {
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