use crate::db::db::Pool;
use crate::db::group::*;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::db::schema::memberships::dsl::*;
use crate::db::types::*;
use crate::user::User;
use diesel::dsl::count;
use diesel::prelude::*;
use failure::Error;
use log::info;

pub fn add_user_to_group(
    connection: &PgConnection,
    group_name: String,
    creator: User,
    user: User,
) -> Result<(), Error> {
    let group = groups::groups
        .filter(groups::name.eq(&group_name))
        .first::<Group>(&*connection)?;
    let membership = InsertMembership {
        user_uuid: user.user_uuid,
        group_id: group.id.clone(),
        role_id: None,
        added_by: None,
    };
    let rows_inserted = diesel::insert_into(schema::memberships::table)
        .values(&membership)
        .on_conflict_do_nothing()
        .execute(&*connection)?;
    info!("Inserted {} rows", rows_inserted);

    Ok(())
}

pub fn member_count(pool: &Pool, group_name: String) -> Result<i64, Error> {
    let connection = pool.get()?;
    let count = memberships
        .inner_join(groups::groups)
        .filter(groups::name.eq(&group_name))
        .select(count(user_uuid))
        .first(&connection)?;
    Ok(count)
}
