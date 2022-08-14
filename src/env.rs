use crate::error;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    pub infura_project_id: String,
}

fn default_port() -> u16 {
    3000
}

fn default_log_level() -> String {
    "WARN".to_string()
}

pub fn get_config() -> error::Result<Config> {
    let config = envy::from_env::<Config>()?;
    Ok(config)
}
