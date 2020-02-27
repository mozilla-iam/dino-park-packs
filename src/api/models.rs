use crate::db::operations::models::PaginatedDisplayMembersAndHost;
use crate::db::types::GroupType;
use chrono::NaiveDateTime;
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct GroupInfo {
    pub name: String,
    pub description: String,
    pub typ: GroupType,
    pub expiration: Option<i32>,
    pub created: NaiveDateTime,
    pub terms: bool,
}

#[derive(Serialize)]
pub struct DisplayGroupDetails {
    pub curator: bool,
    pub group: GroupInfo,
    pub members: PaginatedDisplayMembersAndHost,
    pub member_count: i64,
    pub invitation_count: Option<i64>,
    pub renewal_count: Option<i64>,
}
