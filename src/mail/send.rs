use crate::mail::Email;
use failure::Error;
use rusoto_ses::Destination;
use rusoto_ses::SendEmailRequest;
use rusoto_ses::Ses;
use rusoto_ses::SesClient;
use std::future::Future;
use std::pin::Pin;

pub trait EmailSender: Clone + Default + Send + Sync + Unpin + 'static {
    fn send_email(&self, email: Email) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;
}

#[derive(Clone)]
pub struct SesSender {
    pub client: SesClient,
}

impl Default for SesSender {
    fn default() -> Self {
        SesSender {
            client: SesClient::new(Default::default()),
        }
    }
}

impl Into<SendEmailRequest> for Email {
    fn into(self) -> SendEmailRequest {
        let destination = Destination {
            to_addresses: Some(self.to),
            ..Default::default()
        };
        let message = self.message.into();
        SendEmailRequest {
            destination,
            message,
            source: self.from,
            ..Default::default()
        }
    }
}

const SES_TO_CHUNK_SIZE: usize = 50;

impl EmailSender for SesSender {
    fn send_email(
        &self,
        mut email: Email,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> {
        let client = self.client.clone();
        Box::pin(async move {
            while email.to.len() > SES_TO_CHUNK_SIZE {
                let to = email.to.split_off(SES_TO_CHUNK_SIZE);
                let part_email = Email {
                    to,
                    from: email.from.clone(),
                    message: email.message.clone(),
                };

                client.send_email(part_email.into()).await.map(|_| ())?;
            }
            client.send_email(email.into()).await.map(|_| ())?;
            Ok(())
        })
    }
}
