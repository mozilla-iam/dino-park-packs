#[derive(Fail, Debug, PartialEq)]

pub enum PacksError {
    #[fail(display = "last_admin_of_group")]
    LastAdmin,
    #[fail(display = "error_deleting_members")]
    ErrorDeletingMembers,
    #[fail(display = "profile_not_found: {}. ({})", _0, _1)]
    ProfileNotFound(String, String),
    #[fail(display = "group_name_exists")]
    GroupNameExists,
    #[fail(display = "invalid_group_data")]
    InvalidGroupData,
    #[fail(display = "invalid_group_name")]
    InvalidGroupName,
    #[fail(display = "no_primary_email")]
    NoPrimaryEmail,
    #[fail(display = "no_uuid")]
    NoUuid,
}
