use crate::cis::operations::send_groups_to_cis;
use crate::db::internal;
use crate::db::Pool;
use crate::error::PacksError;
use crate::mail::manager::send_email;
use crate::mail::manager::subscribe_nda;
use crate::mail::templates::Template;
use crate::rules::engine::*;
use crate::rules::RuleContext;
use crate::user::User;
use cis_client::AsyncCisClientTrait;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use std::sync::Arc;

pub async fn add_admin(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    group_name: &str,
    host: &User,
    user: &User,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    CAN_ADD_CURATOR.run(&RuleContext::minimal_with_member_uuid(
        pool,
        scope_and_user,
        &group_name,
        &host.user_uuid,
        &user.user_uuid,
    ))?;
    let connection = pool.get()?;
    let user_profile = internal::user::slim_user_profile_by_uuid(&connection, &user.user_uuid)?;
    if group_name == "nda" {
        subscribe_nda(&user_profile.email)
    }
    internal::admin::add_admin(&connection, &group_name, host, user)?;
    drop(connection);
    send_groups_to_cis(pool, cis_client, &user.user_uuid).await
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
        .map(|_| ())?;
        let user = internal::user::slim_user_profile_by_uuid(&connection, &user.user_uuid)?;
        send_email(user.email, &Template::DemoteCurator(group_name.to_owned()));
        Ok(())
    } else {
        Err(PacksError::LastAdmin.into())
    }
}
