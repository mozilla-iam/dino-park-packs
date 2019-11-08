use crate::error::PacksError;
use chrono::NaiveDateTime;
use chrono::Utc;
use std::time::Duration;

pub fn to_expiration_ts(duration: Duration) -> Result<NaiveDateTime, PacksError> {
    Ok((Utc::now()
        + chrono::Duration::from_std(duration).map_err(|_| PacksError::DurationConversionError)?)
    .naive_utc())
}
