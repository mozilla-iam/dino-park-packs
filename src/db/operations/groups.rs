use crate::db::db::Pool;
use crate::db::operations::internal;
use crate::db::types::*;
use crate::user::User;
use failure::Error;
use uuid::Uuid;
use crate::rules::engine::CREATE_GROUP;
use crate::rules::rules::RuleContext;
use dino_park_gate::scope::ScopeAndUser;

pub fn add_new_group(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    name: String,
    description: String,
    creator: User,
    typ: GroupType,
    trust: TrustType,
) -> Result<(), Error> {
    CREATE_GROUP.run(&RuleContext::minimal(pool, scope_and_user, &name, &creator.user_uuid))?;
    let new_group = internal::group::add_group(pool, name, description, vec![], typ, trust, None)?;
    internal::admin::add_admin_role(pool, new_group.id)?;
    internal::admin::add_admin(pool, new_group.id, creator.user_uuid, Uuid::nil())?;
    Ok(())
}

pub use internal::group::get_group;
pub use internal::group::update_group;
