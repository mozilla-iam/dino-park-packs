use crate::cis::operations::add_group_to_profile;
use crate::db::internal;
use crate::db::logs::LogContext;
use crate::db::operations::models::NewGroup;
use crate::db::types::*;
use crate::db::users::UserProfile;
use crate::db::Pool;
use crate::import::tsv::MozilliansGroup;
use crate::import::tsv::MozilliansGroupCurator;
use crate::import::tsv::MozilliansGroupMembership;
use crate::user::User;
use cis_client::getby::GetBy;
use cis_client::AsyncCisClientTrait;
use diesel::pg::PgConnection;
use failure::Error;
use log::warn;
use std::convert::TryInto;
use std::sync::Arc;

pub struct GroupImport {
    pub group: MozilliansGroup,
    pub memberships: Vec<MozilliansGroupMembership>,
    pub curators: Vec<MozilliansGroupCurator>,
}

pub fn import_group(connection: &PgConnection, moz_group: MozilliansGroup) -> Result<(), Error> {
    let new_group = NewGroup {
        name: moz_group.name,
        typ: if moz_group.typ == "Reviewed" {
            GroupType::Reviewed
        } else {
            GroupType::Closed
        },
        description: moz_group.description,
        trust: TrustType::Ndaed,
        capabilities: Default::default(),
        group_expiration: Some(moz_group.expiration),
    };
    let creator = User::default();
    let new_group = internal::group::add_group(&creator.user_uuid, &connection, new_group)?;
    let log_ctx = LogContext::with(new_group.id, creator.user_uuid);
    internal::admin::add_admin_role(&log_ctx, &connection, new_group.id)?;
    internal::member::add_member_role(&creator.user_uuid, &connection, new_group.id)?;
    Ok(())
}

async fn get_user_profile(
    connection: &PgConnection,
    user_id: &str,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<UserProfile, Error> {
    if let Ok(user_profile) = internal::user::user_profile_by_user_id(&connection, user_id) {
        Ok(user_profile)
    } else {
        cis_client
            .clone()
            .get_user_by(user_id, &GetBy::UserId, None)
            .await
            .and_then(|p| p.try_into())
    }
}

async fn import_curator(
    connection: &PgConnection,
    group_name: &str,
    curator: MozilliansGroupCurator,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let user_profile =
        get_user_profile(connection, &curator.auth0_user_id, cis_client.clone()).await?;
    let user = User {
        user_uuid: user_profile.user_uuid,
    };
    internal::admin::add_admin(&connection, group_name, &User::default(), &user)?;
    add_group_to_profile(cis_client, group_name.to_owned(), user_profile.profile).await?;
    Ok(())
}

pub async fn import_curators(
    connection: &PgConnection,
    group_name: &str,
    moz_curators: Vec<MozilliansGroupCurator>,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    for curator in moz_curators {
        let user_id = curator.auth0_user_id.clone();
        match import_curator(connection, group_name, curator, cis_client.clone()).await {
            Ok(()) => {}
            Err(e) => warn!(
                "unable to add curator {} for group {}: {}",
                &user_id, group_name, e
            ),
        }
    }
    Ok(())
}

pub async fn import_member(
    connection: &PgConnection,
    group_name: &str,
    member: MozilliansGroupMembership,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let user_profile =
        get_user_profile(connection, &member.auth0_user_id, cis_client.clone()).await?;
    let user = User {
        user_uuid: user_profile.user_uuid,
    };
    let role = internal::member::role_for(connection, &user.user_uuid, group_name)?;
    if role.is_some() {
        return Ok(());
    }
    let host = if member.host.is_empty() {
        User::default()
    } else {
        let host_profile = get_user_profile(connection, &member.host, cis_client.clone()).await?;
        User {
            user_uuid: host_profile.user_uuid,
        }
    };
    internal::member::add_to_group(
        connection,
        group_name,
        &host,
        &user,
        Some(member.expiration),
    )?;
    add_group_to_profile(
        cis_client.clone(),
        group_name.to_owned(),
        user_profile.profile,
    )
    .await?;
    Ok(())
}

pub async fn import_members(
    connection: &PgConnection,
    group_name: &str,
    moz_members: Vec<MozilliansGroupMembership>,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    for member in moz_members {
        let user_id = member.auth0_user_id.clone();
        match import_member(connection, group_name, member, cis_client.clone()).await {
            Ok(()) => {}
            Err(e) => warn!(
                "unable to add member {} for group {}: {}",
                &user_id, group_name, e
            ),
        }
    }
    Ok(())
}

pub async fn import(
    pool: &Pool,
    group_import: GroupImport,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let group_name = group_import.group.name.clone();
    import_group(&connection, group_import.group)?;
    import_curators(
        &connection,
        &group_name,
        group_import.curators,
        cis_client.clone(),
    )
    .await?;
    import_members(
        &connection,
        &group_name,
        group_import.memberships,
        cis_client.clone(),
    )
    .await?;
    Ok(())
}
