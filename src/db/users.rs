use crate::db::schema::*;
use crate::db::types::TrustType;
use cis_profile::schema::Display;
use cis_profile::schema::Profile;
use cis_profile::schema::StandardAttributeString;
use uuid::Uuid;

#[derive(Identifiable, Queryable, PartialEq, Debug, Insertable, AsChangeset)]
#[primary_key(user_id)]
#[table_name = "user_ids"]
pub struct UserIdUuid {
    pub user_id: String,
    pub user_uuid: Uuid,
}

fn field_for_display(field: &StandardAttributeString, display: &Display) -> Option<String> {
    match &field.metadata.display {
        Some(field_display) if field_display <= display => field.value.clone(),
        _ => None,
    }
}

fn trust_for_profile(profile: &Profile) -> TrustType {
    if profile.staff_information.staff.value.unwrap_or_default() {
        return TrustType::Staff;
    }
    if profile
        .access_information
        .mozilliansorg
        .values
        .as_ref()
        .map(|groups| groups.0.contains_key("nda"))
        .unwrap_or_default()
    {
        return TrustType::Ndaed;
    }
    TrustType::Authenticated
}

macro_rules! user_t {
    ($user_typ:ident, $table:expr, $display:expr) => {
        #[derive(Identifiable, Queryable, PartialEq, Debug, Insertable, AsChangeset)]
        #[primary_key(user_uuid)]
        #[table_name = $table]
        pub struct $user_typ {
            pub user_uuid: Uuid,
            pub picture: Option<String>,
            pub first_name: Option<String>,
            pub last_name: Option<String>,
            pub username: String,
            pub email: Option<String>,
            pub trust: TrustType,
        }
        impl From<$user_typ> for DisplayUser {
            fn from(user: $user_typ) -> DisplayUser {
                DisplayUser {
                    user_uuid: user.user_uuid,
                    picture: user.picture,
                    first_name: user.first_name,
                    last_name: user.last_name,
                    username: user.username,
                    email: user.email,
                    trust: user.trust,
                }
            }
        }
        impl From<&Profile> for $user_typ {
            fn from(profile: &Profile) -> $user_typ {
                $user_typ {
                    user_uuid: Uuid::parse_str(
                        &profile.uuid.value.clone().unwrap_or_default()
                    ).unwrap_or_default(),
                    picture: field_for_display(&profile.picture, &$display),
                    first_name: field_for_display(&profile.first_name, &$display),
                    last_name: field_for_display(&profile.last_name, &$display),
                    username: profile.primary_username.value.clone().unwrap_or_default(),
                    email: field_for_display(&profile.primary_email, &$display),
                    trust: trust_for_profile(profile)
                }

            }
        }
    };
}

pub struct DisplayUser {
    pub user_uuid: Uuid,
    pub picture: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub trust: TrustType,
}

user_t!(UsersStaff, "users_staff", Display::Staff);
user_t!(UsersNdaed, "users_ndaed", Display::Ndaed);
user_t!(UsersVouched, "users_vouched", Display::Vouched);
user_t!(
    UsersAuthenticated,
    "users_authenticated",
    Display::Authenticated
);
user_t!(UsersPublic, "users_public", Display::Public);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_filter() {
        const UUID: &str = "5035efe5-c1cd-42bf-8148-d2a004e81ddc";
        const FIRST_NAME: &str = "Hans";
        const LAST_NAME: &str = "KNALL";
        const USERNAME: &str = "hans";
        const EMAIL: &str = "hans@knall.org";

        let mut profile = Profile::default();
        profile.uuid.value = Some(String::from(UUID));
        profile.first_name.value = Some(String::from(FIRST_NAME));
        profile.last_name.value = Some(String::from(LAST_NAME));
        profile.primary_username.value = Some(String::from(USERNAME));
        profile.primary_email.value = Some(String::from(EMAIL));

        profile.first_name.metadata.display = Some(Display::Private);
        profile.last_name.metadata.display = Some(Display::Staff);
        profile.primary_username.metadata.display = Some(Display::Ndaed);
        profile.primary_email.metadata.display = Some(Display::Public);

        let public = UsersPublic::from(&profile);
        let authenticated = UsersAuthenticated::from(&profile);
        let vouched = UsersVouched::from(&profile);
        let ndaed = UsersNdaed::from(&profile);
        let staff = UsersStaff::from(&profile);

        let uuid = Uuid::parse_str(UUID).unwrap_or_default();

        assert_eq!(public.user_uuid, uuid);
        assert_eq!(public.first_name, None);
        assert_eq!(public.last_name, None);
        assert_eq!(public.username, USERNAME);
        assert_eq!(public.email, Some(EMAIL.to_owned()));

        assert_eq!(authenticated.user_uuid, uuid);
        assert_eq!(authenticated.first_name, None);
        assert_eq!(authenticated.last_name, None);
        assert_eq!(authenticated.username, USERNAME);
        assert_eq!(authenticated.email, Some(EMAIL.to_owned()));

        assert_eq!(vouched.user_uuid, uuid);
        assert_eq!(vouched.first_name, None);
        assert_eq!(vouched.last_name, None);
        assert_eq!(vouched.username, USERNAME);
        assert_eq!(vouched.email, Some(EMAIL.to_owned()));

        assert_eq!(ndaed.user_uuid, uuid);
        assert_eq!(ndaed.first_name, None);
        assert_eq!(ndaed.last_name, None);
        assert_eq!(ndaed.username, USERNAME);
        assert_eq!(ndaed.email, Some(EMAIL.to_owned()));

        assert_eq!(staff.user_uuid, uuid);
        assert_eq!(staff.first_name, None);
        assert_eq!(staff.last_name, Some(LAST_NAME.to_owned()));
        assert_eq!(staff.username, USERNAME);
        assert_eq!(staff.email, Some(EMAIL.to_owned()));
    }
}
