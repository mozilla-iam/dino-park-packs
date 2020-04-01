use crate::helpers::db::get_pool;
use crate::helpers::db::reset;
use dino_park_gate::scope::ScopeAndUser;
use dino_park_packs::rules::engine::*;
use dino_park_packs::rules::error::RuleError;
use dino_park_packs::rules::functions::*;
use dino_park_packs::rules::*;
use dino_park_trust::GroupsTrust;
use dino_park_trust::Trust;
use failure::Error;
use uuid::Uuid;

#[test]
fn simple_rule_stuct_creator_success() -> Result<(), Error> {
    reset()?;
    let pool = get_pool();
    let scope_and_user = ScopeAndUser {
        user_id: String::from("some_id"),
        scope: Trust::Staff,
        groups_scope: GroupsTrust::Admin,
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
    reset()?;
    let pool = get_pool();
    let scope_and_user = ScopeAndUser {
        user_id: String::from("some_id"),
        scope: Trust::Staff,
        groups_scope: GroupsTrust::None,
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
