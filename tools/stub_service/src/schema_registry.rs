// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod schema_registry_gen;
mod service;

use std::hash::{DefaultHasher, Hash, Hasher};

pub use crate::schema_registry::service::Service;
pub use schema_registry_gen::schema_registry::service::Schema;
use schema_registry_gen::schema_registry::service::{GetRequestSchema, PutRequestSchema};

pub const CLIENT_ID: &str = "schema_registry_service_stub";
const NAMESPACE: &str = "aio-sr-ns-stub";
const SCHEMA_STATE_FILE: &str = "schema_state.json";

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct SchemaKey {
    content_hash: String,
    version: String,
}

impl From<GetRequestSchema> for SchemaKey {
    fn from(get_request_schema: GetRequestSchema) -> Self {
        Self {
            content_hash: get_request_schema.name.expect("Schema name is required"),
            version: get_request_schema
                .version
                .expect("Schema version is required"),
        }
    }
}

impl Hash for PutRequestSchema {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.schema_content.hash(state);
    }
}

impl From<PutRequestSchema> for Schema {
    fn from(put_request_schema: PutRequestSchema) -> Self {
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
            format: put_request_schema.format,
            hash: Some(schema_hash.clone()),
            name: Some(schema_hash),
            namespace: Some(NAMESPACE.to_string()),
            schema_content: put_request_schema.schema_content,
            schema_type: put_request_schema.schema_type,
            tags: put_request_schema.tags,
            version: put_request_schema.version,
        }
    }
}
