use crate::cis::operations::add_group_to_profile;
use crate::db::internal;
use crate::db::logs::LogContext;
use crate::db::operations;
use crate::db::operations::error::OperationError;
use crate::db::operations::models::GroupUpdate;
use crate::db::operations::models::NewGroup;
use crate::db::Pool;
use crate::rules::engine::CREATE_GROUP;
use crate::rules::engine::HOST_IS_GROUP_ADMIN;
use crate::rules::RuleContext;
use crate::user::User;
use cis_client::CisClient;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use futures::future::join_all;
use futures::future::IntoFuture;
use futures::Future;
use std::convert::TryFrom;
use std::sync::Arc;

fn add_new_group_db(pool: &Pool, new_group: NewGroup, creator: User) -> Result<(), Error> {
    let new_group = internal::group::add_group(&creator.user_uuid, pool, new_group)?;
    let log_ctx = LogContext::with(new_group.id, creator.user_uuid);
    internal::admin::add_admin_role(&log_ctx, pool, new_group.id)?;
    internal::member::add_member_role(&creator.user_uuid, pool, new_group.id)?;
    internal::admin::add_admin(pool, &new_group.name, &User::default(), &creator)?;
    Ok(())
}

pub fn add_new_group(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    new_group: NewGroup,
    cis_client: Arc<CisClient>,
) -> impl Future<Item = (), Error = Error> {
    let group_name_f = new_group.name.clone();
    internal::user::user_profile_by_user_id(&pool, &scope_and_user.user_id)
        .and_then(|user_profile| {
            User::try_from(&user_profile.profile).map(|user| (user, user_profile))
        })
        .and_then(|(user, user_profile)| {
            CREATE_GROUP
                .run(&RuleContext::minimal(
                    pool,
                    scope_and_user,
                    &new_group.name,
                    &user.user_uuid,
                ))
                .map(|_| (user, user_profile))
                .map_err(Into::into)
        })
        .and_then(|(user, user_profile)| {
            add_new_group_db(pool, new_group, user).map(|_| user_profile)
        })
        .into_future()
        .and_then(move |user_profile| {
            add_group_to_profile(cis_client, group_name_f, user_profile.profile)
        })
}

pub fn delete_group(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    name: &str,
    cis_client: Arc<CisClient>,
) -> impl Future<Item = (), Error = Error> {
    // TODO: clean up and reserve group name
    let group_name_f = name.to_owned();
    let group_name_ff = name.to_owned();
    let group_name_fff = name.to_owned();
    let pool_f = pool.clone();
    let pool_ff = pool.clone();
    let pool_fff = pool.clone();
    let scope_and_user_f = scope_and_user.clone();
    let scope_and_user_ff = scope_and_user.clone();
    let cis_client_f = Arc::clone(&cis_client);
    internal::user::user_by_id(pool, &scope_and_user.user_id)
        .and_then(|host| {
            HOST_IS_GROUP_ADMIN
                .run(&RuleContext::minimal(
                    pool,
                    scope_and_user,
                    &name,
                    &host.user_uuid,
                ))
                .map_err(Into::into)
                .map(|_| host)
        })
        .and_then(move |host| {
            internal::member::get_members_not_current(pool, name, &host)
                .map(|members| (host, members))
        })
        .into_future()
        .and_then(move |(host, members)| {
            let v = members
                .iter()
                .map(|user| {
                    operations::members::remove(
                        &pool_f,
                        &scope_and_user_f,
                        &group_name_f,
                        &host,
                        &user,
                        Arc::clone(&cis_client),
                    )
                })
                .collect::<Vec<_>>();
            log::info!("deleting {} members", v.len());
            join_all(v)
                .map_err(|_| OperationError::ErrorDeletingMembers.into())
                .map(move |_| host)
        })
        .and_then(move |host| {
            operations::members::remove(
                &pool_ff,
                &scope_and_user_ff,
                &group_name_ff,
                &host,
                &host,
                cis_client_f,
            )
            .map(move |_| host)
        })
        .and_then(move |host| {
            internal::group::delete_group(&host.user_uuid, &pool_fff, &group_name_fff).into_future()
        })
}

pub fn update_group(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: String,
    group_update: GroupUpdate,
) -> Result<(), Error> {
    let host = internal::user::user_by_id(pool, &scope_and_user.user_id)?;
    HOST_IS_GROUP_ADMIN.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    internal::group::update_group(&host.user_uuid, pool, group_name.to_owned(), group_update)
        .map(|_| ())
        .map_err(Into::into)
}

pub use internal::group::get_group;
pub use internal::group::get_group_with_terms_flag;
