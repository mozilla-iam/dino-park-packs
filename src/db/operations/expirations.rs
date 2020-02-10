use crate::db::internal;
use crate::db::logs::log_comment_body;
use crate::db::model::Membership;
use crate::db::operations::members::revoke_membership;
use crate::db::Pool;
use crate::user::User;
use chrono::Utc;
use cis_client::CisClient;
use failure::Error;
use futures::future::try_join_all;
use futures::TryFutureExt;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

async fn expire_membership(
    pool: &Pool,
    cis_client: Arc<CisClient>,
    user: &User,
    memberships: Vec<Membership>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let groups = internal::group::get_groups_by_ids(
        &connection,
        &memberships.iter().map(|m| m.group_id).collect::<Vec<i32>>(),
    )?;
    let group_names = groups.iter().map(|g| g.name.as_str()).collect::<Vec<_>>();
    let host = User::default();
    revoke_membership(
        pool,
        group_names.as_slice(),
        &host,
        &user,
        true,
        cis_client,
        log_comment_body("expired"),
    )
    .await
}

pub async fn expire_memberships(pool: &Pool, cis_client: Arc<CisClient>) -> Result<(), Error> {
    let expires_before = Utc::now().naive_utc();
    let connection = pool.get()?;
    let memberships =
        internal::member::get_memberships_expired_before(&connection, expires_before)?;
    let memberships = memberships.into_iter().fold(
        HashMap::new(),
        |mut h: HashMap<Uuid, Vec<Membership>>, m| {
            if let Some(v) = h.get_mut(&m.user_uuid) {
                v.push(m);
            } else {
                h.insert(m.user_uuid.clone(), vec![m]);
            }
            h
        },
    );
    try_join_all(memberships.into_iter().map(|(user_uuid, memberships)| {
        let user = User { user_uuid };
        let cis_client = Arc::clone(&cis_client);
        async move {
            let pool = pool.clone();
            expire_membership(&pool, cis_client, &user, memberships).await
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
