use crate::cis::operations::add_group_to_profile;
use crate::db::db::Pool;
use crate::db::operations::internal;
use crate::db::types::*;
use crate::rules::engine::CREATE_GROUP;
use crate::rules::rules::RuleContext;
use crate::user::User;
use cis_client::CisClient;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use futures::future::IntoFuture;
use futures::Future;
use std::sync::Arc;

fn add_new_group_db(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    name: String,
    description: String,
    creator: User,
    typ: GroupType,
    trust: TrustType,
    expiration: Option<i32>,
) -> Result<(), Error> {
    CREATE_GROUP.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &name,
        &creator.user_uuid,
    ))?;
    let new_group =
        internal::group::add_group(pool, name, description, vec![], typ, trust, expiration)?;
    internal::admin::add_admin_role(pool, new_group.id)?;
    internal::member::add_member_role(pool, new_group.id)?;
    internal::admin::add_admin(pool, &new_group.name, &User::default(), &creator)?;
    Ok(())
}

pub fn add_new_group(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    name: String,
    description: String,
    creator: User,
    typ: GroupType,
    trust: TrustType,
    expiration: Option<i32>,
    cis_client: Arc<CisClient>,
    profile: Profile,
) -> impl Future<Item = (), Error = Error> {
    let group_name_f = name.clone();
    add_new_group_db(
        pool,
        scope_and_user,
        name,
        description,
        creator,
        typ,
        trust,
        expiration,
    )
    .into_future()
    .and_then(move |_| add_group_to_profile(cis_client, group_name_f, profile))
}

pub use internal::group::get_group;
pub use internal::group::get_group_with_terms_flag;
pub use internal::group::update_group;
