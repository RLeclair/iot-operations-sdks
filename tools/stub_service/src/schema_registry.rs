// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod schema_registry_gen;
mod service;

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

pub use crate::schema_registry::service::Service;
use schema_registry_gen::schema_registry::service as service_gen;
use serde::{Deserialize, Serialize};

pub const CLIENT_ID: &str = "schema_registry_service_stub";
const NAMESPACE: &str = "aio-sr-ns-stub";
const SCHEMA_STATE_FILE: &str = "schema_state.json";

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct SchemaKey {
    content_hash: String,
    version: String,
}

impl From<service_gen::GetRequestSchema> for SchemaKey {
    fn from(get_request_schema: service_gen::GetRequestSchema) -> Self {
        Self {
            content_hash: get_request_schema.name.expect("Schema name is required"),
            version: get_request_schema
                .version
                .expect("Schema version is required"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Format {
    #[serde(rename = "Delta/1.0")]
    Delta1,
    #[serde(rename = "JsonSchema/draft-07")]
    JsonSchemaDraft07,
}

impl From<schema_registry_gen::schema_registry::service::Format> for Format {
    fn from(format: schema_registry_gen::schema_registry::service::Format) -> Self {
        match format {
            schema_registry_gen::schema_registry::service::Format::Delta1 => Format::Delta1,
            schema_registry_gen::schema_registry::service::Format::JsonSchemaDraft07 => {
                Format::JsonSchemaDraft07
            }
        }
    }
}

impl From<Format> for schema_registry_gen::schema_registry::service::Format {
    fn from(format: Format) -> Self {
        match format {
            Format::Delta1 => schema_registry_gen::schema_registry::service::Format::Delta1,
            Format::JsonSchemaDraft07 => {
                schema_registry_gen::schema_registry::service::Format::JsonSchemaDraft07
            }
        }
    }
}

/// Supported schema types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SchemaType {
    MessageSchema,
}

impl From<schema_registry_gen::schema_registry::service::SchemaType> for SchemaType {
    fn from(schema_type: schema_registry_gen::schema_registry::service::SchemaType) -> Self {
        match schema_type {
            schema_registry_gen::schema_registry::service::SchemaType::MessageSchema => {
                SchemaType::MessageSchema
            }
        }
    }
}

impl From<SchemaType> for schema_registry_gen::schema_registry::service::SchemaType {
    fn from(schema_type: SchemaType) -> Self {
        match schema_type {
            SchemaType::MessageSchema => {
                schema_registry_gen::schema_registry::service::SchemaType::MessageSchema
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Schema {
    /// Human-readable description of the schema.
    description: Option<String>,

    /// Human-readable display name.
    #[serde(rename = "displayName")]
    display_name: Option<String>,

    /// Format of the schema.
    format: Format,

    /// Hash of the schema content.
    hash: String,

    /// Schema name.
    name: String,

    /// Schema registry namespace. Uniquely identifies a schema registry within a tenant.
    pub namespace: String,

    /// Content stored in the schema.
    #[serde(rename = "schemaContent")]
    pub schema_content: String,

    /// Type of the schema.
    #[serde(rename = "schemaType")]
    pub schema_type: SchemaType,

    /// Schema tags.
    pub tags: HashMap<String, String>,

    /// Version of the schema. Allowed between 0-9.
    pub version: String,
}

impl From<Schema> for schema_registry_gen::schema_registry::service::Schema {
    fn from(schema: Schema) -> Self {
        Self {
            description: schema.description,
            display_name: schema.display_name,
            format: Some(schema.format.into()),
            hash: Some(schema.hash),
            name: Some(schema.name),
            namespace: Some(schema.namespace),
            schema_content: Some(schema.schema_content),
            schema_type: Some(schema.schema_type.into()),
            tags: Some(schema.tags),
            version: Some(schema.version),
        }
    }
}

impl Hash for service_gen::PutRequestSchema {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.schema_content.hash(state);
    }
}

impl From<service_gen::PutRequestSchema> for Schema {
    fn from(put_request_schema: service_gen::PutRequestSchema) -> Self {
        // Create the hash of the schema content
        let schema_hash = match &put_request_schema.schema_content {
            Some(content) => {
                let mut hasher = DefaultHasher::new();
                content.hash(&mut hasher);
                hasher.finish().to_string()
            }
            None => todo!(),
        };

        // Transfrom the put request schema into a Schema
        Self {
            description: put_request_schema.description,
            display_name: put_request_schema.display_name,
            format: put_request_schema
                .format
                .expect("Format is required")
                .into(),
            hash: schema_hash.clone(),
            name: schema_hash,
            namespace: NAMESPACE.to_string(),
            schema_content: put_request_schema
                .schema_content
                .expect("Schema content is required"),
            schema_type: put_request_schema
                .schema_type
                .expect("Schema type is required")
                .into(),
            tags: put_request_schema.tags.unwrap_or_default(),
            version: put_request_schema
                .version
                .expect("Schema version is required"),
        }
    }
}
