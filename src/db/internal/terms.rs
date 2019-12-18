use crate::db::internal;
use crate::db::logs::LogContext;
use crate::db::model::*;
use crate::db::schema;
use crate::db::types::LogOperationType;
use crate::db::types::LogTargetType;
use crate::db::Pool;
use diesel::prelude::*;
use failure::Error;
use uuid::Uuid;

pub fn get_terms(pool: &Pool, group_name: &str) -> Result<Option<String>, Error> {
    let connection = pool.get()?;
    let group = internal::group::get_group(pool, group_name)?;
    Terms::belonging_to(&group)
        .first(&connection)
        .map(|t: Terms| t.text)
        .optional()
        .map_err(Into::into)
}

pub fn delete_terms(host_uuid: &Uuid, pool: &Pool, group_name: &str) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = internal::group::get_group(pool, group_name)?;
    let log_ctx = LogContext::with(group.id, *host_uuid);
    diesel::delete(schema::terms::table)
        .filter(schema::terms::group_id.eq(&group.id))
        .execute(&connection)
        .map(|_| {
            internal::log::db_log(
                &connection,
                &log_ctx,
                LogTargetType::Terms,
                LogOperationType::Updated,
                None,
            );
        })
        .map_err(Into::into)
}

pub fn set_terms(
    host_uuid: &Uuid,
    pool: &Pool,
    group_name: &str,
    text: String,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = internal::group::get_group(pool, group_name)?;
    let terms = Terms {
        group_id: group.id,
        text,
    };
    let log_ctx = LogContext::with(group.id, *host_uuid);
    diesel::insert_into(schema::terms::table)
        .values(&terms)
        .on_conflict(schema::terms::group_id)
        .do_update()
        .set(&terms)
        .execute(&connection)
        .map(|_| {
            internal::log::db_log(
                &connection,
                &log_ctx,
                LogTargetType::Terms,
                LogOperationType::Updated,
                None,
            );
        })
        .map_err(Into::into)
}
