use serde::Deserialize;

pub mod onrampurl;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorQueryParams {
    pub project_id: String,
}
