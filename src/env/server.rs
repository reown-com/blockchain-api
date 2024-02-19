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
    pub prometheus_port: u16,
    pub log_level: String,
    pub external_ip: Option<IpAddr>,
    pub s3_endpoint: Option<String>,
    pub blocked_countries: Vec<String>,
    pub geoip_db_bucket: Option<String>,
    pub geoip_db_key: Option<String>,
    pub testing_project_id: Option<String>,
    pub validate_project_id: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            prometheus_port: 4000,
            log_level: "INFO".to_string(),
            external_ip: None,
            s3_endpoint: None,
            blocked_countries: Vec::new(),
            geoip_db_bucket: None,
            geoip_db_key: None,
            testing_project_id: None,
            validate_project_id: true,
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
