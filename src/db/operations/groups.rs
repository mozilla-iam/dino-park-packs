use crate::db::db::Pool;
use crate::db::group::*;
use crate::db::operations::internal;
use crate::db::types::*;
use crate::user::User;
use failure::Error;
use uuid::Uuid;

pub fn add_new_group(
    pool: &Pool,
    name: String,
    description: String,
    creator: User,
    typ: GroupType,
) -> Result<(), Error> {
    let new_group = internal::group::add_group(pool, name, description, vec![], typ, None)?;
    internal::admin::add_admin_role(pool, new_group.id)?;
    internal::admin::add_admin(pool, new_group.id, creator.user_uuid, Uuid::nil())?;
    Ok(())
}

pub use internal::group::get_group;
pub use internal::group::update_group;
