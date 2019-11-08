use crate::db::db::Pool;
use crate::db::group::*;
use crate::db::schema;
use crate::db::schema::groups::dsl::*;
use crate::db::types::*;
use crate::user::User;
use diesel::prelude::*;
use failure::Error;
use log::info;
use uuid::Uuid;

pub fn get_group(pool: &Pool, group_name: &str) -> Result<Group, Error> {
    let connection = pool.get()?;
    groups
        .filter(name.eq(group_name))
        .first::<Group>(&connection)
        .map_err(Into::into)
}

pub fn add_new_group(
    pool: &Pool,
    group_name: String,
    group_description: String,
    creator: User,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = InsertGroup {
        name: group_name,
        path: String::from("/access_information/mozillians/"),
        description: group_description,
        capabilities: vec![],
        typ: GroupType::Closed,
        group_expiration: None,
    };
    let new_group = diesel::insert_into(schema::groups::table)
        .values(&group)
        .on_conflict_do_nothing()
        .get_result::<Group>(&connection)?;
    info!("Group: {:#?}", new_group);
    let member = InsertRole {
        group_id: new_group.id,
        typ: None,
        name: String::from("curator"),
        permissions: vec![],
    };
    let group_member = diesel::insert_into(schema::roles::table)
        .values(member)
        .get_result::<Role>(&connection)?;
    info!("Role: {:#?}", group_member);
    let admin = InsertRole {
        group_id: new_group.id,
        typ: Some(RoleType::Admin),
        name: String::from("admin"),
        permissions: vec![],
    };
    let group_admin = diesel::insert_into(schema::roles::table)
        .values(admin)
        .get_result::<Role>(&*connection)?;
    info!("Role: {:#?}", group_admin);
    let creator_membership = InsertMembership {
        group_id: new_group.id,
        user_uuid: creator.user_uuid.clone(),
        role_id: Some(group_admin.id),
        added_by: Uuid::nil(),
    };
    let group_creator = diesel::insert_into(schema::memberships::table)
        .values(creator_membership)
        .get_result::<Membership>(&*connection)?;
    info!("Membership: {:#?}", group_creator);
    Ok(())
}
