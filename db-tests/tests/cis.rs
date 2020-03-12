use cis_client::getby::GetBy;
use cis_client::AsyncCisClientTrait;
use cis_client::CisFut;
use cis_profile::crypto::SecretStore;
use cis_profile::schema::Profile;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

pub struct CisFakeClient {
    pub store: Arc<RwLock<HashMap<String, Profile>>>,
}

impl CisFakeClient {
    pub fn new() -> Self {
        CisFakeClient {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[allow(unused_variables)]
impl AsyncCisClientTrait for CisFakeClient {
    fn get_user_by(&self, id: &str, by: &GetBy, filter: Option<&str>) -> CisFut<Profile> {
        unimplemented!()
    }
    fn get_inactive_user_by(&self, id: &str, by: &GetBy, filter: Option<&str>) -> CisFut<Profile> {
        unimplemented!()
    }
    fn update_user(&self, id: &str, profile: Profile) -> CisFut<Value> {
        unimplemented!()
    }
    fn update_users(&self, profiles: &[Profile]) -> CisFut<Value> {
        unimplemented!()
    }
    fn delete_user(&self, id: &str, profile: Profile) -> CisFut<Value> {
        unimplemented!()
    }
    fn get_secret_store(&self) -> &SecretStore {
        unimplemented!()
    }
}
