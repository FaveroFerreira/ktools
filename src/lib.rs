use std::fs;

use anyhow::{anyhow, bail, Context};
use clap::Parser;

use crate::cli::{
    Command, ConfigCommand, KToolsCliArgs, KafkaCommand, Options, SchemaRegistryCommand,
};
use crate::config::KToolsConfig;
use crate::schema_registry::SchemaRegistryClient;

mod cli;
mod config;
mod kafka;
mod schema_registry;

pub struct KTools {
    config: KToolsConfig,
}

impl KTools {
    pub fn new() -> anyhow::Result<Self> {
        let config = KToolsConfig::load()?;

        Ok(Self { config })
    }

    pub async fn parse_args_and_run(self) -> anyhow::Result<()> {
        let args = KToolsCliArgs::parse();

        match args.command {
            Command::Kafka(command) => self.kafka(args.options, command).await,
            Command::SchemaRegistry(command) => self.schema_registry(args.options, command).await,
            Command::Config(command) => self.config(command).await,
        }
    }

    async fn config(self, command: ConfigCommand) -> anyhow::Result<()> {
        match command {
            ConfigCommand::Edit => {
                let cfg_path = KToolsConfig::path()?;

                if !cfg_path.exists() {
                    // Should never happen because we create the file if it doesn't exist
                    bail!("Could not find the config file at {:?}", cfg_path);
                }

                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());

                let status = std::process::Command::new(editor)
                    .arg(cfg_path)
                    .status()
                    .context("Failed to open the default system editor")?;

                if !status.success() {
                    bail!("Failed to open the default system editor");
                }
            }
        }

        Ok(())
    }

    async fn kafka(self, options: Options, command: KafkaCommand) -> anyhow::Result<()> {
        todo!()
    }

    async fn schema_registry(
        self,
        options: Options,
        command: SchemaRegistryCommand,
    ) -> anyhow::Result<()> {
        let context = options.context.as_deref().ok_or(anyhow!(
            "No context specified. Please, specify a context with the --context flag."
        ))?;

        let context_config = self.config.contexts.get(context).with_context(|| {
            anyhow!(
                "Could not find the context {:?}, please, check your configuration file.",
                context
            )
        })?;

        let schema_registry_client = SchemaRegistryClient::configure(context_config)?;

        match command {
            SchemaRegistryCommand::Download {
                subject,
                version,
                output,
            } => {
                let schema = schema_registry_client.get_schema(&subject, version).await?;

                match output {
                    Some(output) => {
                        std::fs::write(output, schema)?;
                    }
                    None => {
                        println!("{}", schema);
                    }
                }

                Ok(())
            }
            SchemaRegistryCommand::Register { subject, schema } => {
                let registered_version = schema_registry_client
                    .register_schema(&subject, &schema)
                    .await?;

                println!("Registered version: {}", registered_version);

                Ok(())
            }
            SchemaRegistryCommand::Diff {
                subject,
                version,
                schema,
            } => {
                let schema = fs::read_to_string(schema)?;
                let sr_schema = schema_registry_client.get_schema(&subject, version).await?;

                let diff = similar::TextDiff::from_chars(&sr_schema[..], &schema[..]);

                for change in diff.iter_all_changes() {
                    let sign = match change.tag() {
                        similar::ChangeTag::Delete => "-",
                        similar::ChangeTag::Insert => "+",
                        similar::ChangeTag::Equal => " ",
                    };
                    print!("{}{}", sign, change);
                }

                Ok(())
            }
        }
    }
}
