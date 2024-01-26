use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "ktools",
    version = "0.1.0",
    author = "Guilherme Favero Ferreira <guifaveroferreira@gmail.com>",
    about = "KTools is a command line tool tweaking with Kafka and Schema Registry"
)]
pub struct KToolsCliArgs {
    #[command(flatten)]
    pub options: Options,

    #[command(subcommand)]
    pub command: Command,
}

/// Global arguments to be used in all subcommands
#[derive(Parser)]
pub struct Options {
    /// Specify the context to be used with the command
    #[clap(short, long)]
    pub context: Option<String>,
}

#[derive(Parser)]
pub enum Command {
    /// Manage KTools configuration file
    #[command(subcommand)]
    Config(ConfigCommand),

    /// Create a consumer or producer for a given topic
    #[command(subcommand)]
    Kafka(KafkaCommand),

    /// Manage the schema registry, including subjects and schemas
    #[command(subcommand)]
    SchemaRegistry(SchemaRegistryCommand),
}

#[derive(Parser)]
pub enum ConfigCommand {
    /// Open the global configuration file for editing
    Edit,
}

#[derive(Parser)]
pub enum SchemaRegistryCommand {
    /// Download a schema from the schema registry
    Download {
        #[arg(short, long)]
        subject: String,

        #[arg(short, long)]
        version: Option<u32>,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Upload a schema to the schema registry
    Register {
        #[arg(short, long)]
        subject: String,

        #[arg(short, long)]
        schema: PathBuf,
    },

    /// Compare a schema with the one in the schema registry
    Diff {
        /// The subject of the schema to be compared
        #[arg(short, long)]
        subject: String,

        /// The version of the schema to be compared
        #[arg(short, long)]
        version: Option<u32>,

        /// The file containing the schema to be compared
        #[arg(short, long)]
        schema: PathBuf,
    },
}

#[derive(Parser)]
pub enum KafkaCommand {
    /// Listen to kafka topic and print or save the messages
    Consume {
        /// The topic to be consumed
        #[arg(short, long)]
        topic: String,
    },

    /// Send messages to a kafka topic
    Produce {
        /// The topic to produce the messages
        #[arg(short, long)]
        topic: String,

        /// The message to be sent
        #[arg(long, conflicts_with = "payload")]
        message: Option<String>,

        /// The file containing the message to be sent
        #[arg(long, conflicts_with = "message")]
        payload: Option<PathBuf>,
    },
}
