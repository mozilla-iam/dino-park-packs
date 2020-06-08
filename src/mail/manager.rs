use crate::mail::send::EmailSender;
#[cfg(all(not(test), not(feature = "local")))]
use crate::mail::send::SesSender;
use crate::mail::templates::Template;
use crate::mail::templates::TemplateManager;
use crate::mail::Email;
#[cfg(all(not(test), not(feature = "local")))]
use crate::settings::Settings;
use actix_rt::Arbiter;
#[cfg(all(not(test), not(feature = "local")))]
use lazy_static::lazy_static;
use log::error;

#[cfg(all(not(test), not(feature = "local")))]
lazy_static! {
    static ref MAIL_MAN: MailMan<SesSender> = {
        let s = Settings::new().expect("invalid settings");
        MailMan::<SesSender>::new(s.packs.domain, s.packs.catcher)
    };
}

#[cfg(all(not(test), not(feature = "local")))]
pub fn send_email(to: String, t: &Template) {
    let message = MAIL_MAN.template_man.render(t);
    MAIL_MAN.send(Email::with(to, message));
}

#[cfg(all(not(test), not(feature = "local")))]
pub fn send_emails(to: Vec<String>, t: &Template) {
    let message = MAIL_MAN.template_man.render(t);
    MAIL_MAN.send(Email::with_many(to, message));
}

#[cfg(any(test, feature = "local"))]
pub fn send_email(_: String, _: &Template) {}

#[cfg(any(test, feature = "local"))]
pub fn send_emails(_: Vec<String>, _: &Template) {}

#[derive(Clone)]
pub struct MailMan<T: EmailSender> {
    pub arbiter: Arbiter,
    pub sender: T,
    pub template_man: TemplateManager,
    pub catcher: Option<String>,
}

impl<T: EmailSender> MailMan<T> {
    pub fn new(domain: String, catcher: Option<String>) -> Self {
        MailMan {
            arbiter: Arbiter::default(),
            sender: T::default(),
            template_man: TemplateManager::new(domain),
            catcher,
        }
    }
}

impl<T: EmailSender> MailMan<T> {
    pub fn send(&self, mut e: Email) {
        if let Some(ref catcher) = self.catcher {
            if let Some(to) = e.to {
                e.message.body = format!("[to: caught for {}]\n\n{}", to, e.message.body);
                e.to = Some(catcher.to_owned());
            };
            if let Some(bcc) = e.bcc {
                e.message.body =
                    format!("[bcc: caught for {}]\n\n{}", bcc.join(", "), e.message.body);
                e.to = Some(catcher.to_owned());
                e.bcc = None;
            };
        }
        let s = self.sender.clone();
        let f = Box::pin(async move {
            if let Err(e) = s.send_email(e).await {
                error!("Error sending email: {}", e);
            }
        });
        self.arbiter.send(f)
    }
}
