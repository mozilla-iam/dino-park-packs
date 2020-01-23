use crate::cis::operations::add_group_to_profile;
use crate::db::internal;
use crate::db::internal::invitation::*;
use crate::db::operations::models::*;
use crate::db::Pool;
use crate::rules::engine::*;
use crate::rules::RuleContext;
use crate::user::User;
use chrono::NaiveDateTime;
use cis_client::CisClient;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use futures::future::IntoFuture;
use futures::Future;
use serde_derive::Serialize;
use std::sync::Arc;

#[derive(Queryable, Serialize)]
pub struct PendingInvitations {}

pub fn delete_invitation(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: User,
    member: User,
) -> Result<(), Error> {
    INVITE_MEMBER.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    let connection = pool.get()?;
    delete(&connection, group_name, host, member)
}

pub fn reject_invitation(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    delete(&connection, group_name, User::default(), user)
}

pub fn update_invitation(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: User,
    member: User,
    invitation_expiration: Option<NaiveDateTime>,
    group_expiration: Option<i32>,
) -> Result<(), Error> {
    INVITE_MEMBER.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    let connection = pool.get()?;
    update(
        &connection,
        group_name,
        host,
        member,
        invitation_expiration,
        group_expiration,
    )
}

pub fn invite_member(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: User,
    member: User,
    invitation_expiration: Option<NaiveDateTime>,
    group_expiration: Option<i32>,
) -> Result<(), Error> {
    // TODO: check db rules
    INVITE_MEMBER.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    let connection = pool.get()?;
    invite(
        &connection,
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
    let connection = pool.get()?;
    pending_count(&connection, group_name)
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

pub fn pending_invitations_for_user(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    user: &User,
) -> Result<Vec<DisplayInvitation>, Error> {
    let connection = pool.get()?;
    match scope_and_user.scope.as_str() {
        "staff" => staff_scoped_invitations_and_host_for_user(&connection, user),
        "ndaed" => ndaed_scoped_invitations_and_host_for_user(&connection, user),
        "vouched" => vouched_scoped_invitations_and_host_for_user(&connection, user),
        "authenticated" => authenticated_scoped_invitations_and_host_for_user(&connection, user),
        _ => public_scoped_invitations_and_host_for_user(&connection, user),
    }
}

pub fn accept_invitation(
    pool: &Pool,
    group_name: &str,
    user: &User,
    cis_client: Arc<CisClient>,
    profile: Profile,
) -> impl Future<Item = (), Error = Error> {
    let group_name_f = group_name.to_owned();
    pool.get()
        .map_err(Into::into)
        .and_then(|connection| accept(&connection, group_name, user))
        .into_future()
        .and_then(move |_| add_group_to_profile(cis_client, group_name_f, profile))
}
