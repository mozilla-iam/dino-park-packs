use chrono::NaiveDateTime;
use chrono::Utc;

pub fn to_expiration_ts(days: i64) -> NaiveDateTime {
    (Utc::now() + chrono::Duration::days(days)).naive_utc()
}
