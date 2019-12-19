use crate::db::error::DBError;
use crate::db::schema;
use crate::db::types::TrustType;
use crate::db::users::UserProfile;
use crate::db::users::*;
use crate::db::Pool;
use crate::user::User;
use cis_profile::schema::Profile;
use diesel::prelude::*;
use failure::Error;
use log::error;
use std::convert::TryFrom;
use uuid::Uuid;

pub fn user_by_id(pool: &Pool, user_id: &str) -> Result<User, Error> {
    let connection = pool.get()?;
    schema::user_ids::table
        .filter(schema::user_ids::user_id.eq(user_id))
        .select(schema::user_ids::user_uuid)
        .first(&connection)
        .map(|user_uuid| User { user_uuid })
        .map_err(Into::into)
}

pub fn user_profile_by_uuid(pool: &Pool, user_uuid: &Uuid) -> Result<UserProfile, Error> {
    let connection = pool.get()?;

    schema::profiles::table
        .filter(schema::profiles::user_uuid.eq(user_uuid))
        .first::<UserProfileValue>(&connection)
        .map_err(Error::from)
        .and_then(|p| UserProfile::try_from(p).map_err(Into::into))
}

pub fn user_profile_by_user_id(pool: &Pool, user_id: &str) -> Result<UserProfile, Error> {
    let connection = pool.get()?;

    schema::profiles::table
        .filter(schema::profiles::user_id.eq(user_id))
        .first::<UserProfileValue>(&connection)
        .map_err(Error::from)
        .and_then(|p| UserProfile::try_from(p).map_err(Into::into))
}

pub fn update_user_cache(pool: &Pool, profile: &Profile) -> Result<(), Error> {
    let connection = pool.get()?;

    let user_profile = UserProfileValue::try_from(UserProfile::try_from(profile.clone())?)?;

    diesel::insert_into(schema::profiles::table)
        .values(&user_profile)
        .on_conflict(schema::profiles::user_uuid)
        .do_update()
        .set(&user_profile)
        .execute(&connection)?;

    let profile_id_uuid = UserIdUuid {
        user_uuid: user_profile.user_uuid,
        user_id: user_profile.user_id.clone(),
    };

    let profile_uuid = &user_profile.user_uuid;

    match schema::user_ids::table
        .filter(schema::user_ids::user_uuid.eq(profile_uuid))
        .first::<UserIdUuid>(&connection)
    {
        Ok(ref id_uuid) if &profile_id_uuid != id_uuid => error!(
            "changed user_id/user_uuid: {}/{} → {}/{}",
            id_uuid.user_uuid, id_uuid.user_id, profile_uuid, profile_uuid
        ),
        Err(diesel::NotFound) => diesel::insert_into(schema::user_ids::table)
            .values(profile_id_uuid)
            .execute(&connection)
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
        .execute(&connection)?;

    let ndaed_profile = UsersNdaed::from(profile);
    diesel::insert_into(schema::users_ndaed::table)
        .values(&ndaed_profile)
        .on_conflict(schema::users_ndaed::user_uuid)
        .do_update()
        .set(&ndaed_profile)
        .execute(&connection)?;

    let vouched_profile = UsersVouched::from(profile);
    diesel::insert_into(schema::users_vouched::table)
        .values(&vouched_profile)
        .on_conflict(schema::users_vouched::user_uuid)
        .do_update()
        .set(&vouched_profile)
        .execute(&connection)?;

    let authenticated_profile = UsersAuthenticated::from(profile);
    diesel::insert_into(schema::users_authenticated::table)
        .values(&authenticated_profile)
        .on_conflict(schema::users_authenticated::user_uuid)
        .do_update()
        .set(&authenticated_profile)
        .execute(&connection)?;

    let public_profile = UsersPublic::from(profile);
    diesel::insert_into(schema::users_public::table)
        .values(&public_profile)
        .on_conflict(schema::users_public::user_uuid)
        .do_update()
        .set(&public_profile)
        .execute(&connection)?;
    Ok(())
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
            .get_results::<$typ>(&$connection)
            .map(|users| users.into_iter().map(|u| u.into()).collect())
            .map_err(Into::into)
    }};
}

pub fn search_users(
    pool: &Pool,
    trust: Option<TrustType>,
    scope: TrustType,
    q: &str,
    limit: i64,
) -> Result<Vec<DisplayUser>, Error> {
    let connection = pool.get()?;
    let trust = trust.unwrap_or(TrustType::Staff);
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
        _ => Err(DBError::InvalidTurstLevel.into()),
    }
}