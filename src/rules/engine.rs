use crate::db::db::Pool;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;

pub struct RuleContext<'a> {
    pub pool: &'a Pool,
    pub scope_and_user: &'a ScopeAndUser,
    pub group: &'a str,
    pub host: &'a Profile,
    pub member: &'a Profile,
}

pub trait Rule {
    fn run(ctx: &RuleContext) -> Result<bool, Error>;
}

pub struct Creator {}

impl Rule for Creator {
    fn run(ctx: &RuleContext) -> Result<bool, Error> {
        match ctx.scope_and_user.groups_scope.as_ref().map(|s| &**s) {
            Some("creator") | Some("admin") => Ok(true),
            _ => Ok(false),
        }
    }
}

pub struct AddMember {}

impl Rule for AddMember {
    fn run(ctx: &RuleContext) -> Result<bool, Error> {
        Ok(false)
    }
}
