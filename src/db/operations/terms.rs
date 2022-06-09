use crate::db::internal;
use crate::db::Pool;
use crate::rules::engine::*;
use crate::rules::RuleContext;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;

pub fn update_terms(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    text: String,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    EDIT_TERMS.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        group_name,
        &host.user_uuid,
    ))?;
    internal::terms::set_terms(&host.user_uuid, &connection, group_name, text)
}

pub fn delete_terms(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    EDIT_TERMS.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        group_name,
        &host.user_uuid,
    ))?;
    internal::terms::delete_terms(&host.user_uuid, &connection, group_name)
}

pub fn get_terms(pool: &Pool, group_name: &str) -> Result<Option<String>, Error> {
    let connection = pool.get()?;
    internal::terms::get_terms(&connection, group_name)
}
