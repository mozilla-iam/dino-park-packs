use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::schema;
use crate::db::types::*;
use diesel::prelude::*;
use failure::Error;
use uuid::Uuid;

const ROLE_MEMBER: &str = "member";

pub fn add_member_role(pool: &Pool, group_id: i32) -> Result<Role, Error> {
    let connection = pool.get()?;
    let admin = InsertRole {
        group_id,
        typ: RoleType::Member,
        name: ROLE_MEMBER.to_owned(),
        permissions: vec![],
    };
    diesel::insert_into(schema::roles::table)
        .values(admin)
        .get_result(&*connection)
        .map_err(Into::into)
}

pub fn role_for(pool: &Pool, user_uuid: &Uuid, group_name: &str) -> Result<Role, Error> {
    let connection = pool.get()?;
    schema::memberships::table
        .filter(schema::memberships::user_uuid.eq(user_uuid))
        .inner_join(schema::groups::table)
        .filter(schema::groups::name.eq(group_name))
        .inner_join(schema::roles::table)
        .get_result::<(Membership, Group, Role)>(&connection)
        .map(|(_, _, r)| r)
        .map_err(Into::into)
}

pub fn member_role(pool: &Pool, group_name: &str) -> Result<Role, Error> {
    let connection = pool.get()?;
    schema::roles::table
        .inner_join(schema::groups::table)
        .filter(schema::groups::name.eq(group_name))
        .filter(schema::roles::typ.eq(RoleType::Member))
        .get_result::<(Role, Group)>(&connection)
        .map(|(r, _)| r)
        .map_err(Into::into)
}

pub fn remove_from_group(pool: &Pool, user_uuid: &Uuid, group_name: &str) -> Result<(), Error> {
    let connection = pool.get()?;
    diesel::delete(schema::memberships::table)
        .filter(schema::memberships::user_uuid.eq(user_uuid))
        .execute(&connection)?;
    Ok(())
}
