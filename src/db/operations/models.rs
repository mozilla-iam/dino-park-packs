use crate::db::model::Group;
use crate::db::types::*;
use chrono::NaiveDateTime;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GroupUpdate {
    pub description: Option<String>,
    pub typ: Option<GroupType>,
    pub capabilities: Option<Vec<CapabilityType>>,
    pub trust: Option<TrustType>,
    #[allow(clippy::option_option)]
    pub group_expiration: Option<Option<i32>>,
}

impl GroupUpdate {
    pub fn log_comment(&self) -> String {
        [
            self.description.as_ref().map(|_| "description"),
            self.typ.as_ref().map(|_| "typ"),
            self.capabilities.as_ref().map(|_| "capabilities"),
            self.trust.as_ref().map(|_| "trust"),
            self.group_expiration.as_ref().map(|_| "expiration"),
        ]
        .iter()
        .filter_map(|s| *s)
        .map(String::from)
        .collect::<Vec<String>>()
        .join(", ")
    }
}

#[derive(Deserialize)]
pub struct NewGroup {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub typ: GroupType,
    #[serde(default)]
    pub capabilities: Vec<CapabilityType>,
    #[serde(default = "TrustType::ndaed")]
    pub trust: TrustType,
    #[serde(default)]
    pub group_expiration: Option<i32>,
}

pub struct GroupWithTermsFlag {
    pub group: Group,
    pub terms: bool,
}

#[derive(Serialize)]
pub struct DisplayInvitation {
    pub user_uuid: Uuid,
    pub picture: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub is_staff: bool,
    pub invitation_expiration: Option<NaiveDateTime>,
    pub group_expiration: Option<i32>,
    pub group_name: String,
    pub terms: bool,
    pub added_by: DisplayHost,
}

#[derive(Queryable, Serialize)]
pub struct InvitationAndHost {
    pub user_uuid: Uuid,
    pub picture: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub is_staff: bool,
    pub invitation_expiration: Option<NaiveDateTime>,
    pub group_expiration: Option<i32>,
    pub group_name: String,
    pub terms: bool,
    pub host_uuid: Uuid,
    pub host_first_name: Option<String>,
    pub host_last_name: Option<String>,
    pub host_username: Option<String>,
    pub host_email: Option<String>,
}

impl From<InvitationAndHost> for DisplayInvitation {
    fn from(m: InvitationAndHost) -> Self {
        DisplayInvitation {
            user_uuid: m.user_uuid,
            picture: m.picture,
            first_name: m.first_name,
            last_name: m.last_name,
            username: m.username,
            email: m.email,
            is_staff: m.is_staff,
            invitation_expiration: m.invitation_expiration,
            group_expiration: m.group_expiration,
            group_name: m.group_name,
            terms: m.terms,
            added_by: DisplayHost {
                user_uuid: m.host_uuid,
                first_name: m.host_first_name,
                last_name: m.host_last_name,
                username: m.host_username,
                email: m.host_email,
            },
        }
    }
}

#[derive(Queryable, Serialize)]
pub struct DisplayMember {
    pub user_uuid: Uuid,
    pub picture: Option<String>,
    pub name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub is_staff: bool,
    pub since: NaiveDateTime,
    pub expiration: Option<NaiveDateTime>,
    pub role: RoleType,
}

#[derive(Serialize)]
pub struct DisplayHost {
    pub user_uuid: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize)]
pub struct DisplayMemberAndHost {
    pub user_uuid: Uuid,
    pub picture: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub is_staff: bool,
    pub since: NaiveDateTime,
    pub expiration: Option<NaiveDateTime>,
    pub role: RoleType,
    pub host: DisplayHost,
}

#[derive(Queryable, Serialize)]
pub struct MemberAndHost {
    pub user_uuid: Uuid,
    pub picture: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub is_staff: bool,
    pub since: NaiveDateTime,
    pub expiration: Option<NaiveDateTime>,
    pub role: RoleType,
    pub host_uuid: Uuid,
    pub host_first_name: Option<String>,
    pub host_last_name: Option<String>,
    pub host_username: Option<String>,
    pub host_email: Option<String>,
}

impl From<MemberAndHost> for DisplayMemberAndHost {
    fn from(m: MemberAndHost) -> Self {
        DisplayMemberAndHost {
            user_uuid: m.user_uuid,
            picture: m.picture,
            first_name: m.first_name,
            last_name: m.last_name,
            username: m.username,
            email: m.email,
            is_staff: m.is_staff,
            since: m.since,
            expiration: m.expiration,
            role: m.role,
            host: DisplayHost {
                user_uuid: m.host_uuid,
                first_name: m.host_first_name,
                last_name: m.host_last_name,
                username: m.host_username,
                email: m.host_email,
            },
        }
    }
}

#[derive(Serialize)]
pub struct PaginatedDisplayMembers {
    pub members: Vec<DisplayMember>,
    pub next: Option<i64>,
}

#[derive(Serialize)]
pub struct PaginatedDisplayMembersAndHost {
    pub members: Vec<DisplayMemberAndHost>,
    pub next: Option<i64>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_group_update_log_comment() {
        let group_update = GroupUpdate {
            description: Some("someting".into()),
            typ: None,
            capabilities: Some(vec![]),
            trust: Some(TrustType::Public),
            group_expiration: Some(None),
        };
        assert_eq!(
            group_update.log_comment(),
            "description, capabilities, trust, expiration"
        );
        let group_update = GroupUpdate {
            description: None,
            typ: None,
            capabilities: None,
            trust: None,
            group_expiration: None,
        };
        assert_eq!(group_update.log_comment(), "");
    }
}
