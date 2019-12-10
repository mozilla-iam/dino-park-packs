#[derive(Fail, Debug, PartialEq)]
pub enum OperationError {
    #[fail(display = "Last admin of the group")]
    LastAdmin,
    #[fail(display = "Error deleting members")]
    ErrorDeletingMembers,
}
