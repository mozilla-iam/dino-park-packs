use crate::db::internal;
use crate::db::schema;
use crate::db::types::TrustType;
use crate::db::users::UserProfile;
use crate::db::users::*;
use crate::error::PacksError;
use crate::user::User;
use cis_profile::schema::Profile;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use failure::Error;
use log::error;
use log::info;
use std::convert::TryFrom;
use uuid::Uuid;

pub fn user_trust(connection: &PgConnection, user_uuid: &Uuid) -> Result<TrustType, Error> {
    schema::profiles::table
        .filter(schema::profiles::user_uuid.eq(user_uuid))
        .select(schema::profiles::trust)
        .first(connection)
        .map_err(Into::into)
}

pub fn user_by_id(connection: &PgConnection, user_id: &str) -> Result<User, Error> {
    schema::user_ids::table
        .filter(schema::user_ids::user_id.eq(user_id))
        .select(schema::user_ids::user_uuid)
        .first(connection)
        .map(|user_uuid| User { user_uuid })
        .map_err(Into::into)
}

pub fn user_profile_by_uuid(
    connection: &PgConnection,
    user_uuid: &Uuid,
) -> Result<UserProfile, Error> {
    schema::profiles::table
        .filter(schema::profiles::user_uuid.eq(user_uuid))
        .first::<UserProfileValue>(connection)
        .map_err(Error::from)
        .and_then(|p| UserProfile::try_from(p).map_err(Into::into))
        .map_err(|_| PacksError::ProfileNotFound.into())
}

pub fn user_profile_by_uuid_maybe(
    connection: &PgConnection,
    user_uuid: &Uuid,
) -> Result<Option<UserProfile>, Error> {
    schema::profiles::table
        .filter(schema::profiles::user_uuid.eq(user_uuid))
        .first::<UserProfileValue>(connection)
        .optional()
        .map_err(Error::from)
        .map(|p| p.and_then(|p| UserProfile::try_from(p).ok()))
        .map_err(|_| PacksError::ProfileNotFound.into())
}

pub fn slim_user_profile_by_uuid(
    connection: &PgConnection,
    user_uuid: &Uuid,
) -> Result<UserProfileSlim, Error> {
    use schema::profiles as p;
    schema::profiles::table
        .filter(p::user_uuid.eq(user_uuid))
        .select((p::user_uuid, p::user_id, p::email, p::username, p::trust))
        .first::<UserProfileSlim>(connection)
        .map_err(|_| PacksError::ProfileNotFound.into())
}

pub fn user_profile_by_user_id(
    connection: &PgConnection,
    user_id: &str,
) -> Result<UserProfile, Error> {
    schema::profiles::table
        .filter(schema::profiles::user_id.eq(user_id))
        .first::<UserProfileValue>(connection)
        .map_err(Error::from)
        .and_then(|p| UserProfile::try_from(p).map_err(Into::into))
        .map_err(|_| PacksError::ProfileNotFound.into())
}

pub fn delete_user(connection: &PgConnection, user: &User) -> Result<(), Error> {
    diesel::delete(schema::invitations::table)
        .filter(schema::invitations::user_uuid.eq(user.user_uuid))
        .execute(connection)?;
    diesel::delete(schema::memberships::table)
        .filter(schema::memberships::user_uuid.eq(user.user_uuid))
        .execute(connection)?;
    diesel::delete(schema::users_staff::table)
        .filter(schema::users_staff::user_uuid.eq(user.user_uuid))
        .execute(connection)?;
    diesel::delete(schema::users_ndaed::table)
        .filter(schema::users_ndaed::user_uuid.eq(user.user_uuid))
        .execute(connection)?;
    diesel::delete(schema::users_vouched::table)
        .filter(schema::users_vouched::user_uuid.eq(user.user_uuid))
        .execute(connection)?;
    diesel::delete(schema::users_authenticated::table)
        .filter(schema::users_authenticated::user_uuid.eq(user.user_uuid))
        .execute(connection)?;
    diesel::delete(schema::users_public::table)
        .filter(schema::users_public::user_uuid.eq(user.user_uuid))
        .execute(connection)?;

    diesel::delete(schema::user_ids::table)
        .filter(schema::user_ids::user_uuid.eq(user.user_uuid))
        .execute(connection)?;
    diesel::delete(schema::profiles::table)
        .filter(schema::profiles::user_uuid.eq(user.user_uuid))
        .execute(connection)?;

    info!("deleted user: {}", user.user_uuid);
    Ok(())
}

pub fn update_user_cache(connection: &PgConnection, profile: &Profile) -> Result<(), Error> {
    let user_profile = UserProfile::try_from(profile.clone())?;
    let user_profile = UserProfileValue::try_from(user_profile)?;

    diesel::insert_into(schema::profiles::table)
        .values(&user_profile)
        .on_conflict(schema::profiles::user_uuid)
        .do_update()
        .set(&user_profile)
        .execute(connection)?;

    let profile_id_uuid = UserIdUuid {
        user_uuid: user_profile.user_uuid,
        user_id: user_profile.user_id.clone(),
    };

    let profile_uuid = &user_profile.user_uuid;

    match schema::user_ids::table
        .filter(schema::user_ids::user_uuid.eq(profile_uuid))
        .first::<UserIdUuid>(connection)
    {
        Ok(ref id_uuid) if &profile_id_uuid != id_uuid => error!(
            "changed user_id/user_uuid: {}/{} → {}/{}",
            id_uuid.user_uuid, id_uuid.user_id, profile_uuid, profile_uuid
        ),
        Err(diesel::NotFound) => diesel::insert_into(schema::user_ids::table)
            .values(profile_id_uuid)
            .execute(connection)
            .map(|_| ())?,
        Err(e) => {
            error!("error verifying uuid/id consistency: {}", e);
            return Err(e.into());
        }
        _ => (),
    }

    let staff_profile = UsersStaff::from(profile);
    diesel::insert_into(schema::users_staff::table)
        .values(&staff_profile)
        .on_conflict(schema::users_staff::user_uuid)
        .do_update()
        .set(&staff_profile)
        .execute(connection)?;

    let ndaed_profile = UsersNdaed::from(profile);
    diesel::insert_into(schema::users_ndaed::table)
        .values(&ndaed_profile)
        .on_conflict(schema::users_ndaed::user_uuid)
        .do_update()
        .set(&ndaed_profile)
        .execute(connection)?;

    let vouched_profile = UsersVouched::from(profile);
    diesel::insert_into(schema::users_vouched::table)
        .values(&vouched_profile)
        .on_conflict(schema::users_vouched::user_uuid)
        .do_update()
        .set(&vouched_profile)
        .execute(connection)?;

    let authenticated_profile = UsersAuthenticated::from(profile);
    diesel::insert_into(schema::users_authenticated::table)
        .values(&authenticated_profile)
        .on_conflict(schema::users_authenticated::user_uuid)
        .do_update()
        .set(&authenticated_profile)
        .execute(connection)?;

    let public_profile = UsersPublic::from(profile);
    diesel::insert_into(schema::users_public::table)
        .values(&public_profile)
        .on_conflict(schema::users_public::user_uuid)
        .do_update()
        .set(&public_profile)
        .execute(connection)?;
    Ok(())
}

macro_rules! scoped_search_users_for_group {
    ($g:ident, $t:ident, $q:ident, $trust:ident, $limit:ident, $connection:ident) => {{
        use schema::$t as u;
        u::table
            .filter(
                u::first_name
                    .concat(" ")
                    .concat(u::last_name)
                    .ilike($q)
                    .or(u::first_name.ilike($q))
                    .or(u::last_name.ilike($q))
                    .or(u::username.ilike($q))
                    .or(u::email.ilike($q)),
            )
            .filter(u::trust.ge($trust))
            .left_outer_join(
                schema::memberships::table.on(schema::memberships::user_uuid
                    .eq(u::user_uuid)
                    .and(schema::memberships::group_id.eq($g))),
            )
            .left_outer_join(
                schema::roles::table.on(schema::roles::role_id.eq(schema::memberships::role_id)),
            )
            .left_outer_join(
                schema::invitations::table.on(schema::invitations::user_uuid
                    .eq(u::user_uuid)
                    .and(schema::invitations::group_id.eq($g))),
            )
            .order((
                schema::memberships::group_id.desc().nulls_first(),
                schema::invitations::group_id.desc().nulls_first(),
            ))
            .select((
                u::user_uuid,
                u::picture,
                u::first_name,
                u::last_name,
                u::username,
                u::email,
                u::trust,
                schema::roles::typ.nullable(),
                schema::invitations::group_id.nullable().is_not_null(),
            ))
            .limit($limit)
            .get_results::<UserForGroup>($connection)
            .map_err(Into::into)
    }};
}

macro_rules! scoped_search_curators_for_group {
    ($g:ident, $t:ident, $q:ident, $trust:ident, $limit:ident, $connection:ident) => {{
        use schema::$t as u;
        u::table
            .filter(
                u::first_name
                    .concat(" ")
                    .concat(u::last_name)
                    .ilike($q)
                    .or(u::first_name.ilike($q))
                    .or(u::last_name.ilike($q))
                    .or(u::username.ilike($q))
                    .or(u::email.ilike($q)),
            )
            .filter(u::trust.ge($trust))
            .left_outer_join(
                schema::memberships::table.on(schema::memberships::user_uuid
                    .eq(u::user_uuid)
                    .and(schema::memberships::group_id.eq($g))),
            )
            .left_outer_join(
                schema::roles::table.on(schema::roles::role_id.eq(schema::memberships::role_id)),
            )
            .order(schema::roles::typ.asc().nulls_first())
            .select((
                u::user_uuid,
                u::picture,
                u::first_name,
                u::last_name,
                u::username,
                u::email,
                u::trust,
                schema::roles::typ.nullable(),
                false.into_sql::<Bool>(),
            ))
            .limit($limit)
            .get_results::<UserForGroup>($connection)
            .map_err(Into::into)
    }};
}

macro_rules! scoped_search_users {
    ($t:ident, $typ:ident, $q:ident, $trust:ident, $limit:ident, $connection:ident) => {{
        use schema::$t as u;
        u::table
            .filter(
                u::first_name
                    .concat(" ")
                    .concat(u::last_name)
                    .ilike($q)
                    .or(u::first_name.ilike($q))
                    .or(u::last_name.ilike($q))
                    .or(u::username.ilike($q))
                    .or(u::email.ilike($q)),
            )
            .filter(u::trust.ge($trust))
            .limit($limit)
            .get_results::<$typ>($connection)
            .map(|users| users.into_iter().map(|u| u.into()).collect())
            .map_err(Into::into)
    }};
}

pub fn search_curators_for_group(
    connection: &PgConnection,
    group_name: &str,
    scope: TrustType,
    q: &str,
    limit: i64,
) -> Result<Vec<UserForGroup>, Error> {
    let q: &str = &format!("{}%", q);
    let group_id = internal::group::get_group(connection, group_name)?.id;
    let trust = TrustType::Ndaed;
    match scope {
        TrustType::Staff => {
            scoped_search_curators_for_group!(group_id, users_staff, q, trust, limit, connection)
        }
        TrustType::Ndaed => {
            scoped_search_curators_for_group!(group_id, users_ndaed, q, trust, limit, connection)
        }
        TrustType::Vouched => {
            scoped_search_curators_for_group!(group_id, users_vouched, q, trust, limit, connection)
        }
        TrustType::Authenticated => scoped_search_curators_for_group!(
            group_id,
            users_authenticated,
            q,
            trust,
            limit,
            connection
        ),
        TrustType::Public => {
            scoped_search_curators_for_group!(group_id, users_public, q, trust, limit, connection)
        }
    }
}
pub fn search_users_for_group(
    connection: &PgConnection,
    group_name: &str,
    trust: TrustType,
    scope: TrustType,
    q: &str,
    limit: i64,
) -> Result<Vec<UserForGroup>, Error> {
    let q: &str = &format!("{}%", q);
    let group_id = internal::group::get_group(connection, group_name)?.id;
    match scope {
        TrustType::Staff => {
            scoped_search_users_for_group!(group_id, users_staff, q, trust, limit, connection)
        }
        TrustType::Ndaed => {
            scoped_search_users_for_group!(group_id, users_ndaed, q, trust, limit, connection)
        }
        TrustType::Vouched => {
            scoped_search_users_for_group!(group_id, users_vouched, q, trust, limit, connection)
        }
        TrustType::Authenticated => scoped_search_users_for_group!(
            group_id,
            users_authenticated,
            q,
            trust,
            limit,
            connection
        ),
        TrustType::Public => {
            scoped_search_users_for_group!(group_id, users_public, q, trust, limit, connection)
        }
    }
}

pub fn search_users(
    connection: &PgConnection,
    trust: TrustType,
    scope: TrustType,
    q: &str,
    limit: i64,
) -> Result<Vec<DisplayUser>, Error> {
    let q: &str = &format!("{}%", q);
    match scope {
        TrustType::Staff => {
            scoped_search_users!(users_staff, UsersStaff, q, trust, limit, connection)
        }
        TrustType::Ndaed => {
            scoped_search_users!(users_ndaed, UsersNdaed, q, trust, limit, connection)
        }
        TrustType::Vouched => {
            scoped_search_users!(users_vouched, UsersVouched, q, trust, limit, connection)
        }
        TrustType::Authenticated => scoped_search_users!(
            users_authenticated,
            UsersAuthenticated,
            q,
            trust,
            limit,
            connection
        ),
        TrustType::Public => {
            scoped_search_users!(users_public, UsersPublic, q, trust, limit, connection)
        }
    }
}
