use crate::db::error::DBError;
use crate::db::internal;
use crate::db::logs::log_comment_body;
use crate::db::logs::LogContext;
use crate::db::model::*;
use crate::db::operations::models::DisplayRequest;
use crate::db::operations::models::DisplayRequestForUser;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::db::types::LogOperationType;
use crate::db::types::LogTargetType;
use crate::db::types::TrustType;
use crate::user::User;
use chrono::NaiveDateTime;
use diesel::dsl;
use diesel::prelude::*;
use failure::Error;
use serde_json::Value;

pub fn requests_for_user(
    connection: &PgConnection,
    user: &User,
) -> Result<Vec<DisplayRequestForUser>, Error> {
    use schema::groups as g;
    use schema::requests as r;
    use schema::terms as t;
    r::table
        .filter(r::user_uuid.eq(user.user_uuid))
        .inner_join(g::table.on(g::group_id.eq(r::group_id)))
        .filter(g::active.eq(true))
        .left_outer_join(t::table.on(t::group_id.eq(r::group_id)))
        .select((
            r::user_uuid,
            r::created,
            r::request_expiration,
            g::name,
            t::text.nullable().is_not_null(),
        ))
        .get_results::<DisplayRequestForUser>(connection)
        .map_err(Into::into)
}

macro_rules! scoped_requests_for {
    ($t:ident, $f:ident) => {
        pub fn $f(
            connection: &PgConnection,
            group_name: &str,
        ) -> Result<Vec<DisplayRequest>, Error> {
            use schema::groups as g;
            use schema::requests as r;
            use schema::terms as t;
            use schema::$t as u;
            g::table
                .filter(g::name.eq(group_name))
                .filter(g::active.eq(true))
                .inner_join(r::table.on(r::group_id.eq(g::group_id)))
                .left_outer_join(t::table.on(t::group_id.eq(r::group_id)))
                .inner_join(u::table.on(u::user_uuid.eq(r::user_uuid)))
                .select((
                    u::user_uuid,
                    u::picture,
                    u::first_name,
                    u::last_name,
                    u::username,
                    u::email,
                    u::trust.eq(TrustType::Staff),
                    r::created,
                    r::request_expiration,
                    g::name,
                    t::text.is_not_null(),
                ))
                .get_results::<DisplayRequest>(connection)
                .map_err(Into::into)
        }
    };
}

scoped_requests_for!(users_staff, staff_scoped_requests);
scoped_requests_for!(users_ndaed, ndaed_scoped_requests);
scoped_requests_for!(users_vouched, vouched_scoped_requests);
scoped_requests_for!(users_authenticated, authenticated_scoped_requests);
scoped_requests_for!(users_public, public_scoped_requests);

pub fn request(
    connection: &PgConnection,
    group_name: &str,
    member: User,
    request_expiration: Option<NaiveDateTime>,
) -> Result<(), Error> {
    let group = internal::group::get_group(connection, group_name)?;
    let req = InsertRequest {
        user_uuid: member.user_uuid,
        group_id: group.id,
        request_expiration,
    };
    let log_ctx = LogContext::with(group.id, member.user_uuid);
    let rows = diesel::insert_into(schema::requests::table)
        .values(&req)
        .on_conflict_do_nothing()
        .execute(&*connection)
        .map(|r| {
            internal::log::db_log(
                connection,
                &log_ctx,
                LogTargetType::Request,
                LogOperationType::Created,
                None,
            );
            r
        })
        .map_err(Error::from)?;
    match rows {
        1 => Ok(()),
        _ => Err(DBError::NotApplicable.into()),
    }
}

pub fn delete(
    connection: &PgConnection,
    group_name: &str,
    host: Option<User>,
    user: &User,
    comment: Option<Value>,
) -> Result<(), Error> {
    let group = internal::group::get_group(connection, group_name)?;
    let log_ctx = match host {
        Some(host) => LogContext::with(group.id, host.user_uuid).with_user(user.user_uuid),
        None => LogContext::with(group.id, user.user_uuid),
    };
    diesel::delete(schema::requests::table)
        .filter(schema::requests::user_uuid.eq(user.user_uuid))
        .filter(schema::requests::group_id.eq(group.id))
        .execute(&*connection)
        .map(|count| {
            if count > 0 {
                internal::log::db_log(
                    connection,
                    &log_ctx,
                    LogTargetType::Request,
                    LogOperationType::Deleted,
                    comment,
                );
            };
        })
        .map_err(Error::from)
}

pub fn cancel(connection: &PgConnection, group_name: &str, user: &User) -> Result<(), Error> {
    delete(
        connection,
        group_name,
        None,
        user,
        log_comment_body("canceled"),
    )
}

pub fn reject(
    connection: &PgConnection,
    group_name: &str,
    host: &User,
    member: &User,
) -> Result<(), Error> {
    delete(
        connection,
        group_name,
        Some(*host),
        member,
        log_comment_body("rejected"),
    )
}

pub fn count(connection: &PgConnection, group_name: &str) -> Result<i64, Error> {
    let count = schema::requests::table
        .inner_join(groups::groups)
        .filter(groups::name.eq(group_name))
        .select(dsl::count(schema::requests::user_uuid))
        .first(connection)?;
    Ok(count)
}
