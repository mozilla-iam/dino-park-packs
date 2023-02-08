pub mod manager;
pub mod send;
pub mod templates;

use rusoto_ses::Body;
use rusoto_ses::Content;
use serde::Deserialize;

#[derive(Clone, Default, Deserialize)]
pub struct Message {
    pub subject: String,
    pub body: String,
}

impl From<Message> for rusoto_ses::Message {
    fn from(m: Message) -> rusoto_ses::Message {
        rusoto_ses::Message {
            subject: Content {
                data: m.subject,
                ..Default::default()
            },
            body: Body {
                text: Some(Content {
                    data: m.body,
                    charset: Some("UTF-8".to_owned()),
                }),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Default)]
pub struct Email {
    pub to: Option<String>,
    pub bcc: Option<Vec<String>>,
    pub from: String,
    pub message: Message,
}

impl Email {
    pub fn with(to: String, domain: &str, message: Message) -> Self {
        Email {
            to: Some(to),
            bcc: None,
            from: format!("no-reply@{domain}"),
            message,
        }
    }
    pub fn with_many(bcc: Vec<String>, domain: &str, message: Message) -> Self {
        Email {
            to: None,
            bcc: Some(bcc),
            from: format!("no-reply@{domain}"),
            message,
        }
    }
}
