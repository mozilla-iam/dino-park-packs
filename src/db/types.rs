use crate::db::error::DBError;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::convert::TryFrom;

#[derive(DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[DieselType = "Rule_type"]
pub enum RuleType {
    Staff,
    Nda,
    Group,
    Custom,
}

#[derive(Clone, DbEnum, Debug, Deserialize, PartialEq, PartialOrd, Serialize)]
#[DieselType = "Trust_type"]
pub enum TrustType {
    Public,
    Authenticated,
    Vouched,
    Ndaed,
    Staff,
    Private,
}

impl TrustType {
    pub fn ndaed() -> Self {
        Self::Ndaed
    }
}

impl TryFrom<String> for TrustType {
    type Error = failure::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "private" => Ok(TrustType::Private),
            "staff" => Ok(TrustType::Staff),
            "ndaed" => Ok(TrustType::Ndaed),
            "vouched" => Ok(TrustType::Vouched),
            "authenticated" => Ok(TrustType::Authenticated),
            "public" => Ok(TrustType::Public),
            _ => Err(DBError::InvalidTurstLevel.into()),
        }
    }
}

#[derive(DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[DieselType = "Permission_type"]
pub enum PermissionType {
    InviteMember,
    EditDescription,
    AddCurator,
    RemoveCurator,
    DeleteGroup,
    RemoveMember,
    EditTerms,
}

#[derive(DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[DieselType = "Capability_type"]
pub enum CapabilityType {
    Gdrive,
    Deiscourse,
}

#[derive(Clone, DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[DieselType = "Role_type"]
pub enum RoleType {
    Admin,
    Curator,
    Member,
}

#[derive(DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[DieselType = "Group_type"]
pub enum GroupType {
    Open,
    Reviewed,
    Closed,
}

impl Default for GroupType {
    fn default() -> Self {
        Self::Closed
    }
}

#[derive(Copy, Clone, DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[DieselType = "Log_target_type"]
pub enum LogTargetType {
    Group,
    Terms,
    Membership,
    Role,
    Invitation,
}

#[derive(Copy, Clone, DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[DieselType = "Log_operation_type"]
pub enum LogOperationType {
    Created,
    Deleted,
    Updated,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_trust_type_order() {
        assert!(TrustType::Staff > TrustType::Public);
    }
}
