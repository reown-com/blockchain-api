use {serde::Deserialize, serde_piecewise_default::DeserializePiecewiseDefault};

#[derive(DeserializePiecewiseDefault, Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    pub s3_endpoint: Option<String>,
    pub export_bucket: Option<String>,
    pub geoip_db_bucket: Option<String>,
    pub geoip_db_key: Option<String>,
}
