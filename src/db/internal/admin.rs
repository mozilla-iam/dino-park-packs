use crate::db::internal;
use crate::db::logs::log_comment_body;
use crate::db::logs::LogContext;
use crate::db::model::*;
use crate::db::schema;
use crate::db::types::*;
use crate::db::Pool;
use crate::user::User;
use crate::utils::to_expiration_ts;
use diesel::prelude::*;
use failure::Error;
use uuid::Uuid;

const ROLE_ADMIN: &str = "admin";

pub fn add_admin_role(log_ctx: &LogContext, pool: &Pool, group_id: i32) -> Result<Role, Error> {
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
        .map(|role| {
            internal::log::db_log(
                &connection,
                &log_ctx,
                LogTargetType::Role,
                LogOperationType::Created,
                log_comment_body("admin"),
            );
            role
        })
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

pub fn demote_to_member(
    host_uuid: &Uuid,
    pool: &Pool,
    group_name: &str,
    user: &User,
    expiration: Option<i32>,
) -> Result<Membership, Error> {
    let connection = pool.get()?;
    let expiration = expiration.map(to_expiration_ts);
    let group = internal::group::get_group(pool, group_name)?;
    let role = internal::member::member_role(pool, group_name)?;
    let log_ctx = LogContext::with(group.id, *host_uuid).with_user(user.user_uuid);
    diesel::update(
        schema::memberships::table.filter(
            schema::memberships::user_uuid
                .eq(user.user_uuid)
                .and(schema::memberships::group_id.eq(group.id)),
        ),
    )
    .set((
        schema::memberships::role_id.eq(role.id),
        schema::memberships::expiration.eq(expiration),
    ))
    .get_result(&*connection)
    .map_err(Into::into)
    .map(|membership| {
        internal::log::db_log(
            &connection,
            &log_ctx,
            LogTargetType::Membership,
            LogOperationType::Updated,
            log_comment_body("demoted from admin to member"),
        );
        membership
    })
}

pub fn add_admin(
    pool: &Pool,
    group_name: &str,
    host: &User,
    user: &User,
) -> Result<Membership, Error> {
    let connection = pool.get()?;
    let group = internal::group::get_group(pool, group_name)?;
    let role = get_admin_role(pool, group.id)?;
    let admin_membership = InsertMembership {
        group_id: group.id,
        user_uuid: user.user_uuid,
        role_id: role.id,
        expiration: None,
        added_by: host.user_uuid,
    };
    let log_ctx = LogContext::with(group.id, host.user_uuid).with_user(user.user_uuid);
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
        .map(|membership| {
            internal::log::db_log(
                &connection,
                &log_ctx,
                LogTargetType::Membership,
                LogOperationType::Created,
                log_comment_body("admin"),
            );
            membership
        })
}

pub fn is_last_admin(pool: &Pool, group_name: &str, user_uuid: &Uuid) -> Result<bool, Error> {
    let group = internal::group::get_group(&pool, group_name)?;
    let role = get_admin_role(pool, group.id)?;
    let connection = pool.get()?;
    schema::memberships::table
        .filter(schema::memberships::role_id.eq(role.id))
        .select(schema::memberships::user_uuid)
        .get_results(&connection)
        .map(|admins: Vec<Uuid>| admins.contains(user_uuid) && admins.len() == 1)
        .map_err(Into::into)
}
