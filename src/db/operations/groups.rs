use crate::cis::operations::add_group_to_profile;
use crate::db::internal;
use crate::db::logs::LogContext;
use crate::db::model::Group;
use crate::db::operations;
use crate::db::operations::models::GroupUpdate;
use crate::db::operations::models::GroupWithTermsFlag;
use crate::db::operations::models::NewGroup;
use crate::db::operations::models::PaginatedGroupsLists;
use crate::db::operations::models::SortGroupsBy;
use crate::db::Pool;
use crate::error::PacksError;
use crate::rules::engine::CREATE_GROUP;
use crate::rules::engine::HOST_IS_GROUP_ADMIN;
use crate::rules::RuleContext;
use crate::user::User;
use cis_client::CisClient;
use diesel::pg::PgConnection;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use futures::future::try_join_all;
use futures::TryFutureExt;
use std::convert::TryFrom;
use std::sync::Arc;

fn add_new_group_db(
    connection: &PgConnection,
    new_group: NewGroup,
    creator: User,
) -> Result<(), Error> {
    let new_group = internal::group::add_group(&creator.user_uuid, &connection, new_group)?;
    let log_ctx = LogContext::with(new_group.id, creator.user_uuid);
    internal::admin::add_admin_role(&log_ctx, &connection, new_group.id)?;
    internal::member::add_member_role(&creator.user_uuid, &connection, new_group.id)?;
    internal::admin::add_admin(&connection, &new_group.name, &User::default(), &creator)?;
    Ok(())
}

pub async fn add_new_group(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    new_group: NewGroup,
    cis_client: Arc<CisClient>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let user_profile =
        internal::user::user_profile_by_user_id(&connection, &scope_and_user.user_id)?;
    let user = User::try_from(&user_profile.profile)?;
    CREATE_GROUP.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &new_group.name,
        &user.user_uuid,
    ))?;
    let new_group_name = new_group.name.clone();
    add_new_group_db(&connection, new_group, user).map_err(|_| PacksError::GroupNameExists)?;
    add_group_to_profile(cis_client, new_group_name, user_profile.profile).await
}

pub async fn delete_group(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    name: &str,
    cis_client: Arc<CisClient>,
) -> Result<(), Error> {
    // TODO: clean up and reserve group name
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    HOST_IS_GROUP_ADMIN.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &name,
        &host.user_uuid,
    ))?;
    let members = internal::member::get_members_not_current(&connection, name, &host)?;
    let v = members
        .iter()
        .map(|user| {
            operations::members::remove(
                &pool,
                &scope_and_user,
                &name,
                &host,
                &user,
                Arc::clone(&cis_client),
            )
        })
        .collect::<Vec<_>>();
    log::info!("deleting {} members", v.len());
    try_join_all(v)
        .map_err(|_| PacksError::ErrorDeletingMembers)
        .await?;
    operations::members::remove(&pool, &scope_and_user, &name, &host, &host, cis_client).await?;
    internal::group::delete_group(&host.user_uuid, &connection, &name)
}

pub fn update_group(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: String,
    group_update: GroupUpdate,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    HOST_IS_GROUP_ADMIN.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    internal::group::update_group(&host.user_uuid, &connection, group_name, group_update)
        .map(|_| ())
        .map_err(Into::into)
}

pub fn get_group(pool: &Pool, group_name: &str) -> Result<Group, Error> {
    let connection = pool.get()?;
    internal::group::get_group(&connection, group_name)
}

pub fn get_group_with_terms_flag(
    pool: &Pool,
    group_name: &str,
) -> Result<GroupWithTermsFlag, Error> {
    let connection = pool.get()?;
    internal::group::get_group_with_terms_flag(&connection, group_name)
}

pub fn list_groups(
    pool: &Pool,
    filter: Option<String>,
    sort_by: SortGroupsBy,
    limit: i64,
    offset: i64,
) -> Result<PaginatedGroupsLists, Error> {
    let connection = pool.get()?;
    internal::group::list_groups(&connection, filter, sort_by, limit, offset)
}
