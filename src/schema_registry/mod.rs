
use std::path::PathBuf;

use anyhow::{bail, Context};

use schema_registry_converter::async_impl::easy_avro::{EasyAvroDecoder, EasyAvroEncoder};
use schema_registry_converter::async_impl::easy_json::{EasyJsonDecoder, EasyJsonEncoder};

use schema_registry_converter::async_impl::easy_proto_raw::{
    EasyProtoRawDecoder, EasyProtoRawEncoder,
};
use schema_registry_converter::async_impl::proto_raw::RawDecodeResult;
use schema_registry_converter::async_impl::schema_registry as sr;
use schema_registry_converter::async_impl::schema_registry::SrSettings;
use schema_registry_converter::avro_common::DecodeResult;
use schema_registry_converter::schema_registry_common::{
    RegisteredReference, SchemaType, SubjectNameStrategy, SuppliedSchema,
};
use serde_json::Value as JsonValue;

use crate::cli::CodecKind;
use crate::config::KToolsContext;

pub mod config;
pub struct SchemaRegistryClient {
    inner: SrSettings,
    avro_decoder: EasyAvroDecoder,
    avro_encoder: EasyAvroEncoder,
    proto_decoder: EasyProtoRawDecoder,
    proto_encoder: EasyProtoRawEncoder,
    json_decoder: EasyJsonDecoder,
    json_encoder: EasyJsonEncoder,
}

impl SchemaRegistryClient {
    pub fn configure(context: &KToolsContext) -> anyhow::Result<SchemaRegistryClient> {
        let sr_context = context
            .schema_registry
            .as_ref()
            .context("No schema registry configuration found")?;

        let mut builder = SrSettings::new_builder(sr_context.url.clone());

        if let Some(basic_auth) = &sr_context.basic_auth {
            builder.set_basic_authorization(
                basic_auth.username.as_ref(),
                basic_auth.password.as_deref(),
            );
        }

        let inner = builder.build()?;

        let avro_decoder = EasyAvroDecoder::new(inner.clone());
        let avro_encoder = EasyAvroEncoder::new(inner.clone());
        let proto_decoder = EasyProtoRawDecoder::new(inner.clone());
        let proto_encoder = EasyProtoRawEncoder::new(inner.clone());
        let json_decoder = EasyJsonDecoder::new(inner.clone());
        let json_encoder = EasyJsonEncoder::new(inner.clone());

        Ok(Self {
            inner,
            avro_decoder,
            avro_encoder,
            proto_decoder,
            proto_encoder,
            json_decoder,
            json_encoder,
        })
    }

    pub async fn decode(
        &self,
        codec: CodecKind,
        payload: Option<&[u8]>,
    ) -> anyhow::Result<Option<JsonValue>> {
        match codec {
            CodecKind::Proto => {
                if payload.is_none() {
                    return Ok(None);
                }

                if let Some(RawDecodeResult { bytes, .. }) =
                    self.proto_decoder.decode(payload).await?
                {
                    let json = JsonValue::try_from(bytes)?;
                    return Ok(Some(json));
                }

                Ok(None)
            }
            CodecKind::Avro => {
                if payload.is_none() {
                    return Ok(None);
                }

                let DecodeResult { value, .. } = self.avro_decoder.decode(payload).await?;
                let json = JsonValue::try_from(value)?;
                Ok(Some(json))
            }
            CodecKind::Json => {
                if payload.is_none() {
                    return Ok(None);
                }

                if let Some(res) = self.json_decoder.decode(payload).await? {
                    return Ok(Some(res.value));
                }

                Ok(None)
            }
            CodecKind::Raw => {
                if payload.is_none() {
                    return Ok(None);
                }

                let json = JsonValue::try_from(payload)?;
                Ok(Some(json))
            }
        }
    }

    pub async fn encode(
        &self,
        codec: CodecKind,
        topic: &str,
        payload: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        let strategy = SubjectNameStrategy::TopicNameStrategy(topic.to_string(), false);

        let bytes = match codec {
            CodecKind::Proto => {
                self.proto_encoder
                    .encode_single_message(payload, strategy)
                    .await?
            }
            CodecKind::Avro => {
                let json: JsonValue = serde_json::from_slice(payload)?;

                println!("sending {:?}", json);

                self.avro_encoder.encode_struct(&json, &strategy).await?
            }
            CodecKind::Json => {
                let json: JsonValue = serde_json::from_slice(payload)?;

                self.json_encoder.encode(&json, strategy).await?
            }
            CodecKind::Raw => payload.to_vec(),
        };

        Ok(bytes)
    }

    pub async fn get_schema(&self, subject: &str, version: Option<u32>) -> anyhow::Result<String> {
        let versions = self.get_subject_versions(subject).await?;

        let search_version = match version {
            Some(version) => {
                if versions.contains(&version) {
                    version
                } else {
                    bail!(
                        "Subject {} does not have version {}. Subject versions: {}.",
                        subject,
                        version,
                        versions
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    );
                }
            }
            None => *versions.last().context("No versions found")?,
        };

        let schema = sr::get_referenced_schema(
            &self.inner,
            &RegisteredReference {
                subject: subject.to_string(),
                version: search_version,
                name: String::new(),
            },
        )
        .await?;

        Ok(schema.schema)
    }

    pub async fn get_subject_versions(&self, subject: &str) -> anyhow::Result<Vec<u32>> {
        let schema = sr::get_all_versions(&self.inner, subject.to_owned()).await?;

        Ok(schema)
    }

    pub async fn register_schema(&self, subject: &str, schema: &PathBuf) -> anyhow::Result<u32> {
        let schema_type = match schema
            .extension()
            .context("No extension found")?
            .to_str()
            .context("Invalid extension")?
        {
            "avsc" => SchemaType::Avro,
            "json" => SchemaType::Json,
            "proto" => SchemaType::Protobuf,
            other => bail!("Unsupported schema type: {}", other),
        };

        let schema = std::fs::read_to_string(schema)?;

        let supplied_schema = SuppliedSchema {
            name: None,
            schema_type,
            schema,
            references: vec![],
        };

        let result = sr::post_schema(&self.inner, subject.to_owned(), supplied_schema).await?;

        Ok(result.id)
    }
}
