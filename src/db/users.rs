use crate::db::schema::*;
use uuid::Uuid;

macro_rules! user_t {
    ($user_typ:ident, $table:expr) => {
        #[derive(Identifiable, Queryable, PartialEq, Debug)]
        #[primary_key(user_uuid)]
        #[table_name = $table]
        pub struct $user_typ {
            pub user_uuid: Uuid,
            pub first_name: Option<String>,
            pub last_name: Option<String>,
            pub username: Option<String>,
            pub email: Option<String>,
        }
        impl From<$user_typ> for DisplayUser {
            fn from(user: $user_typ) -> DisplayUser {
                DisplayUser {
                    user_uuid: user.user_uuid,
                    first_name: user.first_name,
                    last_name: user.last_name,
                    username: user.username,
                    email: user.email,
                }
            }
        }
    };
}

pub struct DisplayUser {
    pub user_uuid: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
}

user_t!(UsersStaff, "users_staff");
user_t!(UsersNdaed, "users_ndaed");
user_t!(UsersVouched, "users_vouched");
user_t!(UsersAuthenticated, "users_authenticated");
user_t!(UsersPublic, "users_public");
