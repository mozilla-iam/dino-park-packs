use crate::mail::Message;

fn invitation(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            r#"[{domain}] You have been invited to join the "{group_name}" group"#,
            group_name = group_name,
            domain = domain
        ),
        body: format!(
            "
        Dear Mozillian,
        you've been invited to join the access group '{group_name}'.
        Please visit https://{domain}/a/ to accept the invitation.

        Cheers,
        The Mozilla IAM Team
        ",
            group_name = group_name,
            domain = domain
        ),
    }
}

#[derive(Clone)]
pub struct TemplateManager {
    domain: String,
}

impl TemplateManager {
    pub fn new(domain: String) -> Self {
        TemplateManager { domain }
    }

    pub fn render(&self, t: &Template) -> Message {
        match t {
            Template::Invitation(ref group_name) => invitation(group_name, &self.domain),
        }
    }
}

pub enum Template {
    Invitation(String),
}
