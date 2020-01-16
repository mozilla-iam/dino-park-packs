use crate::cis::operations::remove_group_from_profile;
use crate::db::internal;
use crate::db::logs::log_comment_body;
use crate::db::model::Membership;
use crate::db::Pool;
use chrono::Utc;
use cis_client::CisClient;
use failure::Error;
use futures::future::join_all;
use futures::future::IntoFuture;
use futures::Future;
use std::sync::Arc;
use uuid::Uuid;

fn expire_membership(
    pool: &Pool,
    cis_client: Arc<CisClient>,
    membership: Membership,
) -> impl Future<Item = (), Error = Error> {
    pool.get()
        .map_err(Error::from)
        .and_then(|connection| {
            internal::group::get_group_by_id(&connection, membership.group_id)
                .map(|group| (connection, group))
        })
        .and_then(|(connection, group)| {
            internal::user::user_profile_by_uuid(&connection, &membership.user_uuid)
                .map(move |user_profile| (connection, group, user_profile))
        })
        .into_future()
        .and_then(move |(connection, group, user_profile)| {
            let user_uuid = user_profile.user_uuid;
            remove_group_from_profile(cis_client, &group.name, user_profile.profile)
                .map(move |_| (connection, group, user_uuid))
        })
        .and_then(move |(connection, group, user_uuid)| {
            let host_uuid = Uuid::default();
            internal::member::remove_from_group(
                &host_uuid,
                &connection,
                &user_uuid,
                &group.name,
                log_comment_body("expired"),
            )
            .into_future()
        })
}
pub fn expire_memberships(
    pool: &Pool,
    cis_client: Arc<CisClient>,
) -> impl Future<Item = (), Error = Error> {
    let expires_before = Utc::now().naive_utc();
    let pool_f = pool.clone();
    pool.get()
        .map_err(Error::from)
        .and_then(|connection| {
            internal::member::get_memberships_expired_before(&connection, expires_before)
        })
        .into_future()
        .and_then(move |memberships| {
            join_all(memberships.into_iter().map(move |membership| {
                let cis_client = Arc::clone(&cis_client);
                let pool = pool_f.clone();
                expire_membership(&pool, cis_client, membership)
            }))
            .map(|_| ())
        })
}

pub fn expire_invitations(pool: &Pool) -> Result<(), Error> {
    let connection = pool.get()?;
    let expires_before = Utc::now().naive_utc();
    internal::invitation::expire_before(&connection, expires_before)?;
    Ok(())
}
