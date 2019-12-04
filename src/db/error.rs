#[derive(Fail, Debug, PartialEq)]
pub enum DBError {
    #[fail(display = "User profile v2 is invalid")]
    InvalidProfile,
    #[fail(display = "Trust level not supported is invalid")]
    InvalidTurstLevel,
}
