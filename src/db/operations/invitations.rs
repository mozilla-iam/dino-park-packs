use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::operations::internal::invitation;
use crate::db::operations::models::*;
use crate::db::schema;
use crate::db::types::*;
use crate::db::views;
use crate::rules::engine::*;
use crate::rules::rules::RuleContext;
use crate::user::User;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use serde_derive::Serialize;

#[derive(Queryable, Serialize)]
pub struct PendingInvitations {}

macro_rules! scoped_invitations_for {
    ($t:ident, $h:ident, $f:ident) => {
        fn $f(
            connection: &PgConnection,
            group_name: &str,
        ) -> Result<Vec<DisplayInvitation>, Error> {
            use schema::groups as g;
            use schema::invitations as i;
            use schema::$t as u;
            use views::$h as h;
            g::table
                .filter(g::name.eq(group_name))
                .first(connection)
                .and_then(|group: Group| {
                    i::table
                        .filter(i::group_id.eq(group.id))
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
                            h::user_uuid,
                            h::first_name.concat(" ").concat(h::last_name),
                            h::username,
                            h::email,
                        ))
                        .get_results::<InvitationAndHost>(connection)
                        .map(|invitations| invitations.into_iter().map(|m| m.into()).collect())
                })
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

pub fn invite_member(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: User,
    member: User,
    invitation_expiration: Option<NaiveDateTime>,
    group_expiration: Option<NaiveDateTime>,
) -> Result<(), Error> {
    // TODO: check db rules
    INVITE_MEMBER.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    invitation::invite(
        pool,
        group_name,
        host,
        member,
        invitation_expiration,
        group_expiration,
    )
}

pub fn pending_invitations_count(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
) -> Result<i64, Error> {
    HOST_IS_CURATOR.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    invitation::pending_count(pool, group_name)
}

pub fn pending_invitations(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
) -> Result<Vec<DisplayInvitation>, Error> {
    HOST_IS_CURATOR.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    let connection = pool.get()?;
    match scope_and_user.scope.as_str() {
        "staff" => staff_scoped_invitations_and_host(&connection, group_name),
        "ndaed" => ndaed_scoped_invitations_and_host(&connection, group_name),
        "vouched" => vouched_scoped_invitations_and_host(&connection, group_name),
        "authenticated" => authenticated_scoped_invitations_and_host(&connection, group_name),
        _ => public_scoped_invitations_and_host(&connection, group_name),
    }
}

pub fn accept_invitation(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    user: &User,
) -> Result<(), Error> {
    invitation::accept(pool, group_name, user)
}
