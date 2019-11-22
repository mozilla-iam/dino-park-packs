use log::info;
use crate::rules::error::RuleError;
use crate::rules::rules::*;

pub const CREATE_GROUP: Engine = Engine {
    rules: &[&rule_is_creator],
};

pub const INVITE_MEMBER: Engine = Engine {
    rules: &[&rule_host_can_invite],
};

pub const HOST_IS_CURATOR: Engine = Engine {
    rules: &[&rule_host_is_curator],
};

pub const HOST_IS_GROUP_ADMIN: Engine = Engine {
    rules: &[&rule_host_is_group_admin],
};

pub struct Engine<'a> {
    pub rules: &'a [&'static Rule],
}

impl<'a> Engine<'a> {
    pub fn run(self: &Self, ctx: &RuleContext) -> Result<(), RuleError> {
        let ok = self.rules.iter().try_for_each(|rule| {
            rule(ctx)
        });
        if !ok.is_err() && ctx.scope_and_user.groups_scope.as_ref().map(|s| &**s) == Some("admin") {
            info!("using admin priviledges for {}", ctx.host_uuid);
            return Ok(());
        }
        ok
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db;
    use failure::Error;
    use dino_park_gate::scope::ScopeAndUser;
    use uuid::Uuid;

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
        let ok = engine.run(&ctx);
        assert!(ok.is_ok());
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
        let ok = engine.run(&ctx);
        assert_eq!(ok, Err::<(), _>(RuleError::NotAllowedToCreateGroups));
        Ok(())
    }
}
