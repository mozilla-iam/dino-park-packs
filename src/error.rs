#[derive(Fail, Debug)]
pub enum PacksError {
    #[fail(display = "Failed to convert duration")]
    DurationConversionError
}