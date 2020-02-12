use crate::db::internal;
use crate::db::types::*;
use crate::rules::error::RuleError;
use crate::rules::RuleContext;
use cis_profile::schema::Display;
use diesel::result::Error as DieselError;
use std::convert::TryFrom;

const NDA_GROUPS: [&str; 2] = ["nda", "contingentworkernda"];

pub fn is_nda_group(group_name: &str) -> bool {
    NDA_GROUPS.contains(&group_name)
}

pub type Rule = dyn Fn(&RuleContext) -> Result<(), RuleError>;

/// Always fails, caught by admin override.
pub fn rule_only_admins(_: &RuleContext) -> Result<(), RuleError> {
    Err(RuleError::NeverAllowed)
}

/// Check if curent user is allowed to create groups.
pub fn rule_is_creator(ctx: &RuleContext) -> Result<(), RuleError> {
    match ctx.scope_and_user.groups_scope.as_ref().map(|s| &**s) {
        Some("creator") | Some("admin") => Ok(()),
        _ => Err(RuleError::NotAllowedToCreateGroups),
    }
}

/// Check if the host is either `RoleTpye::Admin` or has `InviteMember` permissions for the given
/// group.
pub fn rule_host_can_invite(ctx: &RuleContext) -> Result<(), RuleError> {
    let connection = ctx.pool.get().map_err(|_| RuleError::PoolError)?;
    match internal::member::role_for(&connection, ctx.host_uuid, ctx.group) {
        Ok(role)
            if role.typ == RoleType::Admin
                || role.permissions.contains(&PermissionType::InviteMember) =>
        {
            Ok(())
        }
        _ => Err(RuleError::NotAllowedToInviteMember),
    }
}

/// Check if the host is either `RoleTpye::Admin` or has `RemoveMember` permissions for the given
/// group.
pub fn rule_host_can_remove(ctx: &RuleContext) -> Result<(), RuleError> {
    let connection = ctx.pool.get().map_err(|_| RuleError::PoolError)?;
    match internal::member::role_for(&connection, ctx.host_uuid, ctx.group) {
        Ok(role)
            if role.typ == RoleType::Admin
                || role.permissions.contains(&PermissionType::RemoveMember) =>
        {
            Ok(())
        }
        _ => Err(RuleError::NotAllowedToRemoveMember),
    }
}

pub fn user_not_a_member(ctx: &RuleContext) -> Result<(), RuleError> {
    let member_uuid = ctx.member_uuid.ok_or(RuleError::InvalidRuleContext)?;
    let connection = ctx.pool.get().map_err(|_| RuleError::PoolError)?;
    match internal::member::role_for(&connection, member_uuid, ctx.group) {
        Ok(_) => Err(RuleError::AlreadyMember),
        Err(e) => {
            if let Some(DieselError::NotFound) = e.downcast_ref::<DieselError>() {
                Ok(())
            } else {
                Err(RuleError::DBError)
            }
        }
    }
}

/// Check if the member is nda'd
pub fn member_is_ndaed(ctx: &RuleContext) -> Result<(), RuleError> {
    let connection = ctx.pool.get().map_err(|_| RuleError::PoolError)?;
    let trust = internal::user::user_trust(
        &connection,
        ctx.member_uuid.ok_or(RuleError::InvalidRuleContext)?,
    )
    .map_err(|_| RuleError::UserNotFound)?;
    if trust >= TrustType::Ndaed {
        return Ok(());
    }
    Err(RuleError::NotAllowedToJoinGroup)
}

/// Check if the user is nda'd or the group is the nda group
pub fn member_can_join(ctx: &RuleContext) -> Result<(), RuleError> {
    let connection = ctx.pool.get().map_err(|_| RuleError::PoolError)?;
    let trust = internal::user::user_trust(
        &connection,
        ctx.member_uuid.ok_or(RuleError::InvalidRuleContext)?,
    )
    .map_err(|_| RuleError::UserNotFound)?;
    let ndaed = trust >= TrustType::Ndaed;
    if ndaed | is_nda_group(&ctx.group) {
        return Ok(());
    }
    Err(RuleError::NotAllowedToJoinGroup)
}

/// Check if the current user is nda'd or the group is the nda group
pub fn current_user_can_join(ctx: &RuleContext) -> Result<(), RuleError> {
    let ndaed = Display::try_from(ctx.scope_and_user.scope.as_str())
        .map_err(|_| RuleError::InvalidScope)?
        >= Display::Ndaed;
    if ndaed | is_nda_group(&ctx.group) {
        return Ok(());
    }
    Err(RuleError::NotAllowedToJoinGroup)
}

/// Check if the host is either `RoleType::Admin` of `RoleType::Curator`
pub fn rule_host_is_curator(ctx: &RuleContext) -> Result<(), RuleError> {
    let connection = ctx.pool.get().map_err(|_| RuleError::PoolError)?;
    match internal::member::role_for(&connection, ctx.host_uuid, ctx.group) {
        Ok(role) if role.typ == RoleType::Admin || role.typ == RoleType::Curator => Ok(()),
        _ => Err(RuleError::NotACurator),
    }
}

/// Check if the host is either `RoleTpye::Admin` for the given group
pub fn rule_host_is_group_admin(ctx: &RuleContext) -> Result<(), RuleError> {
    let connection = ctx.pool.get().map_err(|_| RuleError::PoolError)?;
    match internal::member::role_for(&connection, ctx.host_uuid, ctx.group) {
        Ok(role) if role.typ == RoleType::Admin => Ok(()),
        _ => Err(RuleError::NotAnAdmin),
    }
}

/// Check if the host is either `RoleTpye::Admin` for the given group
pub fn rule_user_has_member_role(ctx: &RuleContext) -> Result<(), RuleError> {
    let member_uuid = ctx.member_uuid.ok_or(RuleError::InvalidRuleContext)?;
    let connection = ctx.pool.get().map_err(|_| RuleError::PoolError)?;
    match internal::member::role_for(&connection, member_uuid, ctx.group) {
        Ok(role) if role.typ == RoleType::Member => Ok(()),
        _ => Err(RuleError::NotAnAdmin),
    }
}

/// Check if the host is either `RoleTpye::Admin` or has `EditTerms` permissions for the given
/// group.
pub fn rule_host_can_edit_terms(ctx: &RuleContext) -> Result<(), RuleError> {
    let connection = ctx.pool.get().map_err(|_| RuleError::PoolError)?;
    match internal::member::role_for(&connection, ctx.host_uuid, ctx.group) {
        Ok(role)
            if role.typ == RoleType::Admin
                || role.permissions.contains(&PermissionType::EditTerms) =>
        {
            Ok(())
        }
        _ => Err(RuleError::NotAllowedToEditTerms),
    }
}
