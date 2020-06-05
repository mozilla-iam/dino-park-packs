use crate::db::internal;
use crate::db::logs::log_comment_body;
use crate::db::model::Membership;
use crate::db::operations::members::revoke_membership;
use crate::db::types::RoleType;
use crate::db::Pool;
use crate::error::PacksError;
use crate::mail::manager::send_email;
use crate::mail::manager::send_emails;
use crate::mail::templates::Template;
use crate::user::User;
use chrono::Duration;
use chrono::Utc;
use cis_client::AsyncCisClientTrait;
use failure::Error;
use futures::future::try_join_all;
use futures::TryFutureExt;
use log::error;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

async fn expire_membership(
    pool: &Pool,
    cis_client: Arc<impl AsyncCisClientTrait>,
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

pub async fn expire_memberships(
    pool: &Pool,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
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
                h.insert(m.user_uuid, vec![m]);
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

pub fn expiration_notification(pool: &Pool, first: bool) -> Result<usize, Error> {
    let days = if first { 14 } else { 7 };
    let lower = Utc::now()
        .checked_add_signed(Duration::days(days))
        .unwrap()
        .date()
        .and_hms(0, 0, 0)
        .naive_utc();
    let upper = Utc::now()
        .checked_add_signed(Duration::days(days))
        .unwrap()
        .date()
        .and_hms_nano(23, 59, 59, 999_999_999)
        .naive_utc();
    let connection = pool.get()?;
    let memberships = internal::member::get_memberships_expire_between(&connection, lower, upper)?;
    info!(
        "{} memberships expiring in {} days ({}-{})",
        memberships.len(),
        days,
        lower,
        upper
    );
    let mut count = 0;
    for membership in memberships {
        let group = internal::group::get_group_by_id(&connection, membership.group_id)?
            .ok_or(PacksError::InvalidGroupData)?;
        let host = internal::user::slim_user_profile_by_uuid(&connection, &membership.added_by)?;
        let host_valid =
            match internal::member::role_for(&connection, &host.user_uuid, &group.name)? {
                Some(r) => r.typ != RoleType::Member,
                None => false,
            };
        let user = internal::user::slim_user_profile_by_uuid(&connection, &membership.added_by)?;
        let to = if host_valid {
            vec![host.email]
        } else {
            internal::member::get_curator_emails(&connection, group.id)?
        };
        if first {
            send_emails(
                to,
                &Template::FirstHostExpiration(group.name, user.username),
            );
        } else {
            send_emails(
                to,
                &Template::SecondHostExpiration(group.name.clone(), user.username),
            );
            if let Err(e) = send_email(user.email, &Template::MemberExpiration(group.name)) {
                error!(
                    "unable to send expiration email to {}: {}",
                    user.user_uuid, e
                );
            }
        }
        count += 1;
    }
    Ok(count)
}

pub fn expire_invitations(pool: &Pool) -> Result<(), Error> {
    let connection = pool.get()?;
    let expires_before = Utc::now().naive_utc();
    internal::invitation::expire_before(&connection, expires_before)?;
    Ok(())
}
