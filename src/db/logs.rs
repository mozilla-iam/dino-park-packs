// FIXME: this check is firing on pub struct "Log" for unknown reasons. [IAM-1072]
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

#[derive(Identifiable, Associations, Queryable, PartialEq, Eq, Debug, Serialize)]
#[belongs_to(Group)]
#[primary_key(group_id)]
#[table_name = "logs"]
pub struct Log {
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
