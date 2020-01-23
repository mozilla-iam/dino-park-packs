use crate::db::logs::InsertLog;
use crate::db::logs::Log;
use crate::db::logs::LogContext;
use crate::db::schema;
use crate::db::types::LogOperationType;
use crate::db::types::LogTargetType;
use diesel::prelude::*;
use diesel::PgConnection;
use failure::Error;
use log::error;
use serde_json::Value;

pub fn db_log(
    connection: &PgConnection,
    ctx: &LogContext,
    target: LogTargetType,
    operation: LogOperationType,
    body: Option<Value>,
) {
    let log = InsertLog {
        ts: None,
        target,
        operation,
        group_id: ctx.group_id,
        host_uuid: ctx.host_uuid,
        user_uuid: ctx.user_uuid,
        ok: true,
        body,
    };
    if let Err(e) = diesel::insert_into(schema::logs::table)
        .values(&log)
        .execute(connection)
    {
        error!("Failed to log operation: {}. Logentry: {:?}", e, log);
    }
}

/*
pub fn paginated_raw_logs(connection: &PgConnection, limit: i64, offset: Option<i64>) -> Result<Vec<Log>, Error> {
        schema::logs::table.limit(limit).offset(offset.unwrap_or_default()).get_results(connection).map_err(Into::into)
}
*/

pub fn raw_logs(connection: &PgConnection) -> Result<Vec<Log>, Error> {
    schema::logs::table
        .get_results(connection)
        .map_err(Into::into)
}
