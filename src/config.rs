use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{bail, Context, Ok};
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
    pub fn edit() -> anyhow::Result<()> {
        let cfg_path = Self::path()?;

        if !cfg_path.exists() {
            Self::create()?;
        }

        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());

        let status = std::process::Command::new(editor)
            .arg(cfg_path)
            .status()
            .context("Failed to open the default system editor")?;

        if !status.success() {
            bail!("Failed to open the default system editor");
        }

        Ok(())
    }

    pub fn path() -> anyhow::Result<PathBuf> {
        let path = dirs::config_dir()
            .context("Could not find the config directory")?
            .join("ktools")
            .join("config");

        Ok(path)
    }

    pub fn load() -> anyhow::Result<Self> {
        let cfg_path = Self::path()?;

        if !cfg_path.exists() {
            Self::create()?;
        }

        let config = std::fs::read_to_string(cfg_path)?;

        Ok(serde_yaml::from_str(&config)?)
    }

    pub fn create() -> anyhow::Result<()> {
        let cfg_path = Self::path()?;

        if !cfg_path.exists() {
            let config = Self::default();

            let parent_dir = cfg_path
                .parent()
                .context("Could not find the configuration parent directory")?;

            std::fs::create_dir_all(parent_dir)?;
            std::fs::write(cfg_path, serde_yaml::to_string(&config)?)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KToolsContext {
    pub kafka: Option<KafkaConfig>,
    #[serde(alias = "schemaRegistry", alias = "schema-registry")]
    pub schema_registry: Option<SchemaRegistryConfig>,
}

impl Default for KToolsConfig {
    fn default() -> Self {
        Self {
            user: "user.name".into(),
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
                            basic_auth: None,
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
                                password: Some("admin".into()),
                            }),
                        }),
                    },
                ),
            ]),
        }
    }
}
