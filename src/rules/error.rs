#[derive(Fail, Debug, PartialEq)]
pub enum RuleError {
    #[fail(display = "Not allowed to create groups")]
    NotAllowedToCreateGroups,
    #[fail(display = "Not allowed to invite member")]
    NotAllowedToInviteMember,
    #[fail(display = "Not an admin")]
    NotAnAdmin,
    #[fail(display = "Not a curator")]
    NotACurator,
}
