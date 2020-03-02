#[derive(Fail, Debug, PartialEq)]
pub enum RuleError {
    #[fail(display = "Unknown DB error")]
    DBError,
    #[fail(display = "No DB connection available")]
    PoolError,
    #[fail(display = "Not allowed to join groups")]
    NotAllowedToJoinGroup,
    #[fail(display = "Not allowed to create groups")]
    NotAllowedToCreateGroups,
    #[fail(display = "Not allowed to invite member")]
    NotAllowedToInviteMember,
    #[fail(display = "Not allowed to remove member")]
    NotAllowedToRemoveMember,
    #[fail(display = "Not an admin")]
    NotAnAdmin,
    #[fail(display = "Not a curator")]
    NotACurator,
    #[fail(display = "Not allowed to edit terms")]
    NotAllowedToEditTerms,
    #[fail(display = "Never allowed (only admins)")]
    NeverAllowed,
    #[fail(display = "Invalid rule context")]
    InvalidRuleContext,
    #[fail(display = "User not found")]
    UserNotFound,
    #[fail(display = "Already a member")]
    AlreadyMember,
    #[fail(display = "Invalid group name")]
    InvalidGroupName,
}
