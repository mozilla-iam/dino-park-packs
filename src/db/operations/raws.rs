use crate::db::internal;
use crate::db::internal::raw::*;
use crate::db::operations::models::RawUserData;
use crate::db::Pool;
use dino_park_gate::scope::ScopeAndUser;
use failure::Error;
use uuid::Uuid;

pub fn raw_user_data(
    pool: &Pool,
    scope_and_user: &ScopeAndUser,
    user_uuid: Option<Uuid>,
) -> Result<RawUserData, Error> {
    let connection = pool.get()?;
    let user_uuid = match user_uuid {
        Some(user_uuid) => user_uuid,
        _ => internal::user::user_by_id(&connection, &scope_and_user.user_id)?.user_uuid,
    };

    let user_profile = raw_user_for_user(&connection, &user_uuid)?;
    let memberships = raw_memberships_for_user(&connection, &user_uuid)?;
    let invitations = raw_invitations_for_user(&connection, &user_uuid)?;
    let requests = raw_requests_for_user(&connection, &user_uuid)?;
    let logs = raw_logs_for_user(&connection, &user_uuid)?;

    Ok(RawUserData {
        user_profile,
        memberships,
        invitations,
        requests,
        logs,
    })
}
