use crate::mail::Message;

fn invitation(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] You have been invited to join the '{group_name}' group",
            group_name = group_name,
            domain = domain
        ),
        body: format!(
            "\
Dear Mozillian,
you've been invited to join the access group '{group_name}'.
Please visit https://{domain}/a/ to accept the invitation.

Cheers,
The Mozilla IAM Team",
            group_name = group_name,
            domain = domain
        ),
    }
}

fn reject_request(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] Your invitation request for the '{group_name}' group has been rejected",
            group_name = group_name,
            domain = domain
        ),
        body: format!(
            "\
Dear Mozillian,
your request to be invited to the '{group_name}' access group has been rejected.
Please make sure to read the group description at https://{domain}/a/{group_name}

Cheers,
The Mozilla IAM Team",
            group_name = group_name,
            domain = domain
        ),
    }
}

fn delete_invitation(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] Your invitation for the '{group_name}' group has been revoked",
            group_name = group_name,
            domain = domain
        ),
        body: format!(
            "\
Dear Mozillian,
your invitation to the '{group_name}' access group has been revoked.
Please make sure to read the group description at https://{domain}/a/{group_name}

Cheers,
The Mozilla IAM Team",
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
            Template::RejectRequest(ref group_name) => reject_request(group_name, &self.domain),
            Template::DeleteInvitation(ref group_name) => delete_invitation(group_name, &self.domain),
        }
    }
}

pub enum Template {
    Invitation(String),
    RejectRequest(String),
    DeleteInvitation(String),
}
