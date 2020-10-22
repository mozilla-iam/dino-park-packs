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
use chrono::NaiveDateTime;
use diesel::prelude::*;
use dino_park_trust::Trust;
use failure::Error;
use serde_json::Value;
use uuid::Uuid;

const ROLE_MEMBER: &str = "member";

macro_rules! scoped_members_for {
    ($t:ident, $f:ident, $s:ident) => {
        pub fn $f(
            connection: &PgConnection,
            group_id: i32,
            options: MembersQueryOptions,
        ) -> Result<PaginatedDisplayMembersAndHost, Error> {
            use schema::memberships as m;
            use schema::roles as r;
            use schema::$t as u;
            let offset = options.offset.unwrap_or_default();
            let limit = options.limit;
            let q = format!("{}%", options.query.unwrap_or_default());
            let (members, next) = m::table
                .filter(m::group_id.eq(group_id))
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
                .limit(limit + 1)
                .get_results::<Member>(connection)
                .map(|members| {
                    let next = match members.len() as i64 {
                        l if l > limit => Some(offset + limit),
                        _ => None,
                    };
                    let members: Vec<DisplayMemberAndHost> = members
                        .into_iter()
                        .take(limit as usize)
                        .map(|m| DisplayMemberAndHost::from_with_scope(m, &Trust::$s))
                        .collect();
                    (members, next)
                })?;
            Ok(PaginatedDisplayMembersAndHost { next, members })
        }
    };
}

macro_rules! scoped_members_and_host_for {
    ($t:ident, $h:ident, $f:ident) => {
        pub fn $f(
            connection: &PgConnection,
            group_id: i32,
            options: MembersQueryOptions,
        ) -> Result<PaginatedDisplayMembersAndHost, Error> {
            use schema::memberships as m;
            use schema::roles as r;
            use schema::$t as u;
            use views::$h as h;
            let offset = options.offset.unwrap_or_default();
            let limit = options.limit;
            let q = format!("{}%", options.query.unwrap_or_default());
            let mut query = m::table
                .filter(m::group_id.eq(group_id))
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
                .limit(limit + 1);
            let (members, next) =
                query
                    .get_results::<MemberAndHost>(connection)
                    .map(|members| {
                        let next = match members.len() as i64 {
                            l if l > limit => Some(offset + limit),
                            _ => None,
                        };
                        let members: Vec<DisplayMemberAndHost> = members
                            .into_iter()
                            .take(limit as usize)
                            .map(|m| m.into())
                            .collect();
                        (members, next)
                    })?;
            Ok(PaginatedDisplayMembersAndHost { next, members })
        }
    };
}

macro_rules! privileged_scoped_members_and_host_for {
    ($t:ident, $h:ident, $f:ident) => {
        pub fn $f(
            connection: &PgConnection,
            group_id: i32,
            options: MembersQueryOptions,
        ) -> Result<PaginatedDisplayMembersAndHost, Error> {
            use schema::legacy_user_data as l;
            use schema::memberships as m;
            use schema::roles as r;
            use schema::$t as u;
            use views::$h as h;
            let offset = options.offset.unwrap_or_default();
            let limit = options.limit;
            let q = format!("{}%", options.query.unwrap_or_default());
            let mut query = m::table
                .filter(m::group_id.eq(group_id))
                .inner_join(u::table.on(m::user_uuid.eq(u::user_uuid)))
                .left_outer_join(l::table.on(m::user_uuid.eq(l::user_uuid)))
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
                        .or(u::email.ilike(&q))
                        .or(l::first_name.ilike(&q))
                        .or(l::email.ilike(&q)),
                )
                .select((
                    m::user_uuid,
                    u::picture,
                    u::first_name,
                    l::first_name.nullable(),
                    u::last_name,
                    u::username,
                    u::email.nullable(),
                    l::email.nullable(),
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
                .limit(limit + 1);
            let (members, next) =
                query
                    .get_results::<LegacyMemberAndHost>(connection)
                    .map(|members| {
                        let next = match members.len() as i64 {
                            l if l > limit => Some(offset + limit),
                            _ => None,
                        };
                        let members: Vec<DisplayMemberAndHost> = members
                            .into_iter()
                            .take(limit as usize)
                            .map(|m| m.into())
                            .collect();
                        (members, next)
                    })?;
            Ok(PaginatedDisplayMembersAndHost { next, members })
        }
    };
}

macro_rules! membership_and_scoped_host_for {
    ($h:ident, $f:ident) => {
        pub fn $f(
            connection: &PgConnection,
            group_id: i32,
            user_uuid: Uuid,
        ) -> Result<Option<DisplayMembershipAndHost>, Error> {
            use schema::memberships as m;
            use schema::roles as r;
            use views::$h as h;
            m::table
                .filter(m::group_id.eq(group_id))
                .filter(m::user_uuid.eq(user_uuid))
                .left_outer_join(h::table.on(m::added_by.eq(h::user_uuid)))
                .inner_join(r::table)
                .select((
                    m::user_uuid,
                    m::added_ts,
                    m::expiration,
                    r::typ,
                    m::added_by,
                    h::first_name.nullable(),
                    h::last_name.nullable(),
                    h::username.nullable(),
                    h::email.nullable(),
                ))
                .first::<MembershipAndHost>(connection)
                .optional()
                .map(|r| r.map(Into::into))
                .map_err(Error::from)
        }
    };
}

// scoped_members_for!(users_staff, staff_scoped_members);
// scoped_members_for!(users_ndaed, ndaed_scoped_members);
scoped_members_for!(users_vouched, vouched_scoped_members, Vouched);
scoped_members_for!(
    users_authenticated,
    authenticated_scoped_members,
    Authenticated
);
scoped_members_for!(users_public, public_scoped_members, Public);

scoped_members_and_host_for!(users_staff, hosts_staff, staff_scoped_members_and_host);
scoped_members_and_host_for!(users_ndaed, hosts_ndaed, ndaed_scoped_members_and_host);
privileged_scoped_members_and_host_for!(
    users_staff,
    hosts_staff,
    privileged_staff_scoped_members_and_host
);
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

membership_and_scoped_host_for!(hosts_staff, membership_and_staff_host);
membership_and_scoped_host_for!(hosts_ndaed, membership_and_ndaed_host);
membership_and_scoped_host_for!(hosts_vouched, membership_and_vouched_host);
membership_and_scoped_host_for!(hosts_authenticated, membership_and_authenticated_host);

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
) -> Result<Option<Role>, Error> {
    schema::memberships::table
        .filter(schema::memberships::user_uuid.eq(user_uuid))
        .inner_join(schema::groups::table)
        .filter(schema::groups::name.eq(group_name))
        .inner_join(schema::roles::table)
        .select(schema::roles::all_columns)
        .get_result::<Role>(connection)
        .optional()
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

pub fn transfer_membership(
    connection: &PgConnection,
    group_name: &str,
    host: &User,
    old_member: &User,
    new_member: &User,
) -> Result<(), Error> {
    let group = internal::group::get_group(connection, group_name)?;
    let log_ctx_old = LogContext::with(group.id, host.user_uuid).with_user(old_member.user_uuid);
    let log_ctx_new = LogContext::with(group.id, host.user_uuid).with_user(new_member.user_uuid);
    diesel::update(
        schema::memberships::table.filter(
            schema::memberships::group_id
                .eq(group.id)
                .and(schema::memberships::user_uuid.eq(old_member.user_uuid)),
        ),
    )
    .set(schema::memberships::user_uuid.eq(new_member.user_uuid))
    .execute(connection)
    .map(|_| {
        internal::log::db_log(
            connection,
            &log_ctx_old,
            LogTargetType::Membership,
            LogOperationType::Updated,
            log_comment_body(&format!("moved to {}", new_member.user_uuid)),
        );
        internal::log::db_log(
            connection,
            &log_ctx_new,
            LogTargetType::Membership,
            LogOperationType::Updated,
            log_comment_body(&format!("moved from {}", old_member.user_uuid)),
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
    let expiration = internal::expiration::map_expiration(expiration, group.group_expiration);
    let log_ctx = LogContext::with(group.id, *host_uuid).with_user(member.user_uuid);
    diesel::update(
        schema::memberships::table.filter(
            schema::memberships::group_id
                .eq(group.id)
                .and(schema::memberships::user_uuid.eq(member.user_uuid)),
        ),
    )
    .set(schema::memberships::expiration.eq(expiration))
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

pub fn get_members_by_trust_less_than(
    connection: &PgConnection,
    group_name: &str,
    trust: &TrustType,
) -> Result<Vec<User>, Error> {
    let group = internal::group::get_group(connection, group_name)?;
    schema::memberships::table
        .filter(schema::memberships::group_id.eq(group.id))
        .left_join(
            schema::profiles::table
                .on(schema::profiles::user_uuid.eq(schema::memberships::user_uuid)),
        )
        .filter(schema::profiles::trust.lt(trust))
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

pub fn get_memberships_expire_between(
    connection: &PgConnection,
    lower: NaiveDateTime,
    upper: NaiveDateTime,
) -> Result<Vec<Membership>, Error> {
    schema::memberships::table
        .filter(schema::memberships::expiration.between(lower, upper))
        .get_results(connection)
        .map_err(Into::into)
}

pub fn get_member_emails_by_group_name(
    connection: &PgConnection,
    group_name: &str,
) -> Result<Vec<String>, Error> {
    use schema::groups as g;
    use schema::memberships as m;
    use schema::profiles as p;
    g::table
        .filter(g::name.eq(group_name))
        .inner_join(m::table)
        .inner_join(p::table.on(m::user_uuid.eq(p::user_uuid)))
        .select(p::email)
        .get_results::<String>(connection)
        .map_err(Into::into)
}

pub fn get_curator_emails(connection: &PgConnection, group_id: i32) -> Result<Vec<String>, Error> {
    use schema::memberships as m;
    use schema::profiles as p;
    use schema::roles as r;
    m::table
        .filter(m::group_id.eq(group_id))
        .inner_join(r::table.on(r::role_id.eq(m::role_id)))
        .filter(r::typ.eq_any(&[RoleType::Admin, RoleType::Curator]))
        .inner_join(p::table.on(m::user_uuid.eq(p::user_uuid)))
        .select(p::email)
        .get_results::<String>(connection)
        .map_err(Into::into)
}

pub fn get_curator_emails_by_group_name(
    connection: &PgConnection,
    group_name: &str,
) -> Result<Vec<String>, Error> {
    use schema::groups as g;
    use schema::memberships as m;
    use schema::profiles as p;
    use schema::roles as r;
    g::table
        .filter(g::name.eq(group_name))
        .inner_join(m::table)
        .inner_join(r::table.on(r::role_id.eq(m::role_id)))
        .filter(r::typ.eq_any(&[RoleType::Admin, RoleType::Curator]))
        .inner_join(p::table.on(m::user_uuid.eq(p::user_uuid)))
        .select(p::email)
        .get_results::<String>(connection)
        .map_err(Into::into)
}

pub fn get_anonymous_member_emails(connection: &PgConnection) -> Result<Vec<String>, Error> {
    use schema::memberships as m;
    use schema::profiles as p;
    use schema::users_ndaed as u;
    m::table
        .inner_join(u::table.on(m::user_uuid.eq(u::user_uuid)))
        .filter(
            u::trust.ne(TrustType::Staff).and(
                u::email
                    .is_null()
                    .or(u::first_name.is_null().and(u::last_name.is_null())),
            ),
        )
        .inner_join(p::table.on(m::user_uuid.eq(p::user_uuid)))
        .select(p::email)
        .distinct()
        .get_results::<String>(connection)
        .map_err(Into::into)
}

pub fn group_names_for_user(
    connection: &PgConnection,
    user_uuid: &Uuid,
) -> Result<Vec<String>, Error> {
    use schema::groups as g;
    use schema::memberships as m;

    m::table
        .filter(m::user_uuid.eq(user_uuid))
        .inner_join(g::table)
        .select(g::name)
        .get_results::<String>(connection)
        .map_err(Into::into)
}
