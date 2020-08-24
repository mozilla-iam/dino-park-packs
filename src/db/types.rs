use crate::db::error::DBError;
use cis_profile::schema::Display;
use dino_park_trust::Trust;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
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
        TrustType::from(&t)
    }
}

impl From<&Trust> for TrustType {
    fn from(t: &Trust) -> Self {
        match *t {
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

/// Custom order to skip database migration.
impl PartialOrd for RoleType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Admin, Self::Curator)
            | (Self::Admin, Self::Member)
            | (Self::Curator, Self::Member) => Some(Ordering::Greater),
            (Self::Member, Self::Curator)
            | (Self::Member, Self::Admin)
            | (Self::Curator, Self::Admin) => Some(Ordering::Less),
            _ => Some(Ordering::Equal),
        }
    }
}

impl RoleType {
    pub fn is_curator(&self) -> bool {
        match *self {
            Self::Admin | Self::Curator => true,
            Self::Member => false,
        }
    }
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
    Request,
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

    #[test]
    fn test_role_type_order() {
        assert!(RoleType::Admin > RoleType::Member);
    }
}
