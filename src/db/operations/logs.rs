use crate::db::internal;
use crate::db::logs::Log;
use crate::db::Pool;
use crate::rules::engine::ONLY_ADMINS;
use crate::rules::RuleContext;
use crate::user::User;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;

pub fn raw_logs(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    host: &User,
) -> Result<Vec<Log>, Error> {
    ONLY_ADMINS.run(&RuleContext::minimal(
        &pool.clone(),
        scope_and_user,
        "",
        &host.user_uuid,
    ))?;
    let connection = pool.get()?;
    internal::log::raw_logs(&connection)
}
