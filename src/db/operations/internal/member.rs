use crate::db::db::Pool;
use crate::db::group::*;
use crate::db::schema;
use crate::db::types::*;
use diesel::prelude::*;
use failure::Error;
use uuid::Uuid;

pub fn add_member(
    pool: &Pool,
    group_id: i32,
    user_uuid: Uuid,
    added_by: Uuid,
) -> Result<Membership, Error> {
    let connection = pool.get()?;
    let membership = InsertMembership {
        group_id,
        user_uuid,
        role_id: None,
        added_by,
    };
    diesel::insert_into(schema::memberships::table)
        .values(&membership)
        .on_conflict((
            schema::memberships::user_uuid,
            schema::memberships::group_id,
        ))
        .do_update()
        .set(&membership)
        .get_result(&*connection)
        .map_err(Into::into)
}
