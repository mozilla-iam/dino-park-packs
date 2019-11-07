use serde_derive::Serialize;

#[derive(DbEnum, Debug, PartialEq, Serialize)]
#[DieselType = "Rule_type"]
pub enum RuleType {
    Staff,
    Nda,
    Group,
    Custom,
}

#[derive(DbEnum, Debug, PartialEq, Serialize)]
#[DieselType = "Trust_type"]
pub enum TrustType {
    Public,
    Authenticated,
    Vouched,
    Ndaed,
    Staff,
    Private,
}

#[derive(DbEnum, Debug, PartialEq, Serialize)]
#[DieselType = "Permission_type"]
pub enum PermissionType {
    InviteMember,
    RemoveMember,
    AddCurator,
    RemoveCurator,
    EditDescription,
    ChangeName,
    DeleteGroup,
}

#[derive(DbEnum, Debug, PartialEq, Serialize)]
#[DieselType = "Capability_type"]
pub enum CapabilityType {
    Gdrive,
    Deiscourse,
}

#[derive(DbEnum, Debug, PartialEq, Serialize)]
#[DieselType = "Role_type"]
pub enum RoleType {
    Member,
    Curator,
    Admin,
}

#[derive(DbEnum, Debug, PartialEq, Serialize)]
#[DieselType = "Group_type"]
pub enum GroupType {
    Open,
    Reviewd,
    Closed,
}
