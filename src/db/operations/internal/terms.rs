use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::schema;
use crate::db::types::*;
use diesel::prelude::*;
use failure::Error;

pub fn get_terms(pool: &Pool, group_name: &str) -> Result<Option<String>, Error> {
    let connection = pool.get()?;
    let group = schema::groups::table
        .filter(schema::groups::name.eq(group_name))
        .first::<Group>(&connection)?;
    Terms::belonging_to(&group)
        .first(&connection)
        .map(|t: Terms| t.text)
        .optional()
        .map_err(Into::into)
}

pub fn delete_terms(pool: &Pool, group_name: &str) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = schema::groups::table
        .filter(schema::groups::name.eq(group_name))
        .first::<Group>(&connection)?;
    diesel::delete(schema::terms::table)
        .filter(schema::terms::group_id.eq(&group.id))
        .execute(&connection)
        .map(|_| ())
        .map_err(Into::into)
}

pub fn set_terms(pool: &Pool, group_name: &str, text: String) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = schema::groups::table
        .filter(schema::groups::name.eq(group_name))
        .first::<Group>(&connection)?;
    let terms = Terms {
        group_id: group.id,
        text,
    };
    diesel::insert_into(schema::terms::table)
        .values(&terms)
        .on_conflict(schema::terms::group_id)
        .do_update()
        .set(&terms)
        .execute(&connection)
        .map(|_| ())
        .map_err(Into::into)
}
