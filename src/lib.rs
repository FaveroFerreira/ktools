use anyhow::bail;
use clap::Parser;
use cli::GlobalArgs;

use crate::cli::{Command, KToolsCliArgs, KafkaCommand, SchemaRegistryCommand};
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
        }
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
