pub mod engine;
pub mod error;
pub mod functions;

use crate::db::Pool;
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
    pub fn minimal_with_member_uuid(
        pool: &'a Pool,
        scope_and_user: &'a ScopeAndUser,
        group: &'a str,
        host_uuid: &'a Uuid,
        member_uuid: &'a Uuid,
    ) -> Self {
        RuleContext {
            pool,
            scope_and_user,
            group,
            host_uuid,
            host: None,
            member_uuid: Some(member_uuid),
            member: None,
        }
    }
}
