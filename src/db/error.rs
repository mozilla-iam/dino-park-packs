#[derive(Fail, Debug, PartialEq, Eq)]
pub enum DBError {
    #[fail(display = "db_invalid_profile_v2")]
    InvalidProfile,
    #[fail(display = "db_invalid_trust_level")]
    InvalidTrustLevel,
    #[fail(display = "not_applicable")]
    NotApplicable,
}
