use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::operations::internal;
use crate::db::operations::models::DisplayInvitation;
use crate::db::operations::models::InvitationAndHost;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::db::types::TrustType;
use crate::db::views;
use crate::user::User;
use chrono::NaiveDateTime;
use diesel::dsl::count;
use diesel::prelude::*;
use failure::Error;

macro_rules! scoped_invitations_for_user {
    ($t:ident, $h:ident, $f:ident) => {
        pub fn $f(connection: &PgConnection, user: &User) -> Result<Vec<DisplayInvitation>, Error> {
            use schema::groups as g;
            use schema::invitations as i;
            use schema::$t as u;
            use views::$h as h;
            i::table
                .filter(i::user_uuid.eq(user.user_uuid))
                .inner_join(g::table.on(g::group_id.eq(i::group_id)))
                .inner_join(u::table.on(u::user_uuid.eq(i::user_uuid)))
                .inner_join(h::table.on(h::user_uuid.eq(i::added_by)))
                .select((
                    u::user_uuid,
                    u::picture,
                    u::first_name.concat(" ").concat(u::last_name),
                    u::username,
                    u::email,
                    u::trust.eq(TrustType::Staff),
                    i::invitation_expiration,
                    i::group_expiration,
                    g::name,
                    h::user_uuid,
                    h::first_name.concat(" ").concat(h::last_name),
                    h::username,
                    h::email,
                ))
                .get_results::<InvitationAndHost>(connection)
                .map(|invitations| invitations.into_iter().map(|m| m.into()).collect())
                .map_err(Into::into)
        }
    };
}

macro_rules! scoped_invitations_for {
    ($t:ident, $h:ident, $f:ident) => {
        pub fn $f(
            connection: &PgConnection,
            group_name: &str,
        ) -> Result<Vec<DisplayInvitation>, Error> {
            use schema::groups as g;
            use schema::invitations as i;
            use schema::$t as u;
            use views::$h as h;
            g::table
                .filter(g::name.eq(group_name))
                .inner_join(i::table.on(i::group_id.eq(g::group_id)))
                .inner_join(u::table.on(u::user_uuid.eq(i::user_uuid)))
                .inner_join(h::table.on(h::user_uuid.eq(i::added_by)))
                .select((
                    u::user_uuid,
                    u::picture,
                    u::first_name.concat(" ").concat(u::last_name),
                    u::username,
                    u::email,
                    u::trust.eq(TrustType::Staff),
                    i::invitation_expiration,
                    i::group_expiration,
                    g::name,
                    h::user_uuid,
                    h::first_name.concat(" ").concat(h::last_name),
                    h::username,
                    h::email,
                ))
                .get_results::<InvitationAndHost>(connection)
                .map(|invitations| invitations.into_iter().map(|m| m.into()).collect())
                .map_err(Into::into)
        }
    };
}

scoped_invitations_for!(users_staff, hosts_staff, staff_scoped_invitations_and_host);
scoped_invitations_for!(users_ndaed, hosts_ndaed, ndaed_scoped_invitations_and_host);
scoped_invitations_for!(
    users_vouched,
    hosts_vouched,
    vouched_scoped_invitations_and_host
);
scoped_invitations_for!(
    users_authenticated,
    hosts_authenticated,
    authenticated_scoped_invitations_and_host
);
scoped_invitations_for!(
    users_public,
    hosts_public,
    public_scoped_invitations_and_host
);

scoped_invitations_for_user!(
    users_staff,
    hosts_staff,
    staff_scoped_invitations_and_host_for_user
);
scoped_invitations_for_user!(
    users_ndaed,
    hosts_ndaed,
    ndaed_scoped_invitations_and_host_for_user
);
scoped_invitations_for_user!(
    users_vouched,
    hosts_vouched,
    vouched_scoped_invitations_and_host_for_user
);
scoped_invitations_for_user!(
    users_authenticated,
    hosts_authenticated,
    authenticated_scoped_invitations_and_host_for_user
);
scoped_invitations_for_user!(
    users_public,
    hosts_public,
    public_scoped_invitations_and_host_for_user
);

pub fn invite(
    pool: &Pool,
    group_name: &str,
    host: User,
    member: User,
    invitation_expiration: Option<NaiveDateTime>,
    group_expiration: Option<NaiveDateTime>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = groups::groups
        .filter(groups::name.eq(group_name))
        .first::<Group>(&*connection)?;
    let invitation = Invitation {
        user_uuid: member.user_uuid,
        group_id: group.id.clone(),
        invitation_expiration,
        group_expiration,
        added_by: host.user_uuid,
    };
    diesel::insert_into(schema::invitations::table)
        .values(&invitation)
        .execute(&*connection)
        .map(|_| ())
        .map_err(Error::from)
}

pub fn pending_count(pool: &Pool, group_name: &str) -> Result<i64, Error> {
    let connection = pool.get()?;
    let count = schema::invitations::table
        .inner_join(groups::groups)
        .filter(groups::name.eq(group_name))
        .select(count(schema::invitations::user_uuid))
        .first(&connection)?;
    Ok(count)
}

pub fn accept(pool: &Pool, group_name: &str, member: &User) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = groups::groups
        .filter(groups::name.eq(group_name))
        .first::<Group>(&*connection)?;
    let invitation = schema::invitations::table
        .filter(
            schema::invitations::user_uuid
                .eq(member.user_uuid)
                .and(schema::invitations::group_id.eq(group.id)),
        )
        .first::<Invitation>(&connection)?;
    let role = internal::member::member_role(pool, group_name)?;
    let membership = InsertMembership {
        group_id: invitation.group_id,
        user_uuid: invitation.user_uuid,
        role_id: role.id,
        // TODO: figure out
        expiration: invitation.group_expiration,
        added_by: invitation.added_by,
    };
    diesel::insert_into(schema::memberships::table)
        .values(&membership)
        .on_conflict((
            schema::memberships::user_uuid,
            schema::memberships::group_id,
        ))
        .do_update()
        .set(&membership)
        .execute(&*connection)?;
    diesel::delete(schema::invitations::table)
        .filter(
            schema::invitations::user_uuid
                .eq(member.user_uuid)
                .and(schema::invitations::group_id.eq(group.id)),
        )
        .execute(&connection)?;
    Ok(())
}
