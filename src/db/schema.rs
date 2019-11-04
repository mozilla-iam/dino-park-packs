table! {
    use diesel::sql_types::*;
    use crate::types::*;

    groups (group_id) {
        group_id -> Int4,
        name -> Varchar,
        path -> Varchar,
        description -> Text,
        capabilities -> Array<Capability_type>,
        typ -> Group_type,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::types::*;

    invitations (invitation_id) {
        invitation_id -> Int4,
        group_id -> Int4,
        user_uuid -> Uuid,
        code -> Uuid,
        invitation_expiration -> Nullable<Timestamp>,
        group_expiration -> Nullable<Timestamp>,
        added_by -> Nullable<Uuid>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::types::*;

    memberships (user_uuid, group_id) {
        user_uuid -> Uuid,
        group_id -> Int4,
        role_id -> Int4,
        expiration -> Nullable<Timestamp>,
        added_by -> Nullable<Uuid>,
        added_ts -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::types::*;

    roles (role_id) {
        role_id -> Int4,
        group_id -> Int4,
        typ -> Role_type,
        name -> Varchar,
        permissions -> Array<Permission_type>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::types::*;

    terms (group_id) {
        group_id -> Int4,
        text -> Nullable<Text>,
    }
}

joinable!(invitations -> groups (group_id));
joinable!(memberships -> groups (group_id));
joinable!(memberships -> roles (role_id));
joinable!(roles -> groups (group_id));
joinable!(terms -> groups (group_id));

allow_tables_to_appear_in_same_query!(groups, invitations, memberships, roles, terms,);
