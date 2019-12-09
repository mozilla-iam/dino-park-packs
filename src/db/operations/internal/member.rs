use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::operations;
use crate::db::operations::models::*;
use crate::db::schema;
use crate::db::types::*;
use crate::db::views;
use crate::user::User;
use crate::utils::to_expiration_ts;
use diesel::prelude::*;
use failure::Error;
use uuid::Uuid;

const ROLE_MEMBER: &str = "member";

macro_rules! scoped_members_and_host_for {
    ($t:ident, $h:ident, $f:ident) => {
        pub fn $f(
            connection: &PgConnection,
            group_name: &str,
            query: Option<String>,
            roles: &[RoleType],
            limit: i64,
            offset: Option<i64>,
        ) -> Result<PaginatedDisplayMembersAndHost, Error> {
            use schema::groups as g;
            use schema::memberships as m;
            use schema::roles as r;
            use schema::$t as u;
            use views::$h as h;
            let offset = offset.unwrap_or_default();
            let q = format!("{}%", query.unwrap_or_default());
            g::table
                .filter(g::name.eq(group_name))
                .first(connection)
                .and_then(|group: Group| {
                    m::table
                        .filter(m::group_id.eq(group.id))
                        .inner_join(u::table.on(m::user_uuid.eq(u::user_uuid)))
                        .inner_join(h::table.on(m::added_by.eq(h::user_uuid)))
                        .inner_join(r::table)
                        .filter(r::typ.eq_any(roles))
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
                        .order_by(r::typ)
                        .then_order_by(u::username)
                        .select((
                            m::user_uuid,
                            u::picture,
                            u::first_name.concat(" ").concat(u::last_name),
                            u::username,
                            u::email,
                            u::trust.eq(TrustType::Staff),
                            m::added_ts,
                            m::expiration,
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
        .get_result(&connection)
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
    let group = schema::groups::table
        .filter(schema::groups::name.eq(group_name))
        .first::<Group>(&connection)?;
    diesel::delete(schema::memberships::table)
        .filter(schema::memberships::user_uuid.eq(user_uuid))
        .filter(schema::memberships::group_id.eq(group.id))
        .execute(&connection)?;
    Ok(())
}

pub fn add_to_group(
    pool: &Pool,
    group_name: &str,
    host: &User,
    member: &User,
    expiration: Option<i32>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = schema::groups::table
        .filter(schema::groups::name.eq(group_name))
        .first::<Group>(&*connection)?;
    let role = operations::internal::member::member_role(pool, group_name)?;
    let membership = InsertMembership {
        group_id: group.id,
        user_uuid: member.user_uuid,
        role_id: role.id,
        expiration: expiration.map(to_expiration_ts),
        added_by: host.user_uuid,
    };
    diesel::insert_into(schema::memberships::table)
        .values(&membership)
        .on_conflict((
            schema::memberships::user_uuid,
            schema::memberships::group_id,
        ))
        .do_update()
        .set(&membership)
        .execute(&connection)?;
    Ok(())
}

pub fn renew(
    pool: &Pool,
    group_name: &str,
    member: &User,
    expiration: Option<i32>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = schema::groups::table
        .filter(schema::groups::name.eq(group_name))
        .first::<Group>(&connection)?;
    diesel::update(
        schema::memberships::table.filter(
            schema::memberships::group_id
                .eq(group.id)
                .and(schema::memberships::user_uuid.eq(member.user_uuid)),
        ),
    )
    .set(schema::memberships::expiration.eq(expiration.map(to_expiration_ts)))
    .execute(&connection)?;
    Ok(())
}

pub fn get_members_not_current(
    pool: &Pool,
    group_name: &str,
    current: &User,
) -> Result<Vec<User>, Error> {
    let connection = pool.get()?;
    let group = schema::groups::table
        .filter(schema::groups::name.eq(group_name))
        .first::<Group>(&connection)?;
    schema::memberships::table
        .filter(schema::memberships::group_id.eq(group.id))
        .filter(schema::memberships::user_uuid.ne(current.user_uuid))
        .select(schema::memberships::user_uuid)
        .get_results(&connection)
        .map(|r| r.into_iter().map(|user_uuid| User { user_uuid }).collect())
        .map_err(Into::into)
}
