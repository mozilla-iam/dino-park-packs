use uuid::Uuid;
use crate::groups::schema::*;
use crate::types::*;
use std::time::SystemTime;

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name = "groups"]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub description: String,
    pub capabilities: Vec<CapabilityType>,
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

#[derive(Queryable, Associations, PartialEq, Debug)]
pub struct Membership {
    pub user_uuid: Uuid,
    pub group_id: i32,
    pub role_id: i32,
    pub expiration: Option<SystemTime>,
}

#[derive(Queryable, Associations, PartialEq, Debug)]
pub struct Invitation {
    pub id: i32,
    pub user_uuid: Uuid,
    pub group_id: i32,
    pub invitation_expiration: Option<SystemTime>,
    pub group_expiration: Option<SystemTime>,
}

#[derive(Insertable)]
#[table_name="groups"]
pub struct InsertGroup {
    pub name: String,
    pub path: String,
    pub description: String,
    pub capabilities: Vec<CapabilityType>,
}

#[derive(Insertable)]
#[table_name="memberships"]
pub struct InsertMembership {
    pub user_uuid: Uuid,
    pub group_id: i32,
    pub role_id: Option<i32>,
}

#[derive(Insertable)]
#[table_name="roles"]
pub struct InsertRole {
    pub group_id: i32,
    pub typ: Option<RoleType>,
    pub name: String,
    pub permissions: Vec<PermissionType>,
}

#[derive(Insertable)]
#[table_name="invitations"]
pub struct InsertInvitation {
    pub user_uuid: Uuid,
    pub group_id: i32,
    pub invitation_expiration: Option<SystemTime>,
    pub group_expiration: Option<SystemTime>,
}