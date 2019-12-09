use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::operations::error::OperationError;
use crate::db::operations::models::GroupWithTermsFlag;
use crate::db::schema;
use crate::db::types::*;
use diesel::dsl::exists;
use diesel::dsl::select;
use diesel::prelude::*;
use failure::Error;

pub fn get_group_with_terms_flag(
    pool: &Pool,
    group_name: &str,
) -> Result<GroupWithTermsFlag, Error> {
    let connection = pool.get()?;
    let group = schema::groups::table
        .filter(schema::groups::name.eq(group_name))
        .first::<Group>(&connection)?;
    let terms = select(exists(
        schema::terms::table.filter(schema::terms::group_id.eq(group.id)),
    ))
    .get_result(&connection)?;
    Ok(GroupWithTermsFlag { group, terms })
}

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
    trust: TrustType,
    group_expiration: Option<i32>,
) -> Result<Group, Error> {
    let connection = pool.get()?;
    let group = InsertGroup {
        name,
        path: String::from("/access_information/mozillians/"),
        description,
        capabilities,
        typ,
        trust,
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

pub fn delete_group(pool: &Pool, name: &str) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = get_group(pool, name)?;
    diesel::delete(schema::roles::table)
        .filter(schema::roles::group_id.eq(group.id))
        .execute(&connection)
        .map(|_| ())?;
    diesel::delete(schema::terms::table)
        .filter(schema::terms::group_id.eq(group.id))
        .execute(&connection)
        .optional()
        .map(|_| ())?;
    diesel::delete(schema::groups::table)
        .filter(schema::groups::name.eq(name))
        .execute(&connection)
        .map(|_| ())
        .map_err(Into::into)
}
