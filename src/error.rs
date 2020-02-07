#[derive(Fail, Debug, PartialEq)]
pub enum PacksError {
    #[fail(display = "Last admin of the group")]
    LastAdmin,
    #[fail(display = "Error deleting members")]
    ErrorDeletingMembers,
    #[fail(display = "Profile not found")]
    ProfileNotFound,
    #[fail(display = "Group name exists")]
    GroupNameExists,
}
