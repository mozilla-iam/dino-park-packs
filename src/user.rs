use cis_profile::schema::Profile;
use failure::format_err;
use failure::Error;
use std::convert::TryFrom;
use uuid::Uuid;

pub struct User {
    pub user_uuid: Uuid,
}

impl Default for User {
    fn default() -> Self {
        User {
            user_uuid: Uuid::nil(),
        }
    }
}

impl TryFrom<&Profile> for User {
    type Error = Error;

    fn try_from(profile: &Profile) -> Result<User, Self::Error> {
        Ok(User {
            user_uuid: profile
                .uuid
                .value
                .clone()
                .ok_or_else(|| format_err!("no uuid"))
                .and_then(|uuid| Uuid::parse_str(&uuid).map_err(Into::into))?,
        })
    }
}
