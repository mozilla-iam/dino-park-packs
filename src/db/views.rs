use crate::db::schema::*;

macro_rules! hosts_table {
    ($t:ident, $u:ident) => {
        table! {
            use diesel::sql_types::*;

            $t (user_uuid) {
                user_uuid -> Uuid,
                first_name -> Nullable<Varchar>,
                last_name -> Nullable<Varchar>,
                username -> Varchar,
                email -> Nullable<VarChar>,
            }
        }
        allow_tables_to_appear_in_same_query!($t, $u,);
        allow_tables_to_appear_in_same_query!($t, memberships,);
        allow_tables_to_appear_in_same_query!($t, groups,);
        allow_tables_to_appear_in_same_query!($t, roles,);
        allow_tables_to_appear_in_same_query!($t, invitations,);
        allow_tables_to_appear_in_same_query!($t, terms,);
        allow_tables_to_appear_in_same_query!($t, profiles,);
    };
}
hosts_table!(hosts_staff, users_staff);
hosts_table!(hosts_ndaed, users_ndaed);
hosts_table!(hosts_vouched, users_vouched);
hosts_table!(hosts_authenticated, users_authenticated);
hosts_table!(hosts_public, users_public);

table! {
    use diesel::sql_types::*;
    use crate::db::types::*;

    groups_list (name) {
        name -> VarChar,
        typ -> Group_type,
        trust -> Trust_type,
        members_count -> BigInt,
    }
}
