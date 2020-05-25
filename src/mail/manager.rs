use crate::error::PacksError;
use crate::mail::send::EmailSender;
#[cfg(all(not(test), not(feature = "local")))]
use crate::mail::send::SesSender;
use crate::mail::templates::Template;
use crate::mail::templates::TemplateManager;
use crate::mail::Email;
#[cfg(all(not(test), not(feature = "local")))]
use crate::settings::Settings;
use actix_rt::Arbiter;
use cis_profile::schema::Profile;
#[cfg(all(not(test), not(feature = "local")))]
use lazy_static::lazy_static;
use log::error;

#[cfg(all(not(test), not(feature = "local")))]
lazy_static! {
    static ref MAIL_MAN: MailMan<SesSender> = {
        let s = Settings::new().expect("invalid settings");
        MailMan::<SesSender>::new(s.packs.domain)
    };
}

#[cfg(all(not(test), not(feature = "local")))]
pub fn send_email(p: &Profile, t: &Template) -> Result<(), PacksError> {
    let message = MAIL_MAN.template_man.render(t);
    MAIL_MAN.send(Email::from_with(p, message)?);
    Ok(())
}
#[cfg(any(test, feature = "local"))]
pub fn send_email(_: &Profile, _: &Template) -> Result<(), PacksError> {
    Ok(())
}

#[derive(Clone)]
pub struct MailMan<T: EmailSender> {
    pub arbiter: Arbiter,
    pub sender: T,
    pub template_man: TemplateManager,
}

impl<T: EmailSender> MailMan<T> {
    pub fn new(domain: String) -> Self {
        MailMan {
            arbiter: Arbiter::default(),
            sender: T::default(),
            template_man: TemplateManager::new(domain),
        }
    }
}

impl<T: EmailSender> MailMan<T> {
    pub fn send(&self, e: Email) {
        let s = self.sender.clone();
        let f = Box::pin(async move {
            if let Err(e) = s.send_email(e).await {
                error!("Error sending email: {}", e);
            }
        });
        self.arbiter.send(f)
    }
}
