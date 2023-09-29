use {
    crate::{utils, utils::network::NetworkInterfaceError},
    serde::Deserialize,
    serde_piecewise_default::DeserializePiecewiseDefault,
    std::net::IpAddr,
};

#[derive(DeserializePiecewiseDefault, Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub private_port: u16,
    pub log_level: String,
    pub external_ip: Option<IpAddr>,
    pub blocked_countries: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            private_port: 4000,
            log_level: "INFO".to_string(),
            external_ip: None,
            blocked_countries: Vec::new(),
        }
    }
}

impl ServerConfig {
    pub fn external_ip(&self) -> Result<IpAddr, NetworkInterfaceError> {
        self.external_ip
            .map(Ok)
            .unwrap_or_else(utils::network::find_public_ip_addr)
    }
}
