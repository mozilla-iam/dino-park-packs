use crate::cis::operations::remove_group_from_profile;
use crate::db::internal;
use crate::db::logs::log_comment_body;
use crate::db::model::Membership;
use crate::db::Pool;
use chrono::Utc;
use cis_client::CisClient;
use failure::Error;
use futures::future::try_join_all;
use futures::TryFutureExt;
use std::sync::Arc;
use uuid::Uuid;

async fn expire_membership(
    pool: &Pool,
    cis_client: Arc<CisClient>,
    membership: Membership,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = internal::group::get_group_by_id(&connection, membership.group_id)?;
    let user_profile = internal::user::user_profile_by_uuid(&connection, &membership.user_uuid)?;
    let user_uuid = user_profile.user_uuid;
    remove_group_from_profile(cis_client, &group.name, user_profile.profile).await?;
    let host_uuid = Uuid::default();
    internal::member::remove_from_group(
        &host_uuid,
        &connection,
        &user_uuid,
        &group.name,
        log_comment_body("expired"),
    )
}

pub async fn expire_memberships(pool: &Pool, cis_client: Arc<CisClient>) -> Result<(), Error> {
    let expires_before = Utc::now().naive_utc();
    let connection = pool.get()?;
    let memberships =
        internal::member::get_memberships_expired_before(&connection, expires_before)?;
    try_join_all(memberships.into_iter().map(|membership| {
        async {
            let cis_client = Arc::clone(&cis_client);
            let pool = pool.clone();
            expire_membership(&pool, cis_client, membership).await
        }
    }))
    .map_ok(|_| ())
    .await
}

pub fn expire_invitations(pool: &Pool) -> Result<(), Error> {
    let connection = pool.get()?;
    let expires_before = Utc::now().naive_utc();
    internal::invitation::expire_before(&connection, expires_before)?;
    Ok(())
}
