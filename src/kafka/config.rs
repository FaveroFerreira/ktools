use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KafkaConfig {
    #[serde(
        alias = "bootstrapServer",
        alias = "bootstrap-server",
        alias = "bootstrapServers",
        alias = "bootstrap-servers",
        alias = "bootstrap_servers"
    )]
    pub bootstrap_server: String,
    pub properties: Option<HashMap<String, String>>,
}
