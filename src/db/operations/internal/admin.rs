use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::schema;
use crate::db::types::*;
use diesel::prelude::*;
use failure::Error;
use uuid::Uuid;

const ROLE_ADMIN: &str = "admin";

pub fn add_admin_role(pool: &Pool, group_id: i32) -> Result<Role, Error> {
    let connection = pool.get()?;
    let admin = InsertRole {
        group_id,
        typ: Some(RoleType::Admin),
        name: ROLE_ADMIN.to_owned(),
        permissions: vec![],
    };
    diesel::insert_into(schema::roles::table)
        .values(admin)
        .get_result(&*connection)
        .map_err(Into::into)
}

pub fn get_admin_role(pool: &Pool, group_id: i32) -> Result<Role, Error> {
    let connection = pool.get()?;
    schema::roles::table
        .filter(schema::roles::group_id.eq(group_id))
        .filter(schema::roles::name.eq(ROLE_ADMIN))
        .filter(schema::roles::typ.eq(RoleType::Admin))
        .first(&connection)
        .map_err(Into::into)
}

pub fn add_admin(
    pool: &Pool,
    group_id: i32,
    user_uuid: Uuid,
    added_by: Uuid,
) -> Result<Membership, Error> {
    let role = get_admin_role(pool, group_id)?;
    let connection = pool.get()?;
    let admin_membership = InsertMembership {
        group_id,
        user_uuid,
        role_id: Some(role.id),
        expiration: None,
        added_by,
    };
    diesel::insert_into(schema::memberships::table)
        .values(&admin_membership)
        .on_conflict((
            schema::memberships::user_uuid,
            schema::memberships::group_id,
        ))
        .do_update()
        .set(&admin_membership)
        .get_result(&*connection)
        .map_err(Into::into)
}
