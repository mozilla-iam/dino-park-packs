use crate::db::model::Group;
use crate::db::types::GroupType;
use crate::utils::to_utc;
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Serialize)]
pub struct DisplayGroup {
    pub name: String,
    pub description: String,
    pub typ: GroupType,
    pub expiration: Option<i32>,
    #[serde(serialize_with = "to_utc")]
    pub created: NaiveDateTime,
}

impl From<Group> for DisplayGroup {
    fn from(g: Group) -> Self {
        DisplayGroup {
            name: g.name,
            description: g.description,
            typ: g.typ,
            expiration: g.group_expiration,
            created: g.created,
        }
    }
}

#[derive(Serialize)]
pub struct GroupInfo {
    pub name: String,
    pub description: String,
    pub typ: GroupType,
    pub expiration: Option<i32>,
    #[serde(serialize_with = "to_utc")]
    pub created: NaiveDateTime,
    pub terms: bool,
}

#[derive(Serialize)]
pub struct DisplayGroupDetails {
    pub super_user: bool,
    pub curator: bool,
    pub member: bool,
    pub group: GroupInfo,
    pub member_count: i64,
    pub invitation_count: Option<i64>,
    pub renewal_count: Option<i64>,
    pub request_count: Option<i64>,
}
