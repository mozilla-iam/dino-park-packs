use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use serde::Serialize;
use serde::Serializer;

pub fn to_expiration_ts(days: i32) -> NaiveDateTime {
    (Utc::now() + chrono::Duration::days(days as i64)).naive_utc()
}

pub fn valid_group_name(group_name: &str) -> bool {
    group_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() && c.is_lowercase() || c == '-' || c == '_')
}

pub fn maybe_to_utc<S>(naive: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match naive {
        Some(naive) => to_utc(naive, serializer),
        None => None::<DateTime<Utc>>.serialize(serializer),
    }
}

pub fn to_utc<S>(naive: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let dt = DateTime::<Utc>::from_utc(*naive, Utc);
    dt.serialize(serializer)
}

#[cfg(test)]
mod test {
    use super::*;
    use failure::Error;
    use serde_json;

    #[test]
    fn test_to_utc() -> Result<(), Error> {
        #[derive(Serialize)]
        struct DateWrapper {
            #[serde(serialize_with = "to_utc")]
            date: NaiveDateTime,
        }

        let d = DateWrapper {
            date: NaiveDateTime::from_timestamp(0, 0),
        };
        let v = serde_json::to_string(&d)?;
        assert_eq!(v, r#"{"date":"1970-01-01T00:00:00Z"}"#);
        Ok(())
    }

    #[test]
    fn test_maybe_to_utc() -> Result<(), Error> {
        #[derive(Serialize)]
        struct DateWrapper {
            #[serde(serialize_with = "maybe_to_utc")]
            date: Option<NaiveDateTime>,
        }

        let d = DateWrapper {
            date: Some(NaiveDateTime::from_timestamp(0, 0)),
        };
        let v = serde_json::to_string(&d)?;
        assert_eq!(v, r#"{"date":"1970-01-01T00:00:00Z"}"#);
        let d = DateWrapper { date: None };
        let v = serde_json::to_string(&d)?;
        assert_eq!(v, r#"{"date":null}"#);
        Ok(())
    }
}
