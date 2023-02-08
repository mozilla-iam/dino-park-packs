use crate::mail::Message;

fn invitation(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!("[{domain}] You have been invited to join the '{group_name}' group"),
        body: format!(
            "\
Dear Mozillian,
you've been invited to join the access group '{group_name}'.
Please visit https://{domain}/a/{group_name} to accept the invitation.

Cheers,
The Mozilla IAM Team"
        ),
    }
}

fn custom_invitation(group_name: &str, domain: &str, copy: &str) -> Message {
    Message {
        subject: format!("[{domain}] You have been invited to join the '{group_name}' group"),
        body: format!(
            "\
Dear Mozillian,
you've been invited to join the access group '{group_name}'.
The message from the curator is:

{copy}

Please visit https://{domain}/a/{group_name} to accept the invitation.

Cheers,
The Mozilla IAM Team"
        ),
    }
}

fn reject_request(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] Your invitation request for the '{group_name}' group has been rejected"
        ),
        body: format!(
            "\
Dear Mozillian,
your request to be invited to the '{group_name}' access group has been rejected.
Please make sure to read the group description at https://{domain}/a/{group_name}

Cheers,
The Mozilla IAM Team"
        ),
    }
}

fn delete_invitation(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] Your invitation for the '{group_name}' group has been revoked"
        ),
        body: format!(
            "\
Dear Mozillian,
your invitation to the '{group_name}' access group has been revoked.
Please make sure to read the group description at https://{domain}/a/{group_name}

Cheers,
The Mozilla IAM Team"
        ),
    }
}

fn demote_curator(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] Your membership for the '{group_name}' group has been revoked"
        ),
        body: format!(
            "\
Dear Mozillian,
your curator status for the '{group_name}' access group has been revoked.
You are still a member and can see your status here: https://{domain}/a/{group_name}

Cheers,
The Mozilla IAM Team"
        ),
    }
}

fn delete_member(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] Your membership for the '{group_name}' group has been revoked"
        ),
        body: format!(
            "\
Dear Mozillian,
your membership to the '{group_name}' access group has been revoked.
If you have any questions make sure to read the group description at https://{domain}/a/{group_name}

Cheers,
The Mozilla IAM Team"
        ),
    }
}

fn first_host_expiration(group_name: &str, user: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] {user}'s membership of the '{group_name}' group is about to expire"
        ),
        body: format!(
            "\
Dear Curator,
{user}'s membership of the '{group_name}' group will expire in 14 days.

Please visit https://{domain}/a/{group_name}/edit?section=members to renew the \
membership if applicable.

Or visit {user}'s profile first: https://{domain}/p/{user}

Cheers,
The Mozilla IAM Team"
        ),
    }
}

fn second_host_expiration(group_name: &str, user: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] {user}'s membership of the '{group_name}' group is about to expire"
        ),
        body: format!(
            "\
Dear Curator,
{user}'s membership of the '{group_name}' group will expire in 7 days.

Please visit https://{domain}/a/{group_name}/edit?section=members to renew the \
membership if applicable.

Or visit {user}'s profile first: https://{domain}/p/{user}

Cheers,
The Mozilla IAM Team"
        ),
    }
}

fn member_expiration(group_name: &str, domain: &str) -> Message {
    Message {
        subject: format!(
            "[{domain}] Your membership of the '{group_name}' group is about to expire"
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
The Mozilla IAM Team"
        ),
    }
}

fn pending_request(group_name: &str, count: usize, domain: &str) -> Message {
    let pending = match count {
        1 => String::from("is 1 pending request"),
        c => format!("are {c} pending requests"),
    };
    Message {
        subject: format!("[{domain}] There {pending} in the '{group_name}' group"),
        body: format!(
            "\
Dear Curator,
there are {pending} mozillians asking for invitation in the access group '{group_name}.
For further action please visit: https://{domain}/a/{group_name}/edit?section=invitations

Cheers,
The Mozilla IAM Team"
        ),
    }
}

fn group_deleted(group_name: &str, user: &str, domain: &str) -> Message {
    Message {
        subject: format!("[{domain}] The '{group_name}' group has been deleted"),
        body: format!(
            "\
Dear Curator,
the '{group_name}' group has been deleted by https://{domain}/p/{user}

Cheers,
The Mozilla IAM Team"
        ),
    }
}

fn anonymous_member(domain: &str) -> Message {
    Message {
        subject: format!("[{domain}] mozillians.org decommissioning - PLEASE READ"),
        body: format!(
            "\
Dear Mozillian,

You are receiving this email because you are part of an access group 
(mozillians.org/en-US/groups/) and your profile needs attention.

As we prepare to decommission mozillians.org in a couple of weeks, we have
finalized moving access groups data from mozillians.org to {domain}.

How does this impact you?

If you want to keep the access provided by the groups you're a member of,
you will need to create an account on {domain}.
To do this, please follow these steps:

1. Go to {domain}
2. Create and account by clicking the Log in/Sign up button
3. When logging in, use the login method that you generally use to single sign
on*
4. Change your username to something to your liking
5. Change your *email address* field level visibility settings from 'private'
to 'NDA'd' so that group curators can see who you are when they need to renew
your membership**
6. (Optional) To further ensure curators can verify your identity consider
changing the *first_name*/*last_name* field level visibility settings from
private to 'NDA'd or take other adjustments like sharing a profile picture.

Pro tip: If you need to configure an additional profile on {domain} (because
you currently have multiple mozillians.org identities), avoid being auto logged
in by the system by logging out on sso.mozilla.com first.

*It is important to note that in mozillians.org you were able to have multiple
identities linked to your account. This will not be possible in {domain}
anymore.

If you currently use multiple identities within the mozilla ecosystem and
you're experiencing problems, contact us on the #iam Slack channel so that we
can manually check your account.

**By failing to do so, you take the risk of showing as 'Anonymous user'
to curators of the access groups you're part of, who will not extend your
membership when it's due to expire.

Thank you,
The Mozilla IAM Team"
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
            Template::AnonymousMember => anonymous_member(&self.domain),
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
    AnonymousMember,
}
