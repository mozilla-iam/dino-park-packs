table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    group_rules (rule_id, group_id) {
        rule_id -> Int4,
        group_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    groups (group_id) {
        group_id -> Int4,
        name -> Varchar,
        path -> Varchar,
        description -> Text,
        capabilities -> Array<Capability_type>,
        typ -> Group_type,
        trust -> Trust_type,
        group_expiration -> Nullable<Int4>,
        created -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    invitations (group_id, user_uuid) {
        group_id -> Int4,
        user_uuid -> Uuid,
        invitation_expiration -> Nullable<Timestamp>,
        group_expiration -> Nullable<Timestamp>,
        added_by -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    memberships (user_uuid, group_id) {
        user_uuid -> Uuid,
        group_id -> Int4,
        role_id -> Int4,
        expiration -> Nullable<Timestamp>,
        added_by -> Uuid,
        added_ts -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    profiles (user_uuid) {
        user_uuid -> Uuid,
        user_id -> Varchar,
        email -> Varchar,
        username -> Varchar,
        profile -> Jsonb,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

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
    use crate::db::types::*;

    rules (rule_id) {
        rule_id -> Int4,
        typ -> Rule_type,
        name -> Varchar,
        payload -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    terms (group_id) {
        group_id -> Int4,
        text -> Text,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    user_ids (user_id) {
        user_id -> Varchar,
        user_uuid -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    users_authenticated (user_uuid) {
        user_uuid -> Uuid,
        picture -> Nullable<Varchar>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        username -> Varchar,
        email -> Nullable<Varchar>,
        trust -> Trust_type,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    users_ndaed (user_uuid) {
        user_uuid -> Uuid,
        picture -> Nullable<Varchar>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        username -> Varchar,
        email -> Nullable<Varchar>,
        trust -> Trust_type,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    users_public (user_uuid) {
        user_uuid -> Uuid,
        picture -> Nullable<Varchar>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        username -> Varchar,
        email -> Nullable<Varchar>,
        trust -> Trust_type,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    users_staff (user_uuid) {
        user_uuid -> Uuid,
        picture -> Nullable<Varchar>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        username -> Varchar,
        email -> Nullable<Varchar>,
        trust -> Trust_type,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    users_vouched (user_uuid) {
        user_uuid -> Uuid,
        picture -> Nullable<Varchar>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        username -> Varchar,
        email -> Nullable<Varchar>,
        trust -> Trust_type,
    }
}

joinable!(group_rules -> groups (group_id));
joinable!(group_rules -> rules (rule_id));
joinable!(invitations -> groups (group_id));
joinable!(memberships -> groups (group_id));
joinable!(memberships -> roles (role_id));
joinable!(roles -> groups (group_id));
joinable!(terms -> groups (group_id));
joinable!(user_ids -> profiles (user_uuid));

allow_tables_to_appear_in_same_query!(
    group_rules,
    groups,
    invitations,
    memberships,
    profiles,
    roles,
    rules,
    terms,
    user_ids,
    users_authenticated,
    users_ndaed,
    users_public,
    users_staff,
    users_vouched,
);
