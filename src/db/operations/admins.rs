use crate::cis::operations::add_group_to_profile;
use crate::db::internal;
use crate::db::operations::error::OperationError;
use crate::db::Pool;
use crate::rules::engine::*;
use crate::rules::RuleContext;
use crate::user::User;
use cis_client::CisClient;
use cis_profile::schema::Profile;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use futures::future::IntoFuture;
use futures::Future;
use std::sync::Arc;

pub fn add_admin(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
    user: &User,
    cis_client: Arc<CisClient>,
    profile: Profile,
) -> impl Future<Item = (), Error = Error> {
    let group_name_f = group_name.to_owned();
    HOST_IS_GROUP_ADMIN
        .run(&RuleContext::minimal(
            pool,
            scope_and_user,
            &group_name,
            &host.user_uuid,
        ))
        .map_err(Into::into)
        .and_then(move |_| pool.get().map_err(Into::into))
        .and_then(move |connection| {
            internal::admin::add_admin(&connection, &group_name, host, user)
        })
        .into_future()
        .and_then(move |_| add_group_to_profile(cis_client, group_name_f, profile))
}

pub fn is_admin(pool: &Pool, scope_and_user: &ScopeAndUser, group_name: &str, host: &User) -> bool {
    HOST_IS_GROUP_ADMIN
        .run(&RuleContext::minimal(
            pool,
            scope_and_user,
            &group_name,
            &host.user_uuid,
        ))
        .is_ok()
}

pub fn demote(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
    user: &User,
    expiration: Option<i32>,
) -> Result<(), Error> {
    HOST_IS_GROUP_ADMIN.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    let connection = pool.get()?;
    if !internal::admin::is_last_admin(&connection, group_name, &user.user_uuid)? {
        internal::admin::demote_to_member(
            &host.user_uuid,
            &connection,
            group_name,
            user,
            expiration,
        )
        .map(|_| ())
    } else {
        Err(OperationError::LastAdmin.into())
    }
}
