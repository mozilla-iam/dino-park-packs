use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::operations::internal;
use crate::db::schema;
use crate::db::types::*;
use crate::rules::engine::*;
use crate::rules::rules::RuleContext;
use diesel::prelude::*;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;

pub fn update_terms(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    text: String,
) -> Result<(), Error> {
    let host = internal::user::user_by_id(pool, &scope_and_user.user_id)?;
    EDIT_TERMS.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    internal::terms::set_terms(pool, group_name, text)
}

pub fn delete_terms(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
) -> Result<(), Error> {
    let host = internal::user::user_by_id(pool, &scope_and_user.user_id)?;
    EDIT_TERMS.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    internal::terms::delete_terms(pool, group_name)
}

pub use internal::terms::get_terms;
