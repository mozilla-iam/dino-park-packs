use crate::db::model::Group;
use crate::db::model::GroupsList;
use crate::db::types::*;
use crate::error::PacksError;
use crate::utils::maybe_to_utc;
use crate::utils::to_utc;
use crate::utils::valid_group_name;
use chrono::NaiveDateTime;
use dino_park_trust::Trust;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

const DESCRIPTION_MAX_LEN: usize = 1024;

#[derive(Clone, Debug)]
pub struct NewPendingRequest {
    pub group_id: i32,
    pub group_name: String,
    pub count: usize,
}

#[derive(Deserialize)]
pub enum SortGroupsBy {
    MemberCountAsc,
    MemberCountDesc,
    NameAsc,
    NameDesc,
}

impl Default for SortGroupsBy {
    fn default() -> Self {
        Self::MemberCountDesc
    }
}

#[derive(Deserialize)]
pub enum SortMembersBy {
    None,
    RoleAsc,
    RoleDesc,
    ExpirationAsc,
    ExpirationDesc,
}

impl Default for SortMembersBy {
    fn default() -> Self {
        Self::None
    }
}

pub struct MembersQueryOptions {
    pub query: Option<String>,
    pub roles: Vec<RoleType>,
    pub limit: i64,
    pub offset: Option<i64>,
    pub order: SortMembersBy,
}

impl Default for MembersQueryOptions {
    fn default() -> Self {
        MembersQueryOptions {
            query: None,
            roles: vec![RoleType::Admin, RoleType::Curator, RoleType::Member],
            limit: 20,
            offset: None,
            order: SortMembersBy::RoleAsc,
        }
    }
}

#[derive(Deserialize)]
pub struct GroupUpdate {
    pub description: Option<String>,
    #[serde(default, rename = "type")]
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

    pub fn checked(self) -> Result<Self, PacksError> {
        if self
            .description
            .as_ref()
            .map(|d| d.len() > DESCRIPTION_MAX_LEN)
            .unwrap_or_default()
        {
            return Err(PacksError::InvalidGroupData);
        }
        if self
            .trust
            .as_ref()
            .map(|t| t < &TrustType::Authenticated)
            .unwrap_or_default()
        {
            return Err(PacksError::InvalidGroupData);
        }
        Ok(self)
    }
}

#[derive(Deserialize)]
pub struct NewGroup {
    pub name: String,
    pub description: String,
    #[serde(default, rename = "type")]
    pub typ: GroupType,
    #[serde(default)]
    pub capabilities: Vec<CapabilityType>,
    #[serde(default = "TrustType::ndaed")]
    pub trust: TrustType,
    #[serde(default)]
    pub group_expiration: Option<i32>,
}

impl NewGroup {
    pub fn checked(self) -> Result<Self, PacksError> {
        if self.description.len() > DESCRIPTION_MAX_LEN || self.trust < TrustType::Authenticated {
            return Err(PacksError::InvalidGroupName);
        }
        if !valid_group_name(&self.name) {
            return Err(PacksError::InvalidGroupName);
        }
        Ok(self)
    }
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
    #[serde(serialize_with = "maybe_to_utc")]
    pub invitation_expiration: Option<NaiveDateTime>,
    pub group_expiration: Option<i32>,
    pub group_name: String,
    pub terms: bool,
    pub added_by: DisplayHost,
}

#[derive(Serialize)]
pub struct DisplayInvitationForUser {
    pub user_uuid: Uuid,
    #[serde(serialize_with = "maybe_to_utc")]
    pub invitation_expiration: Option<NaiveDateTime>,
    pub group_expiration: Option<i32>,
    pub group_name: String,
    pub terms: bool,
    pub added_by: DisplayHost,
}

#[derive(Serialize, Queryable)]
pub struct DisplayRequestForUser {
    pub user_uuid: Uuid,
    #[serde(serialize_with = "to_utc")]
    pub created: NaiveDateTime,
    #[serde(serialize_with = "maybe_to_utc")]
    pub request_expiration: Option<NaiveDateTime>,
    pub group_name: String,
    pub terms: bool,
}

#[derive(Serialize, Queryable)]
pub struct DisplayRequest {
    pub user_uuid: Uuid,
    pub picture: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub is_staff: bool,
    #[serde(serialize_with = "to_utc")]
    pub created: NaiveDateTime,
    #[serde(serialize_with = "maybe_to_utc")]
    pub request_expiration: Option<NaiveDateTime>,
    pub group_name: String,
    pub terms: bool,
}

#[derive(Queryable)]
pub struct InvitationAndHostForUser {
    pub user_uuid: Uuid,
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

impl From<InvitationAndHostForUser> for DisplayInvitationForUser {
    fn from(m: InvitationAndHostForUser) -> Self {
        DisplayInvitationForUser {
            user_uuid: m.user_uuid,
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

#[derive(Queryable)]
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
    #[serde(serialize_with = "maybe_to_utc")]
    pub since: Option<NaiveDateTime>,
    #[serde(serialize_with = "maybe_to_utc")]
    pub expiration: Option<NaiveDateTime>,
    pub role: RoleType,
    pub added_by: Option<DisplayHost>,
}

#[derive(Serialize)]
pub struct DisplayMembershipAndHost {
    pub user_uuid: Uuid,
    #[serde(serialize_with = "maybe_to_utc")]
    pub since: Option<NaiveDateTime>,
    #[serde(serialize_with = "maybe_to_utc")]
    pub expiration: Option<NaiveDateTime>,
    pub role: RoleType,
    pub added_by: Option<DisplayHost>,
}

#[derive(Queryable)]
pub struct Member {
    pub user_uuid: Uuid,
    pub picture: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub is_staff: bool,
    pub role: RoleType,
    pub since: NaiveDateTime,
}

#[derive(Queryable)]
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

#[derive(Queryable)]
pub struct MembershipAndHost {
    pub user_uuid: Uuid,
    pub since: NaiveDateTime,
    pub expiration: Option<NaiveDateTime>,
    pub role: RoleType,
    pub host_uuid: Uuid,
    pub host_first_name: Option<String>,
    pub host_last_name: Option<String>,
    pub host_username: Option<String>,
    pub host_email: Option<String>,
}

impl DisplayMemberAndHost {
    pub fn from_with_scope(m: Member, scope: &Trust) -> Self {
        let since = if scope >= &Trust::Authenticated {
            Some(m.since)
        } else {
            None
        };
        DisplayMemberAndHost {
            user_uuid: m.user_uuid,
            picture: m.picture,
            first_name: m.first_name,
            last_name: m.last_name,
            username: m.username,
            email: m.email,
            is_staff: m.is_staff,
            since,
            expiration: None,
            role: m.role,
            added_by: None,
        }
    }
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
            since: Some(m.since),
            expiration: m.expiration,
            role: m.role,
            added_by: Some(DisplayHost {
                user_uuid: m.host_uuid,
                first_name: m.host_first_name,
                last_name: m.host_last_name,
                username: m.host_username,
                email: m.host_email,
            }),
        }
    }
}

impl From<MembershipAndHost> for DisplayMembershipAndHost {
    fn from(m: MembershipAndHost) -> Self {
        DisplayMembershipAndHost {
            user_uuid: m.user_uuid,
            since: Some(m.since),
            expiration: m.expiration,
            role: m.role,
            added_by: Some(DisplayHost {
                user_uuid: m.host_uuid,
                first_name: m.host_first_name,
                last_name: m.host_last_name,
                username: m.host_username,
                email: m.host_email,
            }),
        }
    }
}

#[derive(Serialize)]
pub struct PaginatedDisplayMembersAndHost {
    pub members: Vec<DisplayMemberAndHost>,
    pub next: Option<i64>,
}

#[derive(Serialize)]
pub struct PaginatedGroupsLists {
    pub groups: Vec<GroupsList>,
    pub next: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct InvitationEmail {
    pub body: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_group_update_log_comment() {
        let group_update = GroupUpdate {
            description: Some("something".into()),
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
