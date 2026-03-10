// DEBT: See DEBT later in this file. Applying this to the struct did not
// resolve the warning, so here we are.
#![allow(clippy::misnamed_getters)]

use crate::db::model::Group;
use crate::db::schema::*;
use crate::db::types::*;
use chrono::NaiveDateTime;
use log::error;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;

#[derive(Identifiable, Associations, Queryable, PartialEq, Eq, Debug, Insertable, AsChangeset)]
#[belongs_to(Group)]
#[primary_key(group_id)]
#[table_name = "logs"]
pub struct InsertLog {
    pub ts: Option<NaiveDateTime>,
    pub target: LogTargetType,
    pub operation: LogOperationType,
    pub group_id: i32,
    pub host_uuid: Uuid,
    pub user_uuid: Option<Uuid>,
    pub ok: bool,
    pub body: Option<Value>,
}

// DEBT: `Identifiable` uses the `primary_key`. Quoting the docs:
//
//     This trait can be automatically derived by adding #[derive(Identifiable)] to
//     your struct. By default, the “id” field is assumed to be a single field
//     called id. If it’s not, you can put #[primary_key(your_id)] on your struct.
//     If you have a composite primary key, the syntax is #[primary_key(id1, id2)].
//
// It's unclear why we _also_ specify an `id` field here. We can't remove it
// without also generating a migration, which is out of scope for IAM-1908.
#[derive(Identifiable, Associations, Queryable, PartialEq, Eq, Debug, Serialize)]
#[belongs_to(Group)]
#[primary_key(group_id)]
#[table_name = "logs"]
pub struct Log {
    // DEBT: consider removing this in a later migration.
    pub id: i32,
    pub ts: NaiveDateTime,
    pub target: LogTargetType,
    pub operation: LogOperationType,
    pub group_id: i32,
    pub host_uuid: Uuid,
    pub user_uuid: Option<Uuid>,
    pub ok: bool,
    pub body: Option<Value>,
}

pub fn log_comment_body(comment: &str) -> Option<Value> {
    Some(json!({ "comment": comment }))
}

pub fn add_to_comment_body(key: &str, value: &str, body: Option<Value>) -> Option<Value> {
    let body = match body {
        Some(Value::Object(mut o)) => {
            o.insert(key.into(), value.into());
            o.into()
        }
        None => json!({ key: value }),
        Some(v) => {
            error!("Trying to modify a non-object log comment");
            v
        }
    };
    Some(body)
}

pub struct LogContext {
    pub group_id: i32,
    pub host_uuid: Uuid,
    pub user_uuid: Option<Uuid>,
}

impl LogContext {
    pub fn with(group_id: i32, host_uuid: Uuid) -> Self {
        LogContext {
            group_id,
            host_uuid,
            user_uuid: None,
        }
    }
    pub fn with_user(mut self, user_uuid: Uuid) -> Self {
        self.user_uuid = Some(user_uuid);
        self
    }
}
