use crate::db::db::Pool;
use crate::db::operations;
use crate::db::types::*;
use crate::rules::error::RuleError;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use uuid::Uuid;

pub struct RuleContext<'a> {
    pub pool: &'a Pool,
    pub scope_and_user: &'a ScopeAndUser,
    pub group: &'a str,
    pub host_uuid: &'a Uuid,
    pub host: Option<&'a Profile>,
    pub member_uuid: Option<&'a Uuid>,
    pub member: Option<&'a Profile>,
}

impl<'a> RuleContext<'a> {
    pub fn minimal(
        pool: &'a Pool,
        scope_and_user: &'a ScopeAndUser,
        group: &'a str,
        host_uuid: &'a Uuid,
    ) -> Self {
        RuleContext {
            pool,
            scope_and_user,
            group,
            host_uuid,
            host: None,
            member_uuid: None,
            member: None,
        }
    }
}

pub type Rule = Fn(&RuleContext) -> Result<(), RuleError>;

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
