use crate::db::db::Pool;
use crate::db::schema;
use crate::db::users::UserProfile;
use crate::db::users::*;
use crate::user::User;
use cis_profile::schema::Profile;
use diesel::prelude::*;
use failure::format_err;
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

pub fn update_user_cache(pool: &Pool, profile: &Profile) -> Result<(), Error> {
    let connection = pool.get()?;

    let profile_uuid = Uuid::parse_str(&profile.uuid.value.clone().unwrap_or_default())?;
    let profile_id = profile
        .user_id
        .value
        .clone()
        .ok_or_else(|| format_err!("no user_id"))?;

    let user_profile = UserProfileValue::try_from(UserProfile {
        user_uuid: profile_uuid,
        profile: profile.clone(),
    })?;

    diesel::insert_into(schema::profiles::table)
        .values(&user_profile)
        .on_conflict(schema::profiles::user_uuid)
        .do_update()
        .set(&user_profile)
        .execute(&connection)?;

    let profile_id_uuid = UserIdUuid {
        user_uuid: profile_uuid.clone(),
        user_id: profile_id,
    };

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
        Err(e) => return Err(e.into()),
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
