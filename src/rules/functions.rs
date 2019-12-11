use crate::db::operations;
use crate::db::types::*;
use crate::rules::error::RuleError;
use crate::rules::RuleContext;

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
    match operations::members::role_for(ctx.pool, ctx.host_uuid, ctx.group) {
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
    match operations::members::role_for(ctx.pool, ctx.host_uuid, ctx.group) {
        Ok(role)
            if role.typ == RoleType::Admin
                || role.permissions.contains(&PermissionType::RemoveMember) =>
        {
            Ok(())
        }
        _ => Err(RuleError::NotAllowedToRemoveMember),
    }
}

/// Check if the host is either `RoleType::Admin` of `RoleType::Curator`
pub fn rule_host_is_curator(ctx: &RuleContext) -> Result<(), RuleError> {
    match operations::members::role_for(ctx.pool, ctx.host_uuid, ctx.group) {
        Ok(role) if role.typ == RoleType::Admin || role.typ == RoleType::Curator => Ok(()),
        _ => Err(RuleError::NotACurator),
    }
}

/// Check if the host is either `RoleTpye::Admin` for the given group
pub fn rule_host_is_group_admin(ctx: &RuleContext) -> Result<(), RuleError> {
    match operations::members::role_for(ctx.pool, ctx.host_uuid, ctx.group) {
        Ok(role) if role.typ == RoleType::Admin => Ok(()),
        _ => Err(RuleError::NotAnAdmin),
    }
}

/// Check if the host is either `RoleTpye::Admin` for the given group
pub fn rule_user_has_member_role(ctx: &RuleContext) -> Result<(), RuleError> {
    let member_uuid = ctx.member_uuid.ok_or(RuleError::InvalidRuleContext)?;
    match operations::members::role_for(ctx.pool, member_uuid, ctx.group) {
        Ok(role) if role.typ == RoleType::Member => Ok(()),
        _ => Err(RuleError::NotAnAdmin),
    }
}

/// Check if the host is either `RoleTpye::Admin` or has `EditTerms` permissions for the given
/// group.
pub fn rule_host_can_edit_terms(ctx: &RuleContext) -> Result<(), RuleError> {
    match operations::members::role_for(ctx.pool, ctx.host_uuid, ctx.group) {
        Ok(role)
            if role.typ == RoleType::Admin
                || role.permissions.contains(&PermissionType::EditTerms) =>
        {
            Ok(())
        }
        _ => Err(RuleError::NotAllowedToEditTerms),
    }
}
