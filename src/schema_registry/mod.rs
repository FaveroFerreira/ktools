use std::path::PathBuf;

use anyhow::{bail, Context};
use schema_registry_converter::async_impl::schema_registry as sr;
use schema_registry_converter::async_impl::schema_registry::{get_schema_by_subject, SrSettings};
use schema_registry_converter::schema_registry_common::{
    RegisteredReference, SchemaType, SubjectNameStrategy, SuppliedSchema,
};

use crate::config::KToolsContext;

pub mod config;

pub struct SchemaRegistryClient {
    inner: SrSettings,
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

        Ok(Self { inner })
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

        let schema = std::fs::read_to_string(&schema)?;

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
