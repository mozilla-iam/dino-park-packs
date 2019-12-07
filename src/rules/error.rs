#[derive(Fail, Debug, PartialEq)]
pub enum RuleError {
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
}
