// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Stub Schema Registry service.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use azure_iot_operations_mqtt::interface::ManagedClient;
use azure_iot_operations_protocol::{application::ApplicationContext, rpc_command};

use crate::schema_registry::schema_registry_gen::schema_registry::service::{
    GetCommandExecutor, PutCommandExecutor,
};
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
    get_command_executor: GetCommandExecutor<C>,
    put_command_executor: PutCommandExecutor<C>,
}

impl<C> Service<C>
where
    C: ManagedClient + Clone + Send + Sync + 'static,
    C::PubReceiver: Send + Sync + 'static,
{
    /// Creates a new stub Schema Registry Service.
    pub fn new(application_context: ApplicationContext, client: C) -> Self {
        log::info!("Schema Registry Stub Service created");
        Self {
            get_command_executor: GetCommandExecutor::new(
                application_context.clone(),
                client.clone(),
                &CommandOptionsBuilder::default()
                    .build()
                    .expect("Default command options should be valid"),
            ),
            put_command_executor: PutCommandExecutor::new(
                application_context,
                client,
                &CommandOptionsBuilder::default()
                    .build()
                    .expect("Default command options should be valid"),
            ),
        }
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create a HashMap to store schemas
        let schemas = Arc::new(Mutex::new(HashMap::new()));

        let get_schema_runner_handle = tokio::spawn(Self::get_schema_runner(
            self.get_command_executor,
            schemas.clone(),
        ));
        let put_schema_runner_handle =
            tokio::spawn(Self::put_schema_runner(self.put_command_executor, schemas));

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
            },
        };

        Ok(())
    }

    async fn get_schema_runner(
        mut get_command_executor: GetCommandExecutor<C>,
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
                            .payload(GetResponsePayload { schema })
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
        mut put_command_executor: PutCommandExecutor<C>,
        schemas: Arc<Mutex<HashMap<SchemaKey, Schema>>>,
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
                            content_hash: schema
                                .hash
                                .clone()
                                .expect("Schema hash should be present in the schema"),
                            version: schema
                                .version
                                .clone()
                                .expect("Schema version should be present in the schema"),
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
                        }

                        // Send the response
                        let response = rpc_command::executor::ResponseBuilder::default()
                            .payload(PutResponsePayload { schema })
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
