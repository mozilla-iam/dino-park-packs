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
Please visit https://{domain}/a/{group_name} to accept the invitation.

Cheers,
The Mozilla IAM Team",
            group_name = group_name,
            domain = domain
        ),
    }
}

fn custom_invitation(group_name: &str, domain: &str, copy: &str) -> Message {
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
The message from the curator is:

{copy}

Please visit https://{domain}/a/{group_name} to accept the invitation.

Cheers,
The Mozilla IAM Team",
            group_name = group_name,
            domain = domain,
            copy = copy
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

fn demote_curator(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] Your membership for the '{group_name}' group has been revoked",
            group_name = group_name,
            domain = domain
        ),
        body: format!(
            "\
Dear Mozillian,
your curator status for the '{group_name}' access group has been revoked.
You are still a member and can see your status here: https://{domain}/a/{group_name}

Cheers,
The Mozilla IAM Team",
            group_name = group_name,
            domain = domain
        ),
    }
}

fn delete_member(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] Your membership for the '{group_name}' group has been revoked",
            group_name = group_name,
            domain = domain
        ),
        body: format!(
            "\
Dear Mozillian,
your membership to the '{group_name}' access group has been revoked.
If you have any questions make sure to read the group description at https://{domain}/a/{group_name}

Cheers,
The Mozilla IAM Team",
            group_name = group_name,
            domain = domain
        ),
    }
}

fn first_host_expiration(group_name: &str, user: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] {user}'s membership of the '{group_name}' group is about to expire",
            group_name = group_name,
            user = user,
            domain = domain
        ),
        body: format!(
            "\
Dear Curator,
{user}'s membership of the '{group_name}' group will expire in 14 days.

Please visit https://{domain}/a/{group_name}//edit?section=members to renew the \
membership if applicable.

Or visit {user}'s profile first: https://{domain}/p/{user}

Cheers,
The Mozilla IAM Team",
            group_name = group_name,
            user = user,
            domain = domain
        ),
    }
}

fn second_host_expiration(group_name: &str, user: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] {user}'s membership of the '{group_name}' group is about to expire",
            group_name = group_name,
            user = user,
            domain = domain
        ),
        body: format!(
            "\
Dear Curator,
{user}'s membership of the '{group_name}' group will expire in 7 days.

Please visit https://{domain}/a/{group_name}/edit?section=members to renew the \
membership if applicable.

Or visit {user}'s profile first: https://{domain}/p/{user}

Cheers,
The Mozilla IAM Team",
            group_name = group_name,
            user = user,
            domain = domain
        ),
    }
}

fn member_expiration(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] Your membership of the '{group_name}' group is about to expire",
            group_name = group_name,
            domain = domain
        ),
        body: format!(
            "\
Dear Mozillian,
As per the terms of your membership to group '{group_name} your membership will expire in 7 days \
unless you are renewed by your group’s curators.

Your inviter has also been sent a notice for your renewal and will approve or reject your \
membership renewal in the next 7 days.

For more information visit the group page: https://{domain}/a/{group_name}

Cheers,
The Mozilla IAM Team",
            group_name = group_name,
            domain = domain
        ),
    }
}

fn pending_request(group_name: &str, count: usize, domain: &str) -> Message {
    let pending = match count {
        1 => String::from("is 1 pending request"),
        c => format!("are {} pending requests", c),
    };
    Message {
        subject: format!(
            "[{domain}] There {pending} in the '{group_name}' group",
            group_name = group_name,
            pending = pending,
            domain = domain
        ),
        body: format!(
            "\
Dear Curator,
there {pending} for invitation in the access group '{group_name}.
For further action please visit: https://{domain}/a/{group_name}/edit?section=invitations

Cheers,
The Mozilla IAM Team",
            group_name = group_name,
            pending = pending,
            domain = domain
        ),
    }
}

fn group_deleted(group_name: &str, user: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] The '{group_name}' group has been deleted",
            group_name = group_name,
            domain = domain
        ),
        body: format!(
            "\
Dear Curator,
the '{group_name}' group has been deleted by https://{domain}/p/{user}

Cheers,
The Mozilla IAM Team",
            group_name = group_name,
            user = user,
            domain = domain
        ),
    }
}

#[derive(Clone)]
pub struct TemplateManager {
    pub domain: String,
}

impl TemplateManager {
    pub fn new(domain: String) -> Self {
        TemplateManager { domain }
    }

    pub fn render(&self, t: &Template) -> Message {
        match t {
            Template::Invitation(ref group_name) => invitation(group_name, &self.domain),
            Template::CustomInvitation(ref group_name, copy) => {
                custom_invitation(group_name, &self.domain, copy)
            }
            Template::RejectRequest(ref group_name) => reject_request(group_name, &self.domain),
            Template::DeleteInvitation(ref group_name) => {
                delete_invitation(group_name, &self.domain)
            }
            Template::DemoteCurator(ref group_name) => demote_curator(group_name, &self.domain),
            Template::DeleteMember(ref group_name) => delete_member(group_name, &self.domain),
            Template::MemberExpiration(ref group_name) => {
                member_expiration(group_name, &self.domain)
            }
            Template::FirstHostExpiration(ref group_name, ref user) => {
                first_host_expiration(group_name, user, &self.domain)
            }
            Template::SecondHostExpiration(ref group_name, ref user) => {
                second_host_expiration(group_name, user, &self.domain)
            }
            Template::PendingRequest(ref group_name, count) => {
                pending_request(group_name, *count, &self.domain)
            }
            Template::GroupDeleted(ref group_name, ref user) => {
                group_deleted(group_name, user, &self.domain)
            }
        }
    }
}

pub enum Template {
    Invitation(String),
    CustomInvitation(String, String),
    RejectRequest(String),
    DeleteInvitation(String),
    DemoteCurator(String),
    DeleteMember(String),
    MemberExpiration(String),
    FirstHostExpiration(String, String),
    SecondHostExpiration(String, String),
    PendingRequest(String, usize),
    GroupDeleted(String, String),
}
