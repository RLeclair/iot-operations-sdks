// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Stub Schema Registry service.

use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};

use azure_iot_operations_mqtt::interface::ManagedClient;
use azure_iot_operations_protocol::{application::ApplicationContext, rpc_command};

use crate::schema_registry::{SCHEMA_STATE_FILE, service_gen};
use crate::schema_registry::{
    Schema, SchemaKey,
    schema_registry_gen::{
        common_types::common_options::CommandOptionsBuilder,
        schema_registry::service::{GetResponsePayload, PutResponsePayload},
    },
};

/// Schema Registry service implementation.
pub struct Service<C>
where
    C: ManagedClient + Clone + Send + Sync + 'static,
    C::PubReceiver: Send + Sync + 'static,
{
    schemas: Arc<Mutex<HashMap<SchemaKey, Schema>>>,
    get_command_executor: service_gen::GetCommandExecutor<C>,
    put_command_executor: service_gen::PutCommandExecutor<C>,
    stub_service_output_dir: String,
}

impl<C> Service<C>
where
    C: ManagedClient + Clone + Send + Sync + 'static,
    C::PubReceiver: Send + Sync + 'static,
{
    /// Creates a new stub Schema Registry Service.
    pub fn new(
        application_context: ApplicationContext,
        client: C,
        stub_service_output_dir: &str,
    ) -> Self {
        log::info!("Schema Registry Stub Service created");

        Self {
            schemas: Arc::new(Mutex::new(HashMap::new())),
            get_command_executor: service_gen::GetCommandExecutor::new(
                application_context.clone(),
                client.clone(),
                &CommandOptionsBuilder::default()
                    .build()
                    .expect("Default command options should be valid"),
            ),
            put_command_executor: service_gen::PutCommandExecutor::new(
                application_context,
                client,
                &CommandOptionsBuilder::default()
                    .build()
                    .expect("Default command options should be valid"),
            ),
            stub_service_output_dir: stub_service_output_dir.to_string(),
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create the output file path
        let output_file_path = Path::new(&self.stub_service_output_dir).join(SCHEMA_STATE_FILE);

        // Create the file if it doesn't exist
        let output_file = if !output_file_path.exists() {
            log::info!("Creating output file: {:?}", output_file_path);
            std::fs::File::create(&output_file_path).expect("Failed to create output file")
        } else {
            log::warn!(
                "Output file already exists, creating again: {:?}",
                output_file_path
            );
            std::fs::File::create(&output_file_path).expect("Failed to open output file")
        };

        let get_schema_runner_handle = tokio::spawn(Self::get_schema_runner(
            self.get_command_executor,
            self.schemas.clone(),
        ));
        let put_schema_runner_handle = tokio::spawn(Self::put_schema_runner(
            self.put_command_executor,
            self.schemas,
            output_file,
        ));

        tokio::select! {
            r1 = get_schema_runner_handle => {
                if let Err(e) = r1 {
                    log::error!("Error in get_schema_runner: {:?}", e);
                    return Err(Box::<dyn std::error::Error + Send + Sync>::from(e));
                }
            },
            r2 = put_schema_runner_handle => {
                if let Err(e) = r2 {
                    log::error!("Error in put_schema_runner: {:?}", e);
                    return Err(Box::<dyn std::error::Error + Send + Sync>::from(e));
                }
            }
        };

        Ok(())
    }

    async fn get_schema_runner(
        mut get_command_executor: service_gen::GetCommandExecutor<C>,
        schemas: Arc<Mutex<HashMap<SchemaKey, Schema>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            match get_command_executor.recv().await {
                Some(incoming_request) => match incoming_request {
                    Ok(get_request) => {
                        log::debug!(
                            "Get request received: {:?}",
                            get_request.payload.get_schema_request
                        );

                        // Create a schema key
                        let schema_key =
                            SchemaKey::from(get_request.payload.clone().get_schema_request);

                        // Retrieve the schema
                        let schema = {
                            let schemas = schemas.lock().expect("Mutex management should be safe");
                            schemas.get(&schema_key).cloned()
                        };

                        if schema.is_some() {
                            log::debug!("Schema found: {:?}", schema);
                        } else {
                            log::debug!(
                                "Schema {:?} version {:?} not found",
                                schema_key.content_hash,
                                schema_key.version
                            );
                        }

                        // Send the response
                        let response = rpc_command::executor::ResponseBuilder::default()
                            .payload(GetResponsePayload {
                                schema: schema.map(|s| s.into()),
                            })
                            .expect("Get response payload should be valid")
                            .build()
                            .expect("Get response should not fail to build");

                        match get_request.complete(response).await {
                            Ok(_) => {
                                log::debug!(
                                    "Get request completed successfully for Schema {}, version {}",
                                    schema_key.content_hash,
                                    schema_key.version
                                );
                            }
                            Err(e) => {
                                log::error!("Failed to complete Get request: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error receiving Get request: {:?}", e);
                        return Err(Box::new(e));
                    }
                },
                None => {
                    log::info!("Get command executor closed");
                    return Ok(());
                }
            }
        }
    }

    async fn put_schema_runner(
        mut put_command_executor: service_gen::PutCommandExecutor<C>,
        schemas: Arc<Mutex<HashMap<SchemaKey, Schema>>>,
        mut output_file: File,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            match put_command_executor.recv().await {
                Some(incoming_request) => match incoming_request {
                    Ok(put_request) => {
                        log::debug!(
                            "Put request received: {:?}",
                            put_request.payload.put_schema_request
                        );

                        // Retrieve the schema
                        let schema: Schema = put_request.payload.put_schema_request.clone().into();

                        // TODO: Add verification of schema

                        // Create the schema key
                        let schema_key = SchemaKey {
                            content_hash: schema.hash.clone(),
                            version: schema.version.clone(),
                        };

                        // Store the schema in the HashMap
                        {
                            let mut schemas =
                                schemas.lock().expect("Mutex management should be safe");

                            match schemas.insert(schema_key.clone(), schema.clone()) {
                                Some(old_schema) => {
                                    log::debug!(
                                        "Schema {} updated, current version: {}",
                                        schema_key.content_hash,
                                        schema_key.version,
                                    );
                                    log::debug!("Old schema removed: {:?}", old_schema);
                                }
                                None => {
                                    log::debug!(
                                        "Schema {} added, version: {}",
                                        schema_key.content_hash,
                                        schema_key.version
                                    );
                                }
                            }

                            let schemas_list = schemas
                                .iter()
                                .map(|(_, schema)| schema.clone())
                                .collect::<Vec<_>>();

                            // Write the schemas to the output file
                            output_file.set_len(0).expect("Failed to clear file");
                            output_file
                                .write_all(
                                    serde_json::to_string_pretty(&schemas_list)
                                        .expect("Failed to serialize schemas")
                                        .as_bytes(),
                                )
                                .expect("Failed to write to file");
                        }

                        // Send the response
                        let response = rpc_command::executor::ResponseBuilder::default()
                            .payload(PutResponsePayload {
                                schema: schema.into(),
                            })
                            .expect("Put response payload should be valid")
                            .build()
                            .expect("Put response should not fail to build");

                        match put_request.complete(response).await {
                            Ok(_) => {
                                log::debug!(
                                    "Put request completed successfully for Schema {}, version {}",
                                    schema_key.content_hash,
                                    schema_key.version
                                );
                            }
                            Err(e) => {
                                log::error!("Failed to complete Put request: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error receiving Put request: {:?}", e);
                        return Err(Box::new(e));
                    }
                },
                None => {
                    log::info!("Put command executor closed");
                    return Ok(());
                }
            }
        }
    }
}
