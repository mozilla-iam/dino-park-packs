use crate::db::db::Pool;
use crate::db::group::*;
use crate::db::schema;
use crate::db::schema::groups::dsl::*;
use crate::types::*;
use crate::user::User;
use diesel::prelude::*;
use failure::Error;
use log::info;

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
    };
    let new_group = diesel::insert_into(schema::groups::table)
        .values(&group)
        .on_conflict_do_nothing()
        .get_result::<Group>(&connection)
        .expect("Error saving group");
    info!("Group: {:#?}", new_group);
    let member = InsertRole {
        group_id: new_group.id,
        typ: None,
        name: String::from("curator"),
        permissions: vec![],
    };
    let group_member = diesel::insert_into(schema::roles::table)
        .values(member)
        .get_result::<Role>(&connection)
        .expect("Error saving roles");
    info!("Role: {:#?}", group_member);
    let admin = InsertRole {
        group_id: new_group.id,
        typ: Some(RoleType::Admin),
        name: String::from("admin"),
        permissions: vec![],
    };
    let group_admin = diesel::insert_into(schema::roles::table)
        .values(admin)
        .get_result::<Role>(&*connection)
        .expect("Error saving roles");
    info!("Role: {:#?}", group_admin);
    let creator_membership = InsertMembership {
        group_id: new_group.id,
        user_uuid: creator.user_uuid.clone(),
        role_id: Some(group_admin.id),
        added_by: None,
    };
    let group_creator = diesel::insert_into(schema::memberships::table)
        .values(creator_membership)
        .get_result::<Membership>(&*connection)
        .expect("Error saving roles");
    info!("Membership: {:#?}", group_creator);
    Ok(())
}

pub fn add_user_to_group(
    connection: &PgConnection,
    group_name: String,
    creator: User,
    user: User,
) -> Result<(), Error> {
    let group = groups
        .filter(name.eq(&group_name))
        .first::<Group>(&*connection)
        .expect("Error loading groups");
    let membership = InsertMembership {
        user_uuid: user.user_uuid,
        group_id: group.id.clone(),
        role_id: None,
        added_by: None,
    };
    let rows_inserted = diesel::insert_into(schema::memberships::table)
        .values(&membership)
        .on_conflict_do_nothing()
        .execute(&*connection)
        .expect("Error saving group");
    info!("Inserted {} rows", rows_inserted);

    Ok(())
}
