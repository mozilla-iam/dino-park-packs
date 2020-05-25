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

impl EmailSender for SesSender {
    fn send_email(&self, email: Email) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>> {
        let destination = Destination {
            to_addresses: Some(vec![email.to]),
            ..Default::default()
        };
        let message = email.message.into();
        let req = SendEmailRequest {
            destination,
            message,
            source: email.from,
            ..Default::default()
        };

        let client = self.client.clone();
        Box::pin(async move { client.send_email(req).await.map(|_| ()).map_err(Into::into) })
    }
}
