pub mod manager;
pub mod send;
pub mod templates;

use crate::error::PacksError;
use cis_profile::schema::Profile;
use rusoto_ses::Body;
use rusoto_ses::Content;

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

pub struct Email {
    pub to: String,
    pub from: String,
    pub message: Message,
}

impl Email {
    pub fn from_with(p: &Profile, message: Message) -> Result<Self, PacksError> {
        if let Some(to) = p.primary_email.value.clone() {
            Ok(Email {
                to,
                from: "no-reply@dinopark.k8s.dev.sso.allizom.org".to_owned(),
                message,
            })
        } else {
            Err(PacksError::NoPrimaryEmail)
        }
    }
}
