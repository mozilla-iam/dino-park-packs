use crate::db::model::Group;
use crate::db::types::*;
use chrono::NaiveDateTime;
use serde_derive::Serialize;
use uuid::Uuid;

pub struct NewGroup {
    pub name: String,
    pub description: String,
    pub typ: GroupType,
    pub trust: TrustType,
    pub expiration: Option<i32>,
}

pub struct GroupWithTermsFlag {
    pub group: Group,
    pub terms: bool,
}

#[derive(Serialize)]
pub struct DisplayInvitation {
    pub uuid: Uuid,
    pub picture: Option<String>,
    pub name: Option<String>,
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
    pub name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub is_staff: bool,
    pub invitation_expiration: Option<NaiveDateTime>,
    pub group_expiration: Option<i32>,
    pub group_name: String,
    pub terms: bool,
    pub host_uuid: Uuid,
    pub host_name: Option<String>,
    pub host_username: String,
    pub host_email: Option<String>,
}

impl From<InvitationAndHost> for DisplayInvitation {
    fn from(m: InvitationAndHost) -> Self {
        DisplayInvitation {
            uuid: m.user_uuid,
            picture: m.picture,
            name: m.name,
            username: m.username,
            email: m.email,
            is_staff: m.is_staff,
            invitation_expiration: m.invitation_expiration,
            group_expiration: m.group_expiration,
            group_name: m.group_name,
            terms: m.terms,
            added_by: DisplayHost {
                uuid: m.host_uuid,
                name: m.host_name,
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
    pub uuid: Uuid,
    pub name: Option<String>,
    pub username: String,
    pub email: Option<String>,
}

#[derive(Serialize)]
pub struct DisplayMemberAndHost {
    pub uuid: Uuid,
    pub picture: Option<String>,
    pub name: Option<String>,
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
    pub name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub is_staff: bool,
    pub since: NaiveDateTime,
    pub expiration: Option<NaiveDateTime>,
    pub role: RoleType,
    pub host_uuid: Uuid,
    pub host_name: Option<String>,
    pub host_username: String,
    pub host_email: Option<String>,
}

impl From<MemberAndHost> for DisplayMemberAndHost {
    fn from(m: MemberAndHost) -> Self {
        DisplayMemberAndHost {
            uuid: m.user_uuid,
            picture: m.picture,
            name: m.name,
            username: m.username,
            email: m.email,
            is_staff: m.is_staff,
            since: m.since,
            expiration: m.expiration,
            role: m.role,
            host: DisplayHost {
                uuid: m.host_uuid,
                name: m.host_name,
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
