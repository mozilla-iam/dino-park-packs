use crate::db::logs::Log;
use crate::db::logs::LogContext;
use crate::db::schema;
use crate::db::types::LogOperationType;
use crate::db::types::LogTargetType;
use diesel::prelude::*;
use diesel::PgConnection;
use log::error;
use serde_json::Value;

pub fn db_log(
    connection: &PgConnection,
    ctx: &LogContext,
    target: LogTargetType,
    operation: LogOperationType,
    body: Option<Value>,
) {
    let log = Log {
        ts: None,
        target,
        operation,
        group_id: ctx.group_id,
        host_uuid: ctx.host_uuid,
        user_uuid: ctx.user_uuid,
        ok: true,
        body: body.unwrap_or_default(),
    };
    if let Err(e) = diesel::insert_into(schema::logs::table)
        .values(&log)
        .execute(connection)
    {
        error!("Failed to log operation: {}. Logentry: {:?}", e, log);
    }
}
