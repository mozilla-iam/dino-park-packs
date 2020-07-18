use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Deserializer;

fn unescape<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.replace("\\n", "\n"))
}

mod sql_date {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Deserialize)]
pub struct MozilliansGroupMembership {
    #[serde(with = "sql_date")]
    pub date_joined: DateTime<Utc>,
    #[serde(with = "sql_date")]
    pub updated_on: DateTime<Utc>,
    pub auth0_user_id: String,
    pub expiration: i32,
    pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct MozilliansGroup {
    pub name: String,
    pub expiration: i32,
    #[serde(deserialize_with = "unescape")]
    pub terms: String,
    #[serde(deserialize_with = "unescape")]
    pub description: String,
    #[serde(deserialize_with = "unescape")]
    pub invitation_email: String,
    pub typ: String,
    pub website: String,
    pub wiki: String,
}

#[derive(Debug, Deserialize)]
pub struct MozilliansGroupCurator {
    pub auth0_user_id: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use csv::ReaderBuilder;
    use failure::Error;

    #[test]
    fn test_membership_tsv() -> Result<(), Error> {
        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path("tests/data/import-test/m.tsv")?;
        rdr.deserialize()
            .collect::<Result<Vec<MozilliansGroupMembership>, csv::Error>>()?;
        Ok(())
    }

    #[test]
    fn test_group_tsv() -> Result<(), Error> {
        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path("tests/data/import-test/g.tsv")?;

        let group = rdr
            .deserialize()
            .collect::<Result<Vec<MozilliansGroup>, csv::Error>>()?;
        assert_eq!(group[0].invitation_email, "some \ninvitation email");
        Ok(())
    }

    #[test]
    fn test_curator_tsv() -> Result<(), Error> {
        let mut rdr = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path("tests/data/import-test/c.tsv")?;

        rdr.deserialize()
            .collect::<Result<Vec<MozilliansGroupCurator>, csv::Error>>()?;
        Ok(())
    }
}
