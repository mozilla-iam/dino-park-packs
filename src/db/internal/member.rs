use crate::db::internal;
use crate::db::logs::log_comment_body;
use crate::db::logs::LogContext;
use crate::db::model::*;
use crate::db::operations::models::*;
use crate::db::schema;
use crate::db::types::LogOperationType;
use crate::db::types::LogTargetType;
use crate::db::types::*;
use crate::db::views;
use crate::user::User;
use crate::utils::to_expiration_ts;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use dino_park_trust::Trust;
use failure::Error;
use serde_json::Value;
use uuid::Uuid;

const ROLE_MEMBER: &str = "member";

macro_rules! scoped_members_for {
    ($t:ident, $f:ident) => {
        pub fn $f(
            connection: &PgConnection,
            group_name: &str,
            options: MembersQueryOptions,
            scope: &Trust,
        ) -> Result<PaginatedDisplayMembersAndHost, Error> {
            use schema::groups as g;
            use schema::memberships as m;
            use schema::roles as r;
            use schema::$t as u;
            let offset = options.offset.unwrap_or_default();
            let q = format!("{}%", options.query.unwrap_or_default());
            let group: Group = g::table.filter(g::name.eq(group_name)).first(connection)?;
            let members: Vec<DisplayMemberAndHost> = m::table
                .filter(m::group_id.eq(group.id))
                .inner_join(u::table.on(m::user_uuid.eq(u::user_uuid)))
                .inner_join(r::table)
                .filter(r::typ.eq_any(options.roles))
                .filter(
                    u::first_name
                        .concat(" ")
                        .concat(u::last_name)
                        .ilike(&q)
                        .or(u::first_name.ilike(&q))
                        .or(u::last_name.ilike(&q))
                        .or(u::username.ilike(&q))
                        .or(u::email.ilike(&q)),
                )
                .then_order_by(u::username)
                .select((
                    m::user_uuid,
                    u::picture,
                    u::first_name,
                    u::last_name,
                    u::username,
                    u::email,
                    u::trust.eq(TrustType::Staff),
                    r::typ,
                    m::added_ts,
                ))
                .offset(offset)
                .limit(options.limit)
                .get_results::<Member>(connection)
                .map(|members| {
                    members
                        .into_iter()
                        .map(|m| DisplayMemberAndHost::from_with_socpe(m, scope))
                        .collect()
                })?;
            let next = match members.len() {
                0 => None,
                l => Some(offset + l as i64),
            };
            Ok(PaginatedDisplayMembersAndHost { next, members })
        }
    };
}

macro_rules! scoped_members_and_host_for {
    ($t:ident, $h:ident, $f:ident) => {
        pub fn $f(
            connection: &PgConnection,
            group_name: &str,
            options: MembersQueryOptions,
        ) -> Result<PaginatedDisplayMembersAndHost, Error> {
            use schema::groups as g;
            use schema::memberships as m;
            use schema::roles as r;
            use schema::$t as u;
            use views::$h as h;
            let offset = options.offset.unwrap_or_default();
            let q = format!("{}%", options.query.unwrap_or_default());
            let group: Group = g::table.filter(g::name.eq(group_name)).first(connection)?;
            let mut query = m::table
                .filter(m::group_id.eq(group.id))
                .inner_join(u::table.on(m::user_uuid.eq(u::user_uuid)))
                .left_outer_join(h::table.on(m::added_by.eq(h::user_uuid)))
                .inner_join(r::table)
                .filter(r::typ.eq_any(options.roles))
                .filter(
                    u::first_name
                        .concat(" ")
                        .concat(u::last_name)
                        .ilike(&q)
                        .or(u::first_name.ilike(&q))
                        .or(u::last_name.ilike(&q))
                        .or(u::username.ilike(&q))
                        .or(u::email.ilike(&q)),
                )
                .select((
                    m::user_uuid,
                    u::picture,
                    u::first_name,
                    u::last_name,
                    u::username,
                    u::email,
                    u::trust.eq(TrustType::Staff),
                    m::added_ts,
                    m::expiration,
                    r::typ,
                    m::added_by,
                    h::first_name.nullable(),
                    h::last_name.nullable(),
                    h::username.nullable(),
                    h::email.nullable(),
                ))
                .into_boxed();
            query = match options.order {
                SortMembersBy::None => query,
                SortMembersBy::ExpirationAsc => query.order_by((m::expiration.asc(), r::typ)),
                SortMembersBy::ExpirationDesc => query.order_by((m::expiration.desc(), r::typ)),
                SortMembersBy::RoleAsc => query.order_by((r::typ.asc(), m::expiration)),
                SortMembersBy::RoleDesc => query.order_by((r::typ.desc(), m::expiration)),
            };

            query = query
                .then_order_by(u::username)
                .offset(offset)
                .limit(options.limit);
            let members: Vec<DisplayMemberAndHost> = query
                .get_results::<MemberAndHost>(connection)
                .map(|members| members.into_iter().map(|m| m.into()).collect())?;
            let next = match members.len() {
                0 => None,
                l => Some(offset + l as i64),
            };
            Ok(PaginatedDisplayMembersAndHost { next, members })
        }
    };
}

// scoped_members_for!(users_staff, staff_scoped_members);
// scoped_members_for!(users_ndaed, ndaed_scoped_members);
scoped_members_for!(users_vouched, vouched_scoped_members);
scoped_members_for!(users_authenticated, authenticated_scoped_members);
scoped_members_for!(users_public, public_scoped_members);

scoped_members_and_host_for!(users_staff, hosts_staff, staff_scoped_members_and_host);
scoped_members_and_host_for!(users_ndaed, hosts_ndaed, ndaed_scoped_members_and_host);
/*
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
*/

pub fn add_member_role(
    host_uuid: &Uuid,
    connection: &PgConnection,
    group_id: i32,
) -> Result<Role, Error> {
    let admin = InsertRole {
        group_id,
        typ: RoleType::Member,
        name: ROLE_MEMBER.to_owned(),
        permissions: vec![],
    };
    let log_ctx = LogContext::with(group_id, *host_uuid);
    diesel::insert_into(schema::roles::table)
        .values(admin)
        .get_result(connection)
        .map(|role| {
            internal::log::db_log(
                connection,
                &log_ctx,
                LogTargetType::Role,
                LogOperationType::Created,
                log_comment_body("member"),
            );
            role
        })
        .map_err(Into::into)
}

pub fn role_for(
    connection: &PgConnection,
    user_uuid: &Uuid,
    group_name: &str,
) -> Result<Role, Error> {
    schema::memberships::table
        .filter(schema::memberships::user_uuid.eq(user_uuid))
        .inner_join(schema::groups::table)
        .filter(schema::groups::name.eq(group_name))
        .inner_join(schema::roles::table)
        .get_result::<(Membership, Group, Role)>(connection)
        .map(|(_, _, r)| r)
        .map_err(Into::into)
}

pub fn member_role(connection: &PgConnection, group_name: &str) -> Result<Role, Error> {
    schema::roles::table
        .inner_join(schema::groups::table)
        .filter(schema::groups::name.eq(group_name))
        .filter(schema::roles::typ.eq(RoleType::Member))
        .get_result::<(Role, Group)>(connection)
        .map(|(r, _)| r)
        .map_err(Into::into)
}

pub fn remove_from_group(
    host_uuid: &Uuid,
    connection: &PgConnection,
    user_uuid: &Uuid,
    group_name: &str,
    comment: Option<Value>,
) -> Result<(), Error> {
    let group = internal::group::get_group(connection, group_name)?;
    let log_ctx = LogContext::with(group.id, *host_uuid).with_user(*user_uuid);
    diesel::delete(schema::memberships::table)
        .filter(schema::memberships::user_uuid.eq(user_uuid))
        .filter(schema::memberships::group_id.eq(group.id))
        .execute(connection)
        .map(|_| {
            internal::log::db_log(
                connection,
                &log_ctx,
                LogTargetType::Membership,
                LogOperationType::Deleted,
                comment,
            );
        })
        .map_err(Into::into)
}

pub fn add_to_group(
    connection: &PgConnection,
    group_name: &str,
    host: &User,
    member: &User,
    expiration: Option<i32>,
) -> Result<(), Error> {
    let group = internal::group::get_group(connection, group_name)?;
    let role = internal::member::member_role(connection, group_name)?;
    let expiration = internal::expiration::map_expiration(expiration, group.group_expiration);
    let membership = InsertMembership {
        group_id: group.id,
        user_uuid: member.user_uuid,
        role_id: role.id,
        expiration,
        added_by: host.user_uuid,
    };
    let log_ctx = LogContext::with(group.id, host.user_uuid).with_user(member.user_uuid);
    diesel::insert_into(schema::memberships::table)
        .values(&membership)
        .on_conflict((
            schema::memberships::user_uuid,
            schema::memberships::group_id,
        ))
        .do_update()
        .set(&membership)
        .execute(connection)
        .map(|_| {
            internal::log::db_log(
                connection,
                &log_ctx,
                LogTargetType::Membership,
                LogOperationType::Created,
                log_comment_body("added"),
            );
        })
        .map_err(Into::into)
}

pub fn renew(
    host_uuid: &Uuid,
    connection: &PgConnection,
    group_name: &str,
    member: &User,
    expiration: Option<i32>,
) -> Result<(), Error> {
    let group = internal::group::get_group(connection, group_name)?;
    let log_ctx = LogContext::with(group.id, *host_uuid).with_user(member.user_uuid);
    diesel::update(
        schema::memberships::table.filter(
            schema::memberships::group_id
                .eq(group.id)
                .and(schema::memberships::user_uuid.eq(member.user_uuid)),
        ),
    )
    .set(schema::memberships::expiration.eq(expiration.map(to_expiration_ts)))
    .execute(connection)
    .map(|_| {
        internal::log::db_log(
            connection,
            &log_ctx,
            LogTargetType::Membership,
            LogOperationType::Updated,
            log_comment_body("renewed"),
        );
    })
    .map_err(Into::into)
}

pub fn get_members_not_current(
    connection: &PgConnection,
    group_name: &str,
    current: &User,
) -> Result<Vec<User>, Error> {
    let group = internal::group::get_group(connection, group_name)?;
    schema::memberships::table
        .filter(schema::memberships::group_id.eq(group.id))
        .filter(schema::memberships::user_uuid.ne(current.user_uuid))
        .select(schema::memberships::user_uuid)
        .get_results(connection)
        .map(|r| r.into_iter().map(|user_uuid| User { user_uuid }).collect())
        .map_err(Into::into)
}

pub fn get_memberships_expired_before(
    connection: &PgConnection,
    before: NaiveDateTime,
) -> Result<Vec<Membership>, Error> {
    schema::memberships::table
        .filter(schema::memberships::expiration.le(before))
        .get_results(connection)
        .map_err(Into::into)
}
