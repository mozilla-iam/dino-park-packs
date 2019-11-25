use crate::db::schema::*;
use crate::db::types::*;
use chrono::NaiveDateTime;
use serde_derive::Serialize;
use uuid::Uuid;

#[derive(Identifiable, Queryable, PartialEq, Debug, Serialize)]
#[table_name = "groups"]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub description: String,
    pub capabilities: Vec<CapabilityType>,
    pub typ: GroupType,
    pub trust: TrustType,
    pub group_expiration: Option<i32>,
}

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[primary_key(group_id)]
#[table_name = "terms"]
pub struct Terms {
    pub group_id: i32,
    pub text: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Group)]
pub struct Role {
    pub id: i32,
    pub group_id: i32,
    pub typ: RoleType,
    pub name: String,
    pub permissions: Vec<PermissionType>,
}

#[derive(Queryable, Associations, PartialEq, Debug, Insertable, AsChangeset)]
pub struct Membership {
    pub user_uuid: Uuid,
    pub group_id: i32,
    pub role_id: i32,
    pub expiration: Option<NaiveDateTime>,
    pub added_by: Uuid,
    pub added_ts: NaiveDateTime,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug, Insertable, AsChangeset)]
#[belongs_to(Group)]
#[primary_key(group_id, user_uuid)]
pub struct Invitation {
    pub group_id: i32,
    pub user_uuid: Uuid,
    pub invitation_expiration: Option<NaiveDateTime>,
    pub group_expiration: Option<NaiveDateTime>,
    pub added_by: Uuid,
}

#[derive(Insertable)]
#[table_name = "groups"]
pub struct InsertGroup {
    pub name: String,
    pub path: String,
    pub description: String,
    pub capabilities: Vec<CapabilityType>,
    pub typ: GroupType,
    pub trust: TrustType,
    pub group_expiration: Option<i32>,
}

#[derive(Insertable)]
#[table_name = "terms"]
pub struct InsertTerm {
    pub text: String,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "memberships"]
pub struct InsertMembership {
    pub user_uuid: Uuid,
    pub group_id: i32,
    pub role_id: Option<i32>,
    pub expiration: Option<NaiveDateTime>,
    pub added_by: Uuid,
}

#[derive(Insertable)]
#[table_name = "roles"]
pub struct InsertRole {
    pub group_id: i32,
    pub typ: Option<RoleType>,
    pub name: String,
    pub permissions: Vec<PermissionType>,
}
