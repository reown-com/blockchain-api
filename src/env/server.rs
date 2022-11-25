use std::net::IpAddr;

use crate::utils;
use crate::utils::network::NetworkInterfaceError;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,

    pub external_ip: Option<IpAddr>,
}

impl ServerConfig {
    pub fn external_ip(&self) -> Result<IpAddr, NetworkInterfaceError> {
        self.external_ip
            .map(Ok)
            .unwrap_or_else(utils::network::find_public_ip_addr)
    }
}
// #############################################################################

fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_log_level() -> String {
    "INFO".to_string()
}
