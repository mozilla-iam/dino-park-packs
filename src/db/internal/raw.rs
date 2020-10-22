use crate::db::logs::Log;
use crate::db::model::*;
use crate::db::schema;
use crate::db::users::UserProfile;
use crate::db::users::UserProfileValue;
use diesel::prelude::*;
use failure::Error;
use std::convert::TryInto;
use uuid::Uuid;

pub fn raw_memberships_for_user(
    connection: &PgConnection,
    user_uuid: &Uuid,
) -> Result<Vec<Membership>, Error> {
    use schema::memberships as m;

    m::table
        .filter(m::user_uuid.eq(user_uuid))
        .get_results(connection)
        .map_err(Into::into)
}

pub fn raw_invitations_for_user(
    connection: &PgConnection,
    user_uuid: &Uuid,
) -> Result<Vec<Invitation>, Error> {
    use schema::invitations as i;

    i::table
        .filter(i::user_uuid.eq(user_uuid).or(i::added_by.eq(user_uuid)))
        .get_results(connection)
        .map_err(Into::into)
}

pub fn raw_requests_for_user(
    connection: &PgConnection,
    user_uuid: &Uuid,
) -> Result<Vec<Request>, Error> {
    use schema::requests as r;

    r::table
        .filter(r::user_uuid.eq(user_uuid))
        .get_results(connection)
        .map_err(Into::into)
}

pub fn raw_user_for_user(
    connection: &PgConnection,
    user_uuid: &Uuid,
) -> Result<UserProfile, Error> {
    use schema::profiles as p;

    p::table
        .filter(p::user_uuid.eq(user_uuid))
        .get_result::<UserProfileValue>(connection)
        .map_err(Error::from)
        .and_then(TryInto::try_into)
}

pub fn raw_logs_for_user(connection: &PgConnection, user_uuid: &Uuid) -> Result<Vec<Log>, Error> {
    use schema::logs as l;

    l::table
        .filter(l::user_uuid.eq(user_uuid).or(l::host_uuid.eq(user_uuid)))
        .get_results(connection)
        .map_err(Into::into)
}
