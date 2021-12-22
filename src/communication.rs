use crate::communication::email::EmailCommunicator;

pub mod email;

pub trait CommunicationMethod<T> {

    fn send_report_to(&self, user_id: u32, user_comm_id: &T, reason: &str);

}

pub enum CommunicationMethods {
    Email(EmailCommunicator),
}