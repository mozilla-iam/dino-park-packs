use cis_client::settings::CisSettings;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Packs {
    pub postgres_url: String,
    pub domain: String,
    pub catcher: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Basket {
    pub api_key: String,
    pub basket_url: Url,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub auth: String,
    pub cis: CisSettings,
    pub packs: Packs,
    pub basket: Option<Basket>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let file = env::var("DPP_SETTINGS").unwrap_or_else(|_| String::from(".settings"));
        let mut s = Config::new();
        s.merge(File::with_name(&file))?;
        s.merge(Environment::new().separator("__").prefix("dp"))?;
        s.try_into()
    }
}
