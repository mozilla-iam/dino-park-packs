#[derive(Fail, Debug, PartialEq)]
pub enum DBError {
    #[fail(display = "User profile v2 is invalid")]
    InvalidProfile,
}
