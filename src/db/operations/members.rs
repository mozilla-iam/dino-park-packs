use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::operations::internal;
use crate::db::operations::models::*;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::db::schema::memberships::dsl::*;
use crate::db::types::*;
use crate::db::views;
use crate::user::User;
use chrono::NaiveDateTime;
use chrono::Utc;
use diesel::dsl::count;
use diesel::prelude::*;
use failure::format_err;
use failure::Error;
use log::info;
use serde_derive::Serialize;
use uuid::Uuid;

const DEFAULT_RENEWAL_DAYS: i64 = 14;

macro_rules! scoped_members_for {
    ($t:ident, $f:ident) => {
        fn $f(
            connection: &PgConnection,
            group_name: &str,
            roles: &[RoleType],
            limit: i64,
            offset: Option<i64>,
        ) -> Result<PaginatedDisplayMembers, Error> {
            use schema::groups as g;
            use schema::roles as r;
            use schema::$t as u;
            let offset = offset.unwrap_or_default();
            g::table
                .filter(g::name.eq(group_name))
                .first(connection)
                .and_then(|group: Group| {
                    memberships
                        .filter(group_id.eq(group.id))
                        .inner_join(u::table.on(user_uuid.eq(u::user_uuid)))
                        .inner_join(r::table)
                        .filter(r::typ.eq_any(roles))
                        .select((
                            user_uuid,
                            u::picture,
                            u::first_name.concat(" ").concat(u::last_name),
                            u::username,
                            u::email,
                            u::trust.eq(TrustType::Staff),
                            added_ts,
                            expiration,
                            r::typ,
                        ))
                        .offset(offset)
                        .limit(limit)
                        .get_results::<DisplayMember>(connection)
                })
                .map(|members| {
                    let next = match members.len() {
                        0 => None,
                        l => Some(offset + l as i64),
                    };
                    PaginatedDisplayMembers { next, members }
                })
                .map_err(Into::into)
        }
    };
}

macro_rules! scoped_members_and_host_for {
    ($t:ident, $h:ident, $f:ident) => {
        fn $f(
            connection: &PgConnection,
            group_name: &str,
            roles: &[RoleType],
            limit: i64,
            offset: Option<i64>,
        ) -> Result<PaginatedDisplayMembersAndHost, Error> {
            use schema::groups as g;
            use schema::roles as r;
            use schema::$t as u;
            use views::$h as h;
            let offset = offset.unwrap_or_default();
            g::table
                .filter(g::name.eq(group_name))
                .first(connection)
                .and_then(|group: Group| {
                    memberships
                        .filter(group_id.eq(group.id))
                        .inner_join(u::table.on(user_uuid.eq(u::user_uuid)))
                        .inner_join(h::table.on(added_by.eq(h::user_uuid)))
                        .inner_join(r::table)
                        .filter(r::typ.eq_any(roles))
                        .select((
                            user_uuid,
                            u::picture,
                            u::first_name.concat(" ").concat(u::last_name),
                            u::username,
                            u::email,
                            u::trust.eq(TrustType::Staff),
                            added_ts,
                            expiration,
                            r::typ,
                            h::user_uuid,
                            h::first_name.concat(" ").concat(h::last_name),
                            h::username,
                            h::email,
                        ))
                        .offset(offset)
                        .limit(limit)
                        .get_results::<MemberAndHost>(connection)
                        .map(|members| members.into_iter().map(|m| m.into()).collect())
                })
                .map(|members: Vec<DisplayMemberAndHost>| {
                    let next = match members.len() {
                        0 => None,
                        l => Some(offset + l as i64),
                    };
                    PaginatedDisplayMembersAndHost { next, members }
                })
                .map_err(Into::into)
        }
    };
}

scoped_members_for!(users_staff, staff_scoped_members);
scoped_members_for!(users_ndaed, ndaed_scoped_members);
scoped_members_for!(users_authenticated, authenticated_scoped_members);
scoped_members_for!(users_vouched, vouched_scoped_members);
scoped_members_for!(users_public, public_scoped_members);

scoped_members_and_host_for!(users_staff, hosts_staff, staff_scoped_members_and_host);
scoped_members_and_host_for!(users_ndaed, hosts_ndaed, ndaed_scoped_members_and_host);
scoped_members_and_host_for!(
    users_vouched,
    hosts_vouched,
    vouched_scoped_members_and_host
);
scoped_members_and_host_for!(
    users_authenticated,
    hosts_authenticated,
    authenticated_scoped_members_and_host
);
scoped_members_and_host_for!(users_public, hosts_public, public_scoped_members_and_host);

pub fn scoped_members(
    pool: &Pool,
    group_name: &str,
    scope: &str,
    role: &[RoleType],
    limit: i64,
    offset: Option<i64>,
) -> Result<PaginatedDisplayMembers, Error> {
    let connection = pool.get()?;
    let members = match scope {
        "staff" => staff_scoped_members(&connection, group_name, role, limit, offset),
        "ndaed" => ndaed_scoped_members(&connection, group_name, role, limit, offset),
        "vouched" => vouched_scoped_members(&connection, group_name, role, limit, offset),
        "authenticated" => {
            authenticated_scoped_members(&connection, group_name, role, limit, offset)
        }
        "public" => public_scoped_members(&connection, group_name, role, limit, offset),
        _ => return Err(format_err!("invalid scope")),
    };

    members
}

pub fn scoped_members_and_host(
    pool: &Pool,
    group_name: &str,
    scope: &str,
    role: &[RoleType],
    limit: i64,
    offset: Option<i64>,
) -> Result<PaginatedDisplayMembersAndHost, Error> {
    let connection = pool.get()?;
    let members = match scope {
        "staff" => staff_scoped_members_and_host(&connection, group_name, role, limit, offset),
        "ndaed" => ndaed_scoped_members_and_host(&connection, group_name, role, limit, offset),
        "vouched" => vouched_scoped_members_and_host(&connection, group_name, role, limit, offset),
        "authenticated" => {
            authenticated_scoped_members_and_host(&connection, group_name, role, limit, offset)
        }
        "public" => public_scoped_members_and_host(&connection, group_name, role, limit, offset),
        _ => return Err(format_err!("invalid scope")),
    };

    members
}

pub fn member_count(pool: &Pool, group_name: &str) -> Result<i64, Error> {
    let connection = pool.get()?;
    let count = memberships
        .inner_join(groups::groups)
        .filter(groups::name.eq(group_name))
        .select(count(user_uuid))
        .first(&connection)?;
    Ok(count)
}

pub fn renewal_count(
    pool: &Pool,
    group_name: &str,
    expires_before: Option<NaiveDateTime>,
) -> Result<i64, Error> {
    let expires_before = expires_before
        .unwrap_or_else(|| (Utc::now() + chrono::Duration::days(DEFAULT_RENEWAL_DAYS)).naive_utc());
    let connection = pool.get()?;
    let count = memberships
        .inner_join(groups::groups)
        .filter(groups::name.eq(group_name))
        .filter(expiration.le(expires_before))
        .select(count(user_uuid))
        .first(&connection)?;
    Ok(count)
}

pub use internal::member::member_role;
pub use internal::member::role_for;
