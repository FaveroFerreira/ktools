use std::collections::HashMap;

use anyhow::Context;
use futures::StreamExt;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Headers;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use rdkafka::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::{cli::CodecKind, config::KToolsContext, schema_registry::SchemaRegistryClient};

#[derive(Debug, Serialize, Deserialize)]
pub struct KafkaMessage {
    pub key: Option<String>,
    pub value: Option<JsonValue>,
    pub partition: i32,
    pub offset: i64,
    pub timestamp: i64,
    pub headers: HashMap<String, String>,
}

pub struct KafkaClient {
    producer: FutureProducer,
    consumer: StreamConsumer,
    schema_registry: SchemaRegistryClient,
}

impl KafkaClient {
    pub fn configure(user: &str, context: &KToolsContext) -> anyhow::Result<Self> {
        let ktools_kafka_config = context
            .kafka
            .as_ref()
            .context("No kafka configuration found")?;

        let group_id = format!("ktools-{}", user);

        let mut rdkafka_config = rdkafka::ClientConfig::new();
        rdkafka_config.set("bootstrap.servers", &ktools_kafka_config.bootstrap_server);
        rdkafka_config.set("client.id", "ktools-cli");
        rdkafka_config.set("group.id", group_id);

        if let Some(props) = &ktools_kafka_config.properties {
            for (key, value) in props {
                rdkafka_config.set(key, value);
            }
        }

        let schema_registry = SchemaRegistryClient::configure(context)?;

        Ok(Self {
            producer: rdkafka_config.create()?,
            consumer: rdkafka_config.create()?,
            schema_registry,
        })
    }

    pub async fn produce(
        self,
        codec: CodecKind,
        topic: &str,
        key: Option<String>,
        payload: Vec<u8>,
    ) -> anyhow::Result<()> {
        let key = key.as_ref().map(|k| k.as_bytes()).unwrap_or_default();
        let value = self.schema_registry.encode(codec, topic, &payload).await?;

        let record = FutureRecord::to(topic).key(key).payload(&value);

        if let Err((e, _)) = self.producer.send(record, Timeout::Never).await {
            eprintln!("Failed to produce message: {}", e);
        }

        Ok(())
    }

    pub async fn consume(self, topic: &str, decoding: CodecKind) -> anyhow::Result<()> {
        let consumer = self.consumer;
        consumer.subscribe(&[topic]).unwrap();

        let mut message_stream = consumer.stream();

        while let Some(Ok(message)) = message_stream.next().await {
            let payload = message.payload();
            let key = message
                .key()
                .map(|key| String::from_utf8_lossy(key).to_string());

            let headers = if let Some(headers) = message.headers() {
                let mut h = HashMap::new();

                for header in headers.iter() {
                    let key = String::from_utf8_lossy(header.key.as_bytes()).to_string();
                    let value = header
                        .value
                        .as_ref()
                        .map(|v| String::from_utf8_lossy(v).to_string())
                        .unwrap_or_default();

                    h.insert(key, value);
                }

                h
            } else {
                HashMap::new()
            };

            let payload = self.schema_registry.decode(decoding, payload).await?;

            let message = KafkaMessage {
                key,
                value: payload,
                partition: message.partition(),
                offset: message.offset(),
                timestamp: message.timestamp().to_millis().unwrap_or_default(),
                headers,
            };

            println!("{}", serde_json::to_string_pretty(&message)?);
        }

        Ok(())
    }
}
