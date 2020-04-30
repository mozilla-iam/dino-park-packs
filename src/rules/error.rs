#[derive(Fail, Debug, PartialEq)]
pub enum RuleError {
    #[fail(display = "rule_unknown_db_error")]
    DBError,
    #[fail(display = "rule_no_db_connection_available")]
    PoolError,
    #[fail(display = "rule_not_reviewed")]
    NotReviewedGroup,
    #[fail(display = "rule_not_allowed_to_join_groups")]
    NotAllowedToJoinGroup,
    #[fail(display = "rule_not_allowed_to_create_groups")]
    NotAllowedToCreateGroups,
    #[fail(display = "rule_not_allowed_to_invite_member")]
    NotAllowedToInviteMember,
    #[fail(display = "rule_not_allowed_to_remove_member")]
    NotAllowedToRemoveMember,
    #[fail(display = "rule_not_an_admin")]
    NotAnAdmin,
    #[fail(display = "rule_not_a_curator")]
    NotACurator,
    #[fail(display = "rule_not_allowed_to_edit_terms")]
    NotAllowedToEditTerms,
    #[fail(display = "rule_never_allowed")]
    NeverAllowed,
    #[fail(display = "rule_invalid_context")]
    InvalidRuleContext,
    #[fail(display = "rule_user_not_found")]
    UserNotFound,
    #[fail(display = "rule_already_a_member")]
    AlreadyMember,
    #[fail(display = "rule_invalid_group_name")]
    InvalidGroupName,
}
