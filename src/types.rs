#[derive(DbEnum, Debug, PartialEq)]
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

#[derive(DbEnum, Debug, PartialEq)]
#[DieselType = "Capability_type"]
pub enum CapabilityType {
    Gdrive,
    Deiscourse,
}

#[derive(DbEnum, Debug, PartialEq)]
#[DieselType = "Role_type"]
pub enum RoleType {
    Member,
    Curator,
    Admin,
}
