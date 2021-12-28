use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use toml::Value;

use crate::communication::{CommData, CommunicationMethod, UserCommunication};
use crate::databases::{TrackedPage, User};

pub struct EmailSMTPData {
    smtp_server: String,
    username: String,
    password: String,
    port: Option<i64>,
}

pub struct EmailCommunicator {
    mailer: SmtpTransport,
}

impl EmailCommunicator {
    pub fn new(config_file: &str) -> Self {
        let value = config_file.parse::<Value>().unwrap();

        let email_smtp = EmailSMTPData::new(String::from(value["smtp_server"].as_str().unwrap()),
                                            String::from(value["username"].as_str().unwrap()),
                                            String::from(value["password"].as_str().unwrap()),
                                            value["port"].as_integer());

        let credentials = Credentials::new(String::from(email_smtp.username()),
                                           String::from(email_smtp.password()));

        let mailer = SmtpTransport::relay(email_smtp.smtp_server())
            .unwrap()
            .credentials(credentials)
            .build();

        Self {
            mailer
        }
    }

    fn send_mail_to(&self, from: &str, destination: &str, subject: &str, body: &str) -> Result<String, String> {
        let msg = Message::builder()
            .from(from.parse().unwrap())
            .to(destination.parse().unwrap())
            .subject(subject)
            .body(String::from(body))
            .unwrap();

        let result = self.mailer.send(&msg);

        match result {
            Ok(_) => {
                Ok(String::from("Email sent successfully"))
            }
            Err(e) => {
                Err(e.to_string())
            }
        }
    }
}

impl EmailSMTPData {
    pub fn new(smtp_server: String, username: String, password: String, port: Option<i64>) -> Self {
        EmailSMTPData { smtp_server, username, password, port }
    }

    pub fn smtp_server(&self) -> &str {
        &self.smtp_server
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn password(&self) -> &str {
        &self.password
    }
    pub fn port(&self) -> Option<i64> {
        self.port
    }
}

impl CommunicationMethod for EmailCommunicator {
    fn matches(&self, comm: &CommData) -> bool {
        return match comm { CommData::Email(_) => { true } }
    }

    fn send_report_to(&self, user: &User, comm_method: &UserCommunication, tracked_page: &TrackedPage) -> Result<String, String> {
        return match comm_method.communication() {
            CommData::Email(email) => {
                return self.send_mail_to("Nuno Neto <nunonuninho2@gmail.com>",
                                         format!("{} <{}>", user.user(), email).as_str(),
                                         format!("Defacement detected in tracked page {} with ID {}",
                                                 tracked_page.page_url(), tracked_page.page_id()).as_str(),
                                         "",
                );
            }
            _ => {
                Err(String::from("There is no email registered to that communication method."))
            }
        };
    }
}

#[cfg(test)]
mod email_tests {
    use crate::communication::CommunicationMethod;
    use crate::communication::email::EmailCommunicator;

    #[test]
    fn test_send_mail() {
        let config_file = include_str!("../../resources/email.toml");

        let communicator = EmailCommunicator::new(config_file);

        let result_str = communicator.send_mail_to(
            "Nuno Neto <nunonuninho2@gmail.com>",
            "Nuno Neto <nuno.neto.g@gmail.com>",
            "Test subject",
            "Test body",
        ).unwrap();
    }
}