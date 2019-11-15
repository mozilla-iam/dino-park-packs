use crate::db::db::Pool;
use crate::db::group::*;
use crate::db::schema;
use crate::db::types::*;
use diesel::prelude::*;
use failure::Error;

pub fn get_group(pool: &Pool, group_name: &str) -> Result<Group, Error> {
    let connection = pool.get()?;
    schema::groups::table
        .filter(schema::groups::name.eq(group_name))
        .first::<Group>(&connection)
        .map_err(Into::into)
}

pub fn add_group(
    pool: &Pool,
    name: String,
    description: String,
    capabilities: Vec<CapabilityType>,
    typ: GroupType,
    group_expiration: Option<i32>,
) -> Result<Group, Error> {
    let connection = pool.get()?;
    let group = InsertGroup {
        name,
        path: String::from("/access_information/mozillians/"),
        description,
        capabilities,
        typ,
        group_expiration,
    };
    diesel::insert_into(schema::groups::table)
        .values(&group)
        .on_conflict_do_nothing()
        .get_result(&connection)
        .map_err(Into::into)
}

pub fn update_group(
    pool: &Pool,
    name: String,
    description: Option<String>,
    capabilities: Option<Vec<CapabilityType>>,
    typ: Option<GroupType>,
    group_expiration: Option<Option<i32>>,
) -> Result<Group, Error> {
    let connection = pool.get()?;
    diesel::update(schema::groups::table.filter(schema::groups::name.eq(&name)))
        .set((
            description.map(|d| schema::groups::description.eq(d)),
            capabilities.map(|c| schema::groups::capabilities.eq(c)),
            typ.map(|t| schema::groups::typ.eq(t)),
            group_expiration.map(|e| schema::groups::group_expiration.eq(e)),
        ))
        .get_result(&connection)
        .map_err(Into::into)
}
