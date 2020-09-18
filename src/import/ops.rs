use crate::cis::operations::_send_groups_to_cis;
use crate::db::internal;
use crate::db::logs::LogContext;
use crate::db::operations::models::NewGroup;
use crate::db::schema;
use crate::db::types::*;
use crate::db::users::trust_for_profile;
use crate::db::users::LegacyUserData;
use crate::db::users::UserProfile;
use crate::db::Pool;
use crate::import::tsv::LegacyUserDataRaw;
use crate::import::tsv::MozilliansGroup;
use crate::import::tsv::MozilliansGroupCurator;
use crate::import::tsv::MozilliansGroupMembership;
use crate::user::User;
use chrono::DateTime;
use chrono::Utc;
use cis_client::getby::GetBy;
use cis_client::AsyncCisClientTrait;
use diesel::pg::upsert::excluded;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use failure::Error;
use log::warn;
use std::convert::TryInto;
use std::sync::Arc;

const EXPIRATION_BUFFER: i32 = 60;

fn calc_expiration(member_expiration: i32, updated: DateTime<Utc>) -> Option<i32> {
    if member_expiration > 0 {
        let expiration =
            member_expiration - (Utc::now().signed_duration_since(updated).num_days() as i32);
        if expiration > EXPIRATION_BUFFER {
            Some(expiration)
        } else {
            Some(EXPIRATION_BUFFER)
        }
    } else {
        None
    }
}

pub struct GroupImport {
    pub group: MozilliansGroup,
    pub memberships: Vec<MozilliansGroupMembership>,
    pub curators: Vec<MozilliansGroupCurator>,
    pub trust: TrustType,
}

pub fn import_legacy_user_data(
    pool: &Pool,
    legacy_user_data_raw: Vec<LegacyUserDataRaw>,
) -> Result<(), Error> {
    use schema::legacy_user_data as lud;
    let connection = pool.get()?;
    let legacy_user_data: Vec<LegacyUserData> = legacy_user_data_raw
        .into_iter()
        .filter_map(
            |ludr| match internal::user::user_by_id(&connection, &ludr.user_id) {
                Ok(user) => Some(LegacyUserData {
                    user_uuid: user.user_uuid,
                    first_name: ludr.full_name,
                    email: ludr.email,
                }),
                Err(e) => {
                    warn!("no user_uuid for {}: {}", ludr.user_id, e);
                    None
                }
            },
        )
        .collect();
    diesel::insert_into(lud::table)
        .values(legacy_user_data)
        .on_conflict(lud::user_uuid)
        .do_update()
        .set((
            lud::first_name.eq(excluded(lud::first_name)),
            lud::email.eq(excluded(lud::email)),
        ))
        .execute(&connection)?;
    Ok(())
}

pub fn import_group(
    connection: &PgConnection,
    moz_group: MozilliansGroup,
    trust: TrustType,
) -> Result<(), Error> {
    if internal::group::get_group(connection, &moz_group.name).is_ok() {
        return Ok(());
    }
    let group_name = moz_group.name.clone();
    let description = match (moz_group.website.as_str(), moz_group.wiki.as_str()) {
        ("", "") => moz_group.description,
        (website, "") => format!(
            "{}\n\n**Website:** [{}]({})",
            moz_group.description, website, website
        ),
        ("", wiki) => format!(
            "{}\n\n**Wiki:** [{}]({})",
            moz_group.description, wiki, wiki
        ),
        (website, wiki) if website == wiki => format!(
            "{}\n\n**Website:** [{}]({})",
            moz_group.description, website, website
        ),
        (website, wiki) => format!(
            "{}\n\n**Website:** [{}]({})\n\n**Wiki:** [{}]({})",
            moz_group.description, website, website, wiki, wiki
        ),
    };
    let new_group = NewGroup {
        name: moz_group.name,
        typ: if moz_group.typ == "by_request" {
            GroupType::Reviewed
        } else {
            GroupType::Closed
        },
        description,
        trust,
        capabilities: Default::default(),
        group_expiration: Some(moz_group.expiration),
    };
    let creator = User::default();
    let new_group = internal::group::add_group(&creator.user_uuid, &connection, new_group)?;
    let log_ctx = LogContext::with(new_group.id, creator.user_uuid);
    internal::admin::add_admin_role(&log_ctx, &connection, new_group.id)?;
    internal::member::add_member_role(&creator.user_uuid, &connection, new_group.id)?;
    if !moz_group.terms.is_empty() {
        internal::terms::set_terms(&creator.user_uuid, connection, &group_name, moz_group.terms)?;
    }
    if !moz_group.invitation_email.is_empty() {
        internal::invitation::update_invitation_text(
            connection,
            &group_name,
            &creator,
            moz_group.invitation_email,
        )?;
    }
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
        warn!("no profile for {} → fetching", user_id);
        let profile = cis_client
            .clone()
            .get_user_by(user_id, &GetBy::UserId, None)
            .await?;
        internal::user::update_user_cache(connection, &profile)?;
        profile.try_into()
    }
}

async fn import_curator(
    connection: &PgConnection,
    group_name: &str,
    curator: MozilliansGroupCurator,
    trust: TrustType,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let user_profile =
        get_user_profile(connection, &curator.auth0_user_id, cis_client.clone()).await?;
    let user = User {
        user_uuid: user_profile.user_uuid,
    };
    if trust_for_profile(&user_profile.profile) < trust {
        return Ok(());
    }
    internal::admin::add_admin(&connection, group_name, &User::default(), &user)?;
    let groups = internal::member::group_names_for_user(connection, &user.user_uuid)?;
    _send_groups_to_cis(cis_client, groups, user_profile.profile).await?;
    Ok(())
}

pub async fn import_curators(
    connection: &PgConnection,
    group_name: &str,
    moz_curators: Vec<MozilliansGroupCurator>,
    trust: TrustType,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    for curator in moz_curators {
        let user_id = curator.auth0_user_id.clone();
        match import_curator(connection, group_name, curator, trust, cis_client.clone()).await {
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
    trust: TrustType,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    use schema::memberships as m;

    let user_profile =
        get_user_profile(connection, &member.auth0_user_id, cis_client.clone()).await?;
    if trust_for_profile(&user_profile.profile) < trust {
        return Ok(());
    }

    let user = User {
        user_uuid: user_profile.user_uuid,
    };
    let group = internal::group::get_group(connection, group_name)?;
    let expiration = calc_expiration(member.expiration, member.updated_on);
    let role = internal::member::role_for(connection, &user.user_uuid, group_name)?;
    if role.is_some() {
        diesel::update(m::table)
            .filter(m::user_uuid.eq(user.user_uuid))
            .filter(m::group_id.eq(group.id))
            .set(m::added_ts.eq(member.date_joined.naive_utc()))
            .execute(connection)
            .map(|_| ())?;
        return Ok(());
    }
    let host = if member.host.is_empty() {
        User::default()
    } else {
        match get_user_profile(connection, &member.host, cis_client.clone()).await {
            Ok(p) => User {
                user_uuid: p.user_uuid,
            },
            _ => User::default(),
        }
    };
    internal::member::add_to_group(connection, group_name, &host, &user, expiration)?;

    diesel::update(m::table)
        .filter(m::user_uuid.eq(user.user_uuid))
        .filter(m::group_id.eq(group.id))
        .set(m::added_ts.eq(member.date_joined.naive_utc()))
        .execute(connection)
        .map(|_| ())?;

    let groups = internal::member::group_names_for_user(connection, &user.user_uuid)?;
    _send_groups_to_cis(cis_client, groups, user_profile.profile).await?;
    Ok(())
}

pub async fn import_members(
    connection: &PgConnection,
    group_name: &str,
    moz_members: Vec<MozilliansGroupMembership>,
    trust: TrustType,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    use schema::groups as g;
    let group = internal::group::get_group(connection, group_name)?;
    let mut created = group.created;
    for member in moz_members {
        let joined = member.date_joined.naive_utc();
        if joined < created {
            created = joined;
        }
        let user_id = member.auth0_user_id.clone();
        match import_member(connection, group_name, member, trust, cis_client.clone()).await {
            Ok(()) => {}
            Err(e) => warn!(
                "unable to add member {} for group {}: {}",
                &user_id, group_name, e
            ),
        }
    }
    diesel::update(g::table)
        .filter(g::group_id.eq(group.id))
        .set(g::created.eq(created))
        .execute(connection)
        .map(|_| ())?;
    Ok(())
}

pub async fn import(
    pool: &Pool,
    group_import: GroupImport,
    cis_client: Arc<impl AsyncCisClientTrait>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let group_name = group_import.group.name.clone();
    import_group(&connection, group_import.group, group_import.trust)?;
    import_curators(
        &connection,
        &group_name,
        group_import.curators,
        group_import.trust,
        cis_client.clone(),
    )
    .await?;
    import_members(
        &connection,
        &group_name,
        group_import.memberships,
        group_import.trust,
        cis_client.clone(),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_calc_expiration() {
        let updated = Utc::now() - Duration::days(20);
        let expiration = calc_expiration(365, updated);
        assert_eq!(expiration, Some(345));

        let updated = Utc::now() - Duration::days(400);
        let expiration = calc_expiration(365, updated);
        assert_eq!(expiration, Some(EXPIRATION_BUFFER));

        let updated = Utc::now() - Duration::days(400);
        let expiration = calc_expiration(0, updated);
        assert_eq!(expiration, None);
    }
}
