use crate::db::db::Pool;
use crate::db::group::*;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::db::schema::memberships::dsl::*;
use crate::db::types::*;
use crate::db::views;
use crate::user::User;
use chrono::NaiveDateTime;
use diesel::dsl::count;
use diesel::prelude::*;
use failure::format_err;
use failure::Error;
use log::info;
use serde_derive::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize)]
pub struct PendingInvitations {}

pub fn invite_member(
    pool: &Pool,
    group_name: &str,
    host: User,
    member: User,
    invitation_expiration: Option<NaiveDateTime>,
    group_expiration: Option<NaiveDateTime>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = groups::groups
        .filter(groups::name.eq(group_name))
        .first::<Group>(&*connection)?;
    let invitation = Invitation {
        user_uuid: member.user_uuid,
        group_id: group.id.clone(),
        invitation_expiration,
        group_expiration,
        added_by: host.user_uuid,
    };
    let rows_inserted = diesel::insert_into(schema::invitations::table)
        .values(&invitation)
        .on_conflict_do_nothing()
        .execute(&*connection)?;
    info!("Inserted {} rows", rows_inserted);

    Ok(())
}

pub fn pending_invitations_count(pool: &Pool, group_name: &str) -> Result<i64, Error> {
    let connection = pool.get()?;
    let count = schema::invitations::table
        .inner_join(groups::groups)
        .filter(groups::name.eq(group_name))
        .select(count(schema::invitations::user_uuid))
        .first(&connection)?;
    Ok(count)
}
