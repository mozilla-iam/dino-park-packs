use crate::db::db::Pool;
use crate::db::group::*;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::db::schema::memberships::dsl::*;
use crate::db::types::*;
use crate::user::User;
use diesel::dsl::count;
use diesel::prelude::*;
use failure::format_err;
use failure::Error;
use log::info;
use serde_derive::Serialize;

fn users_for_scope(scope: &str) -> Result<impl Table, Error> {
    match scope {
        "staff" => Ok(schema::users_staff::table),
        _ => Err(format_err!("invalid scope")),
    }
}

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

#[derive(Queryable, Serialize)]
pub struct DisplayMember {
    pub picture: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: RoleType,
}

fn staff_scoped_members(
    connection: &PgConnection,
    group_name: &str,
) -> Result<Vec<DisplayMember>, Error> {
    use schema::users_staff as u;
    memberships
        .inner_join(u::table.on(user_uuid.eq(u::user_uuid)))
        .inner_join(schema::roles::table)
        .select((
            u::picture,
            u::first_name,
            u::last_name,
            u::username,
            u::email,
            schema::roles::typ,
        ))
        .limit(20)
        .get_results(connection)
        .map_err(Into::into)
}

pub fn scoped_members(
    pool: &Pool,
    group_name: &str,
    scope: String,
) -> Result<Vec<DisplayMember>, Error> {
    let connection = pool.get()?;

    staff_scoped_members(&connection, group_name)
}

pub fn member_count(pool: &Pool, group_name: &str) -> Result<i64, Error> {
    let connection = pool.get()?;
    let count = memberships
        .inner_join(groups::groups)
        .filter(groups::name.eq(group_name))
        .select(count(user_uuid))
        .first(&connection)?;
    Ok(count)
}
