use std::collections::HashMap;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::kafka::config::KafkaConfig;
use crate::schema_registry::config::{BasicAuth, SchemaRegistryConfig};

/// Load the global configuration from the default location.
///
/// The default location is `~/.config/ktools/config`.
#[derive(Debug, Serialize, Deserialize)]
pub struct KToolsConfig {
    pub user: String,
    pub contexts: HashMap<String, KToolsContext>,
}

impl KToolsConfig {
    pub fn load_global() -> anyhow::Result<Self> {
        // verify if the config file exists
        let config_file = dirs::config_dir()
            .context("Could not find the config directory")?
            .join("ktools")
            .join("config");

        if !config_file.exists() {
            let config = Self::default();

            let parent_dir = config_file
                .parent()
                .context("Could not find the configuration parent directory")?;

            std::fs::create_dir_all(parent_dir)?;
            std::fs::write(config_file, serde_yaml::to_string(&config)?)?;

            return Ok(config);
        }

        let config = std::fs::read_to_string(config_file)?;

        Ok(serde_yaml::from_str(&config)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KToolsContext {
    pub kafka: Option<KafkaConfig>,
    pub schema_registry: Option<SchemaRegistryConfig>,
}

impl Default for KToolsConfig {
    fn default() -> Self {
        Self {
            user: "guilherme.ferreira".into(),
            contexts: HashMap::from([
                (
                    "local".into(),
                    KToolsContext {
                        kafka: Some(KafkaConfig {
                            bootstrap_server: "localhost:9092".into(),
                            properties: None,
                        }),
                        schema_registry: Some(SchemaRegistryConfig {
                            url: "http://localhost:8081".into(),
                            basic_auth: Some(BasicAuth {
                                username: "admin".into(),
                                password: "admin".into(),
                            }),
                        }),
                    },
                ),
                (
                    "dev".into(),
                    KToolsContext {
                        kafka: Some(KafkaConfig {
                            bootstrap_server: "localhost:9092".into(),
                            properties: None,
                        }),
                        schema_registry: Some(SchemaRegistryConfig {
                            url: "http://localhost:8081".into(),
                            basic_auth: Some(BasicAuth {
                                username: "admin".into(),
                                password: "admin".into(),
                            }),
                        }),
                    },
                ),
            ]),
        }
    }
}
