use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::user::User;
use chrono::NaiveDateTime;
use failure::Error;
use serde_derive::Serialize;
use dino_park_gate::scope::ScopeAndUser;
use crate::rules::engine::*;
use crate::rules::rules::RuleContext;
use crate::db::operations::internal::invitation;

#[derive(Queryable, Serialize)]
pub struct PendingInvitations {}

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
    INVITE_MEMBER.run(&RuleContext::minimal(pool, scope_and_user, &group_name, &host.user_uuid))?;
    invitation::invite(pool, group_name, host, member, invitation_expiration, group_expiration)
}

pub fn pending_invitations_count(pool: &Pool, scope_and_user: &ScopeAndUser, group_name: &str, host: &User) -> Result<i64, Error> {
    HOST_IS_CURATOR.run(&RuleContext::minimal(pool, scope_and_user, &group_name, &host.user_uuid))?;
    invitation::pending_count(pool, group_name)
}

pub fn accept_invitation(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    member: &User,
) -> Result<(), Error> {
    invitation::accept(pool, group_name, member)
}
