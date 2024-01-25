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
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Command,
}

/// Global arguments to be used in all subcommands
#[derive(Parser)]
pub struct GlobalArgs {
    /// Specify the name of the project (overrides crate name)
    #[clap(long, value_parser, num_args = 1.., value_delimiter = ',')]
    pub contexts: Vec<String>,
}

#[derive(Parser)]
pub enum Command {
    /// Create a consumer or producer for a given topic
    #[command(subcommand)]
    Kafka(KafkaCommand),

    /// Manage the schema registry, including subjects and schemas
    #[command(subcommand)]
    SchemaRegistry(SchemaRegistryCommand),
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
        #[arg(short, long)]
        subject: String,

        #[arg(short, long)]
        version: Option<u32>,

        #[arg(short, long)]
        schema: PathBuf,
    },
}

#[derive(Parser)]
pub enum KafkaCommand {
    /// Listen to kafka topic and print or save the messages
    Consume {
        #[arg(short, long)]
        topic: String,
    },

    /// Send messages to a kafka topic
    Produce {
        #[arg(short, long)]
        topic: String,

        #[arg(long, conflicts_with = "payload")]
        message: Option<String>,

        #[arg(long, conflicts_with = "message")]
        payload: Option<PathBuf>,
    },
}
