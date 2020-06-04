pub mod manager;
pub mod send;
pub mod templates;

use rusoto_ses::Body;
use rusoto_ses::Content;

#[derive(Clone)]
pub struct Message {
    pub subject: String,
    pub body: String,
}

impl Into<rusoto_ses::Message> for Message {
    fn into(self) -> rusoto_ses::Message {
        rusoto_ses::Message {
            subject: Content {
                data: self.subject,
                ..Default::default()
            },
            body: Body {
                text: Some(Content {
                    data: self.body,
                    charset: Some("UTF-8".to_owned()),
                }),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone)]
pub struct Email {
    pub to: Vec<String>,
    pub from: String,
    pub message: Message,
}

impl Email {
    pub fn with(to: String, message: Message) -> Self {
        Email::with_many(vec![to], message)
    }
    pub fn with_many(to: Vec<String>, message: Message) -> Self {
        Email {
            to,
            from: "no-reply@dinopark.k8s.dev.sso.allizom.org".to_owned(),
            message,
        }
    }
}
