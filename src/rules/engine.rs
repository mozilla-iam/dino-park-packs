use crate::db::db::Pool;
use crate::db::operations;
use crate::db::types::*;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use uuid::Uuid;

const CREATE_GROUP: Engine = Engine {
    rules: &[&rule_is_creator],
};

pub struct Engine<'a> {
    pub rules: &'a [&'static Rule],
}

impl<'a> Engine<'a> {
    pub fn run(self: &Self, ctx: &RuleContext) -> Result<bool, Error> {
        self.rules.iter().try_fold(true, |mut ok, rule| {
            ok &= rule(ctx)?;
            Ok(ok)
        })
    }
}

pub struct RuleContext<'a> {
    pub pool: &'a Pool,
    pub scope_and_user: &'a ScopeAndUser,
    pub group: &'a str,
    pub host_uuid: &'a Uuid,
    pub host: Option<&'a Profile>,
    pub member_uuid: Option<&'a Uuid>,
    pub member: Option<&'a Profile>,
}

type Rule = Fn(&RuleContext) -> Result<bool, Error>;

/// Check if curent user is allowed to create groups.
pub fn rule_is_creator(ctx: &RuleContext) -> Result<bool, Error> {
    match ctx.scope_and_user.groups_scope.as_ref().map(|s| &**s) {
        Some("creator") | Some("admin") => Ok(true),
        _ => Ok(false),
    }
}

/// Check if the host is either `RoleTpye::Admin` or has `InviiteMember` permissions for the given
/// group.
pub fn rule_host_can_invite(ctx: &RuleContext) -> Result<bool, Error> {
    match operations::members::member_role(ctx.pool, ctx.host_uuid, ctx.group) {
        Ok(role) => {
            Ok(role.typ == RoleType::Admin
                || role.permissions.contains(&PermissionType::InviteMember))
        }
        _ => Ok(false),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db;

    #[test]
    fn simple_rule_stuct_creator_success() -> Result<(), Error> {
        let pool = db::db::establish_connection();
        let scope_and_user = ScopeAndUser {
            user_id: String::from("some_id"),
            scope: String::from("staff"),
            groups_scope: Some(String::from("admin")),
        };
        let ctx = RuleContext {
            pool: &pool,
            scope_and_user: &scope_and_user,
            group: "test",
            host_uuid: &Uuid::nil(),
            host: None,
            member_uuid: None,
            member: None,
        };
        let engine = Engine {
            rules: &[&rule_is_creator],
        };
        let ok = engine.run(&ctx)?;
        assert!(ok);
        Ok(())
    }

    #[test]
    fn simple_rule_stuct_creator_fail() -> Result<(), Error> {
        let pool = db::db::establish_connection();
        let scope_and_user = ScopeAndUser {
            user_id: String::from("some_id"),
            scope: String::from("staff"),
            groups_scope: None,
        };
        let ctx = RuleContext {
            pool: &pool,
            scope_and_user: &scope_and_user,
            group: "test",
            host_uuid: &Uuid::nil(),
            host: None,
            member_uuid: None,
            member: None,
        };
        let engine = Engine {
            rules: &[&rule_is_creator],
        };
        let ok = engine.run(&ctx)?;
        assert_eq!(ok, false);
        Ok(())
    }
}
