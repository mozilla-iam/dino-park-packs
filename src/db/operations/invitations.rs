use crate::cis::operations::add_group_to_profile;
use crate::db::internal;
use crate::db::internal::invitation::*;
use crate::db::logs::log_comment_body;
use crate::db::operations::models::*;
use crate::db::Pool;
use crate::mail::manager::send_email;
use crate::mail::templates::Template;
use crate::rules::engine::*;
use crate::rules::RuleContext;
use crate::user::User;
use chrono::NaiveDateTime;
use cis_client::AsyncCisClientTrait;
use dino_park_gate::scope::ScopeAndUser;
use dino_park_trust::Trust;
use failure::Error;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Queryable, Serialize)]
pub struct PendingInvitations {}

pub fn delete_invitation(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: User,
    member: User,
) -> Result<(), Error> {
    DELETE_INVITATION.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    let connection = pool.get()?;
    delete(&connection, group_name, host, member, None)?;
    let p = internal::user::slim_user_profile_by_uuid(&connection, &member.user_uuid)?;
    send_email(p.email, &Template::DeleteInvitation(group_name.to_owned()))?;
    Ok(())
}

pub fn reject_invitation(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    delete(&connection, group_name, User::default(), user, None)
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
    INVITE_MEMBER.run(&RuleContext::minimal_with_member_uuid(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
        &member.user_uuid,
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
    INVITE_MEMBER.run(&RuleContext::minimal_with_member_uuid(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
        &member.user_uuid,
    ))?;
    let connection = pool.get()?;
    // delete the pending request if it exists
    internal::request::delete(
        &connection,
        group_name,
        Some(host),
        &member,
        log_comment_body("invited"),
    )?;
    invite(
        &connection,
        group_name,
        host,
        member,
        invitation_expiration,
        group_expiration,
    )?;
    let p = internal::user::slim_user_profile_by_uuid(&connection, &member.user_uuid)?;
    send_email(p.email, &Template::Invitation(group_name.to_owned()))?;
    Ok(())
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
    match scope_and_user.scope {
        Trust::Staff => staff_scoped_invitations_and_host(&connection, group_name),
        Trust::Ndaed => ndaed_scoped_invitations_and_host(&connection, group_name),
        Trust::Vouched => vouched_scoped_invitations_and_host(&connection, group_name),
        Trust::Authenticated => authenticated_scoped_invitations_and_host(&connection, group_name),
        Trust::Public => public_scoped_invitations_and_host(&connection, group_name),
    }
}

pub fn pending_invitations_for_user(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
) -> Result<Vec<DisplayInvitationForUser>, Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    match scope_and_user.scope {
        Trust::Staff => staff_scoped_invitations_and_host_for_user(&connection, &user),
        Trust::Ndaed => ndaed_scoped_invitations_and_host_for_user(&connection, &user),
        Trust::Vouched => vouched_scoped_invitations_and_host_for_user(&connection, &user),
        Trust::Authenticated => {
            authenticated_scoped_invitations_and_host_for_user(&connection, &user)
        }
        Trust::Public => public_scoped_invitations_and_host_for_user(&connection, &user),
    }
}

pub async fn accept_invitation(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    user: &User,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    CURRENT_USER_CAN_JOIN.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &Uuid::default(),
    ))?;
    let connection = pool.get()?;
    let user_profile = internal::user::user_profile_by_uuid(&connection, &user.user_uuid)?;
    accept(&connection, group_name, user)?;
    add_group_to_profile(cis_client, group_name.to_owned(), user_profile.profile).await
}
