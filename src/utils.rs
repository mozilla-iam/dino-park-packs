use chrono::NaiveDateTime;
use chrono::Utc;

pub fn to_expiration_ts(days: i32) -> NaiveDateTime {
    (Utc::now() + chrono::Duration::days(days as i64)).naive_utc()
}

pub fn valid_group_name(group_name: &str) -> bool {
    group_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
