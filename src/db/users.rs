use crate::db::error::DBError;
use crate::db::schema::*;
use crate::db::types::RoleType;
use crate::db::types::TrustType;
use crate::rules::functions::is_nda_group;
use cis_profile::schema::Display;
use cis_profile::schema::Profile;
use cis_profile::schema::StandardAttributeString;
use serde::Serialize;
use serde_json::Value;
use std::convert::TryFrom;
use uuid::Uuid;

#[derive(Identifiable, Queryable, PartialEq, Debug, Insertable, AsChangeset)]
#[primary_key(user_uuid)]
#[table_name = "profiles"]
pub struct UserProfileSlim {
    pub user_uuid: Uuid,
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub trust: TrustType,
}

#[derive(Identifiable, Queryable, PartialEq, Debug, Insertable, AsChangeset)]
#[primary_key(user_uuid)]
#[table_name = "profiles"]
pub struct UserProfileValue {
    pub user_uuid: Uuid,
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub profile: Value,
    pub trust: TrustType,
}

pub struct UserProfile {
    pub user_uuid: Uuid,
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub profile: Profile,
    pub trust: TrustType,
}

impl TryFrom<UserProfile> for UserProfileValue {
    type Error = serde_json::Error;

    fn try_from(u: UserProfile) -> Result<Self, Self::Error> {
        Ok(UserProfileValue {
            user_uuid: u.user_uuid,
            user_id: u.user_id,
            email: u.email,
            username: u.username,
            profile: serde_json::to_value(u.profile)?,
            trust: u.trust,
        })
    }
}

impl TryFrom<UserProfileValue> for UserProfile {
    type Error = serde_json::Error;
    fn try_from(u: UserProfileValue) -> Result<Self, Self::Error> {
        Ok(UserProfile {
            user_uuid: u.user_uuid,
            user_id: u.user_id,
            email: u.email,
            username: u.username,
            profile: serde_json::from_value(u.profile)?,
            trust: u.trust,
        })
    }
}

impl TryFrom<Profile> for UserProfile {
    type Error = failure::Error;
    fn try_from(p: Profile) -> Result<Self, Self::Error> {
        let trust = trust_for_profile(&p);
        Ok(UserProfile {
            user_uuid: Uuid::parse_str(&p.uuid.value.clone().unwrap_or_default())?,
            user_id: p.user_id.value.clone().ok_or(DBError::InvalidProfile)?,
            email: p
                .primary_email
                .value
                .clone()
                .ok_or(DBError::InvalidProfile)?,
            username: p
                .primary_username
                .value
                .clone()
                .ok_or(DBError::InvalidProfile)?,
            profile: p,
            trust,
        })
    }
}

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

pub fn trust_for_profile(profile: &Profile) -> TrustType {
    if profile.staff_information.staff.value.unwrap_or_default() {
        return TrustType::Staff;
    }
    if profile
        .access_information
        .mozilliansorg
        .values
        .as_ref()
        .map(|groups| groups.0.keys().any(|k| is_nda_group(k)))
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
        #[changeset_options(treat_none_as_null="true")]
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

#[derive(Serialize)]
pub struct DisplayUser {
    pub user_uuid: Uuid,
    pub picture: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub trust: TrustType,
}

#[derive(Serialize, Queryable, PartialEq, Debug)]
pub struct UserForGroup {
    pub user_uuid: Uuid,
    pub picture: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: String,
    pub email: Option<String>,
    pub trust: TrustType,
    pub role: Option<RoleType>,
    pub invited: bool,
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

    const UUID: &str = "5035efe5-c1cd-42bf-8148-d2a004e81ddc";
    const FIRST_NAME: &str = "Hans";
    const LAST_NAME: &str = "KNALL";
    const USERNAME: &str = "hans";
    const EMAIL: &str = "hans@knall.org";

    fn some_profile() -> Profile {
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
        profile
    }

    #[test]
    fn test_filter_public() {
        let uuid = Uuid::parse_str(UUID).unwrap_or_default();
        let profile = some_profile();
        let public = UsersPublic::from(&profile);
        assert_eq!(public.user_uuid, uuid);
        assert_eq!(public.first_name, None);
        assert_eq!(public.last_name, None);
        assert_eq!(public.username, USERNAME);
        assert_eq!(public.email, Some(EMAIL.to_owned()));
    }

    #[test]
    fn test_filter_authenticated() {
        let uuid = Uuid::parse_str(UUID).unwrap_or_default();
        let profile = some_profile();
        let authenticated = UsersAuthenticated::from(&profile);
        assert_eq!(authenticated.user_uuid, uuid);
        assert_eq!(authenticated.first_name, None);
        assert_eq!(authenticated.last_name, None);
        assert_eq!(authenticated.username, USERNAME);
        assert_eq!(authenticated.email, Some(EMAIL.to_owned()));
    }

    #[test]
    fn test_filter_vouched() {
        let uuid = Uuid::parse_str(UUID).unwrap_or_default();
        let profile = some_profile();
        let vouched = UsersVouched::from(&profile);
        assert_eq!(vouched.user_uuid, uuid);
        assert_eq!(vouched.first_name, None);
        assert_eq!(vouched.last_name, None);
        assert_eq!(vouched.username, USERNAME);
        assert_eq!(vouched.email, Some(EMAIL.to_owned()));
    }

    #[test]
    fn test_filter_ndaed() {
        let uuid = Uuid::parse_str(UUID).unwrap_or_default();
        let profile = some_profile();
        let ndaed = UsersNdaed::from(&profile);
        assert_eq!(ndaed.user_uuid, uuid);
        assert_eq!(ndaed.first_name, None);
        assert_eq!(ndaed.last_name, None);
        assert_eq!(ndaed.username, USERNAME);
        assert_eq!(ndaed.email, Some(EMAIL.to_owned()));
    }

    #[test]
    fn test_filter_staff() {
        let uuid = Uuid::parse_str(UUID).unwrap_or_default();
        let profile = some_profile();
        let staff = UsersStaff::from(&profile);
        assert_eq!(staff.user_uuid, uuid);
        assert_eq!(staff.first_name, None);
        assert_eq!(staff.last_name, Some(LAST_NAME.to_owned()));
        assert_eq!(staff.username, USERNAME);
        assert_eq!(staff.email, Some(EMAIL.to_owned()));
    }
}
