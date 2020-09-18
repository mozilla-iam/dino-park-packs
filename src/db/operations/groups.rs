use crate::cis::operations::send_groups_to_cis;
use crate::db::internal;
use crate::db::logs::LogContext;
use crate::db::model::Group;
use crate::db::operations;
use crate::db::operations::models::GroupUpdate;
use crate::db::operations::models::GroupWithTermsFlag;
use crate::db::operations::models::NewGroup;
use crate::db::operations::models::PaginatedGroupsLists;
use crate::db::operations::models::SortGroupsBy;
use crate::db::types::TrustType;
use crate::db::Pool;
use crate::error::PacksError;
use crate::mail::manager::send_emails;
use crate::mail::templates::Template;
use crate::rules::engine::CREATE_GROUP;
use crate::rules::engine::HOST_IS_GROUP_ADMIN;
use crate::rules::engine::ONLY_ADMINS;
use crate::rules::RuleContext;
use crate::user::User;
use cis_client::AsyncCisClientTrait;
use diesel::pg::PgConnection;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
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
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    CREATE_GROUP.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &new_group.name,
        &user.user_uuid,
    ))?;
    add_new_group_db(&connection, new_group, user).map_err(|_| PacksError::GroupNameExists)?;
    drop(connection);
    send_groups_to_cis(pool, cis_client, &user.user_uuid).await
}

pub async fn delete_group(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    HOST_IS_GROUP_ADMIN.run(&RuleContext::minimal(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
    ))?;
    let bcc = internal::member::get_curator_emails_by_group_name(&connection, group_name)?;
    let members = internal::member::get_members_not_current(&connection, group_name, &host)?;
    drop(connection);
    operations::members::remove_members_silent(
        pool,
        scope_and_user,
        group_name,
        &members,
        Arc::clone(&cis_client),
    )
    .await?;
    operations::members::remove(
        &pool,
        &scope_and_user,
        &group_name,
        &host,
        &host,
        cis_client,
    )
    .await?;
    let connection = pool.get()?;
    internal::group::delete_group(&host.user_uuid, &connection, &group_name)?;
    let host_profile = internal::user::slim_user_profile_by_uuid(&connection, &host.user_uuid)?;
    send_emails(
        bcc,
        &Template::GroupDeleted(group_name.to_string(), host_profile.username),
    );
    Ok(())
}

pub async fn update_group_trust(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    trust: &TrustType,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let to_delete =
        internal::member::get_members_by_trust_less_than(&connection, group_name, trust)?;
    drop(connection);
    operations::members::remove_members_silent(
        pool,
        scope_and_user,
        group_name,
        &to_delete,
        cis_client,
    )
    .await?;
    let connection = pool.get()?;
    let host = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    internal::group::update_group_trust(&host.user_uuid, &connection, group_name, trust)?;
    Ok(())
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

pub fn list_inactive_groups(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    limit: i64,
    offset: i64,
) -> Result<Vec<Group>, Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    ONLY_ADMINS.run(&RuleContext::minimal(
        &pool.clone(),
        scope_and_user,
        "",
        &user.user_uuid,
    ))?;

    internal::group::inactive_groups(&connection, limit, offset)
}

pub fn delete_inactive_group(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    ONLY_ADMINS.run(&RuleContext::minimal(
        &pool.clone(),
        scope_and_user,
        group_name,
        &user.user_uuid,
    ))?;

    internal::group::delete_inactive_group(&connection, group_name)
}

pub fn reserve_group(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let user = internal::user::user_by_id(&connection, &scope_and_user.user_id)?;
    ONLY_ADMINS.run(&RuleContext::minimal(
        &pool.clone(),
        scope_and_user,
        group_name,
        &user.user_uuid,
    ))?;

    internal::group::reserve_group(&connection, &user.user_uuid, group_name)
}
