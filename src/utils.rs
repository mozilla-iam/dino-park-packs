use chrono::NaiveDateTime;
use chrono::Utc;

pub fn to_expiration_ts(days: i32) -> NaiveDateTime {
    (Utc::now() + chrono::Duration::days(days as i64)).naive_utc()
}
