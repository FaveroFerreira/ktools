use anyhow::{bail, Context};
use clap::Parser;

use crate::cli::{
    Command, ConfigCommand, GlobalArgs, KToolsCliArgs, KafkaCommand, SchemaRegistryCommand,
};
use crate::config::{KToolsConfig, KToolsContext};
use crate::kafka::client::KafkaClient;

mod cli;
mod config;
mod kafka;
mod schema_registry;

pub struct KTools {
    config: KToolsConfig,
}

impl KTools {
    pub fn new() -> anyhow::Result<Self> {
        let config = KToolsConfig::load_global()?;

        Ok(Self { config })
    }

    pub async fn parse_args_and_run(self) -> anyhow::Result<()> {
        let cli = KToolsCliArgs::parse();

        match cli.command {
            Command::Kafka(command) => self.kafka(cli.global, command).await,
            Command::SchemaRegistry(command) => self.schema_registry(cli.global, command).await,
            Command::Config(command) => self.config(cli.global, command).await,
        }
    }

    async fn config(self, args: GlobalArgs, command: ConfigCommand) -> anyhow::Result<()> {
        match command {
            ConfigCommand::Edit => {
                let config_file = dirs::config_dir()
                    .context("Could not find the config directory")?
                    .join("ktools")
                    .join("config");

                if !config_file.exists() {
                    bail!("Could not find the config file at {:?}", config_file);
                }

                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());

                let status = std::process::Command::new(editor)
                    .arg(config_file)
                    .status()
                    .context("Failed to open the default system editor")?;

                if !status.success() {
                    bail!("Failed to open the default system editor");
                }
            }
        }

        Ok(())
    }

    async fn kafka(self, args: GlobalArgs, command: KafkaCommand) -> anyhow::Result<()> {
        todo!()
    }

    async fn schema_registry(
        self,
        args: GlobalArgs,
        command: SchemaRegistryCommand,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
