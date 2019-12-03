use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[DieselType = "Rule_type"]
pub enum RuleType {
    Staff,
    Nda,
    Group,
    Custom,
}

#[derive(DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[DieselType = "Trust_type"]
pub enum TrustType {
    Public,
    Authenticated,
    Vouched,
    Ndaed,
    Staff,
    Private,
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

#[derive(DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[DieselType = "Role_type"]
pub enum RoleType {
    Member,
    Curator,
    Admin,
}

#[derive(DbEnum, Debug, Deserialize, PartialEq, Serialize)]
#[DieselType = "Group_type"]
pub enum GroupType {
    Open,
    Reviewd,
    Closed,
}
