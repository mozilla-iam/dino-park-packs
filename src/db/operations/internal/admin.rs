use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::schema;
use crate::db::types::*;
use crate::user::User;
use diesel::prelude::*;
use failure::Error;
use uuid::Uuid;

const ROLE_ADMIN: &str = "admin";

pub fn add_admin_role(pool: &Pool, group_id: i32) -> Result<Role, Error> {
    let connection = pool.get()?;
    let admin = InsertRole {
        group_id,
        typ: RoleType::Admin,
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
    group_name: &str,
    host: &User,
    user: &User,
) -> Result<Membership, Error> {
    let connection = pool.get()?;
    let group = schema::groups::table
        .filter(schema::groups::name.eq(group_name))
        .first::<Group>(&*connection)?;
    let role = get_admin_role(pool, group.id)?;
    let admin_membership = InsertMembership {
        group_id: group.id,
        user_uuid: user.user_uuid.clone(),
        role_id: role.id,
        expiration: None,
        added_by: host.user_uuid.clone(),
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

pub fn is_last_admin(pool: &Pool, group_id: i32, user_uuid: &Uuid) -> Result<bool, Error> {
    let role = get_admin_role(pool, group_id)?;
    let connection = pool.get()?;
    schema::memberships::table
        .filter(schema::memberships::role_id.eq(role.id))
        .select(schema::memberships::user_uuid)
        .get_results(&connection)
        .map(|admins: Vec<Uuid>| admins.contains(user_uuid) && admins.len() == 1)
        .map_err(Into::into)
}
