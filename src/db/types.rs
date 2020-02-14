use crate::db::error::DBError;
use cis_profile::schema::Display;
use dino_park_trust::Trust;
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
}

impl TrustType {
    pub fn ndaed() -> Self {
        Self::Ndaed
    }
}

impl From<Trust> for TrustType {
    fn from(t: Trust) -> Self {
        match t {
            Trust::Staff => TrustType::Staff,
            Trust::Ndaed => TrustType::Ndaed,
            Trust::Vouched => TrustType::Vouched,
            Trust::Authenticated => TrustType::Authenticated,
            Trust::Public => TrustType::Public,
        }
    }
}

impl TryFrom<String> for TrustType {
    type Error = failure::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "staff" => Ok(TrustType::Staff),
            "ndaed" => Ok(TrustType::Ndaed),
            "vouched" => Ok(TrustType::Vouched),
            "authenticated" => Ok(TrustType::Authenticated),
            "public" => Ok(TrustType::Public),
            _ => Err(DBError::InvalidTrustLevel.into()),
        }
    }
}

impl TryFrom<Display> for TrustType {
    type Error = failure::Error;
    fn try_from(d: Display) -> Result<Self, Self::Error> {
        match d {
            Display::Staff => Ok(TrustType::Staff),
            Display::Ndaed => Ok(TrustType::Ndaed),
            Display::Vouched => Ok(TrustType::Vouched),
            Display::Authenticated => Ok(TrustType::Authenticated),
            Display::Public => Ok(TrustType::Public),
            _ => Err(DBError::InvalidTrustLevel.into()),
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
