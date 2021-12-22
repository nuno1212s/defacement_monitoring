use crate::communication::CommunicationMethod;

pub struct EmailCommunicator {}

impl CommunicationMethod for EmailCommunicator {
    fn send_report_to(&self, user_id: u32, user_comm_id: &&str, reason: &str) {
        todo!()
    }
}