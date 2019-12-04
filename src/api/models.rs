use crate::db::operations::models::PaginatedDisplayMembersAndHost;
use crate::db::types::GroupType;
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct GroupInfo {
    pub name: String,
    pub description: String,
    pub typ: GroupType,
}

#[derive(Serialize)]
pub struct DisplayGroupDetails {
    pub group: GroupInfo,
    pub members: PaginatedDisplayMembersAndHost,
    pub member_count: i64,
    pub invitation_count: i64,
    pub renewal_count: i64,
}
