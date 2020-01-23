use crate::utils::to_expiration_ts;
use chrono::NaiveDateTime;

pub fn map_expiration(expiration: Option<i32>, fallback: Option<i32>) -> Option<NaiveDateTime> {
    match expiration {
        Some(exp) if exp > 0 => Some(exp),
        Some(_) => None,
        None => fallback,
    }
    .map(to_expiration_ts)
}
