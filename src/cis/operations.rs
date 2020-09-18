use crate::db::internal;
use crate::db::Pool;
use chrono::DateTime;
use chrono::Utc;
use cis_client::AsyncCisClientTrait;
use cis_profile::crypto::SecretStore;
use cis_profile::crypto::Signer;
use cis_profile::schema::AccessInformationProviderSubObject;
use cis_profile::schema::Display;
use cis_profile::schema::KeyValue;
use cis_profile::schema::Profile;
use cis_profile::schema::PublisherAuthority;
use failure::format_err;
use failure::Error;
use futures::TryFutureExt;
use log::warn;
use std::collections::BTreeMap;
use std::sync::Arc;
use uuid::Uuid;

fn update_groups_and_sign_values_field(
    field: &mut AccessInformationProviderSubObject,
    groups: Vec<String>,
    store: &SecretStore,
    now: &DateTime<Utc>,
) -> Result<(), Error> {
    if let Some(KeyValue(values)) = &field.values {
        field.values = Some(KeyValue({
            let mut btm = BTreeMap::new();
            for group in values.iter().filter_map(|(k, v)| match v {
                Some(s) if s.is_empty() => None,
                _ => Some(k),
            }) {
                btm.insert(group.clone(), None);
            }
            for group in groups {
                btm.insert(group, Some(String::default()));
            }
            btm
        }));
    } else {
        field.metadata.created = *now;
        field.values = Some(KeyValue({
            let mut btm = BTreeMap::new();
            for group in groups {
                btm.insert(group, Some(String::default()));
            }
            btm
        }));
    }
    if field.metadata.display.is_none() {
        field.metadata.display = Some(Display::Staff);
    }
    field.metadata.last_modified = *now;
    field.signature.publisher.name = PublisherAuthority::Mozilliansorg;
    store.sign_attribute(field)
}

fn insert_kv_and_sign_values_field(
    field: &mut AccessInformationProviderSubObject,
    kv: (String, Option<String>),
    store: &SecretStore,
    now: &DateTime<Utc>,
) -> Result<(), Error> {
    if let Some(KeyValue(ref mut values)) = &mut field.values {
        values.insert(kv.0, kv.1);
    } else {
        field.metadata.created = *now;
        field.values = Some(KeyValue({
            let mut btm = BTreeMap::new();
            btm.insert(kv.0, kv.1);
            btm
        }));
    }
    if field.metadata.display.is_none() {
        field.metadata.display = Some(Display::Staff);
    }
    field.metadata.last_modified = *now;
    field.signature.publisher.name = PublisherAuthority::Mozilliansorg;
    store.sign_attribute(field)
}

fn remove_kv_and_sign_values_field(
    field: &mut AccessInformationProviderSubObject,
    keys: &[&str],
    store: &SecretStore,
    now: &DateTime<Utc>,
) -> Result<(), Error> {
    if let Some(KeyValue(ref mut values)) = &mut field.values {
        let mut changed = false;
        for key in keys {
            if values.remove(*key).is_some() {
                changed = true;
            } else {
                warn!("group {} was not present when trying to delete", key);
            }
        }
        if changed {
            field.metadata.last_modified = *now;
            field.signature.publisher.name = PublisherAuthority::Mozilliansorg;
            return store.sign_attribute(field);
        }
    }
    warn!("groups {:?} where not present when trying to delete", keys);
    Ok(())
}

pub async fn _send_groups_to_cis(
    cis_client: Arc<impl AsyncCisClientTrait>,
    groups: Vec<String>,
    profile: Profile,
) -> Result<(), Error> {
    let now = &Utc::now();
    let mut update_profile = Profile::default();
    update_profile.access_information.mozilliansorg = profile.access_information.mozilliansorg;
    update_profile.active = profile.active;
    match update_groups_and_sign_values_field(
        &mut update_profile.access_information.mozilliansorg,
        groups,
        cis_client.get_secret_store(),
        &now,
    ) {
        Ok(_) => {
            if let Some(user_id) = profile.user_id.value.clone() {
                cis_client
                    .update_user(&user_id, update_profile)
                    .map_ok(|_| ())
                    .await
            } else {
                Err(format_err!("invalid user_id"))
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn send_groups_to_cis(
    pool: &Pool,
    cis_client: Arc<impl AsyncCisClientTrait>,
    user_uuid: &Uuid,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let user_profile = internal::user::user_profile_by_uuid(&connection, user_uuid)?;
    let groups = internal::member::group_names_for_user(&connection, user_uuid)?;
    drop(connection);
    _send_groups_to_cis(cis_client, groups, user_profile.profile).await
}

pub async fn add_group_to_profile(
    cis_client: Arc<impl AsyncCisClientTrait>,
    group_name: String,
    profile: Profile,
) -> Result<(), Error> {
    let now = &Utc::now();
    let mut update_profile = Profile::default();
    update_profile.access_information.mozilliansorg = profile.access_information.mozilliansorg;
    update_profile.active = profile.active;
    match insert_kv_and_sign_values_field(
        &mut update_profile.access_information.mozilliansorg,
        (group_name, Some(String::default())),
        cis_client.get_secret_store(),
        &now,
    ) {
        Ok(_) => {
            if let Some(user_id) = profile.user_id.value.clone() {
                cis_client
                    .update_user(&user_id, update_profile)
                    .map_ok(|_| ())
                    .await
            } else {
                Err(format_err!("invalid user_id"))
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn remove_group_from_profile(
    cis_client: Arc<impl AsyncCisClientTrait>,
    group_names: &[&str],
    profile: Profile,
) -> Result<(), Error> {
    if group_names.is_empty() {
        return Ok(());
    }
    let now = &Utc::now();
    let mut update_profile = Profile::default();
    update_profile.access_information.mozilliansorg = profile.access_information.mozilliansorg;
    update_profile.active = profile.active;
    match remove_kv_and_sign_values_field(
        &mut update_profile.access_information.mozilliansorg,
        group_names,
        cis_client.get_secret_store(),
        &now,
    ) {
        Ok(_) => {
            if let Some(user_id) = profile.user_id.value.clone() {
                log::debug!("updating profile");
                cis_client
                    .update_user(&user_id, update_profile)
                    .map_ok(|_| ())
                    .await
            } else {
                Err(format_err!("invalid user_id"))
            }
        }
        Err(e) => Err(e),
    }
}
