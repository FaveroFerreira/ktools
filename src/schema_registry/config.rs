use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SchemaRegistryConfig {
    pub url: String,
    #[serde(alias = "basicAuth", alias = "basic-auth")]
    pub basic_auth: Option<BasicAuth>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: Option<String>,
}
