use std::fs;

use anyhow::{anyhow, bail, Context};
use clap::Parser;



use kafka::client::KafkaClient;
use serde_json::Value as JsonValue;

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
        let config = match KToolsConfig::load() {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Could not load the configuration file: {}", err);
                eprint!("Please, verify that the configuration file is valid.");
                KToolsConfig::edit()?;
                std::process::exit(1)
            }
        };

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
        let context = options.context.as_deref().ok_or(anyhow!(
            "No context specified. Please, specify a context with the --context flag."
        ))?;

        let context = self.config.contexts.get(context).with_context(|| {
            anyhow!(
                "Could not find the context {:?}, please, check your configuration file.",
                context
            )
        })?;

        let kafka_client = KafkaClient::configure(&self.config.user, context)?;

        match command {
            KafkaCommand::Consume { topic, decode } => {
                kafka_client.consume(&topic, decode).await?;
                Ok(())
            }
            KafkaCommand::Produce {
                topic,
                message,
                encode,
                payload,
                key,
            } => {
                let payload: JsonValue = match (message, payload) {
                    (Some(message), None) => {
                        serde_json::from_str(&message).context("Invalid JSON message")?
                    }
                    (None, Some(payload)) => {
                        let text = fs::read_to_string(payload)?;
                        serde_json::from_str(&text).context("Invalid JSON payload")?
                    }
                    _ => bail!("Either message or payload must be specified"),
                };

                kafka_client
                    .produce(encode, &topic, key, serde_json::to_vec(&payload)?)
                    .await?;

                Ok(())
            }
        }
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

                let formatted_schema = serde_json::to_string_pretty(&serde_json::from_str::<
                    serde_json::Value,
                >(&schema)?)?;
                let formatted_sr_schema =
                    serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(
                        &sr_schema,
                    )?)?;

                if formatted_schema == formatted_sr_schema {
                    println!("No differences found");
                    return Ok(());
                }

                let diff = similar::TextDiff::from_lines(&formatted_sr_schema, &formatted_schema);

                for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
                    if idx > 0 {
                        println!("{:-^1$}", "-", 80);
                    }
                    for op in group {
                        for change in diff.iter_inline_changes(op) {
                            let (sign, s) = match change.tag() {
                                similar::ChangeTag::Delete => ("-", console::Style::new().red()),
                                similar::ChangeTag::Insert => ("+", console::Style::new().green()),
                                similar::ChangeTag::Equal => (" ", console::Style::new().dim()),
                            };
                            print!(
                                "{}{} |{}",
                                console::style(Line(change.old_index())).dim(),
                                console::style(Line(change.new_index())).dim(),
                                s.apply_to(sign).bold(),
                            );
                            for (emphasized, value) in change.iter_strings_lossy() {
                                if emphasized {
                                    print!("{}", s.apply_to(value).underlined().on_black());
                                } else {
                                    print!("{}", s.apply_to(value));
                                }
                            }
                            if change.missing_newline() {
                                println!();
                            }
                        }
                    }
                }

                Ok(())
            }
        }
    }
}

struct Line(Option<usize>);

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}
