// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Stub Schema Registry service.

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, Mutex},
};

use azure_iot_operations_mqtt::{interface::ManagedClient, session::Session};
use azure_iot_operations_protocol::{application::ApplicationContext, rpc_command};

use crate::{
    create_service_session,
    schema_registry::schema_registry_gen::schema_registry::service::{
        GetCommandExecutor, PutCommandExecutor,
    },
};

use super::{
    CLIENT_ID, Schema, SchemaKey,
    schema_registry_gen::{
        common_types::common_options::CommandOptionsBuilder,
        schema_registry::service::{GetResponsePayload, PutRequestSchema, PutResponsePayload},
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
        Self {
            get_command_executor: GetCommandExecutor::new(
                application_context.clone(),
                client.clone(),
                &CommandOptionsBuilder::default().build().unwrap(),
            ),
            put_command_executor: PutCommandExecutor::new(
                application_context,
                client,
                &CommandOptionsBuilder::default().build().unwrap(),
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

        let _ = tokio::try_join!(get_schema_runner_handle, put_schema_runner_handle,);

        Ok(())
    }

    async fn get_schema_runner(
        mut get_command_executor: GetCommandExecutor<C>,
        schemas: Arc<Mutex<HashMap<SchemaKey, Schema>>>,
    ) {
        loop {
            match get_command_executor.recv().await {
                Some(incoming_request) => match incoming_request {
                    Ok(get_request) => {
                        // Retrieve the schema
                        let schema = {
                            let schemas = schemas.lock().unwrap();
                            schemas
                                .get(&SchemaKey::from(
                                    get_request.payload.clone().get_schema_request,
                                ))
                                .cloned()
                        };

                        // Send the response
                        let response = rpc_command::executor::ResponseBuilder::default()
                            .payload(GetResponsePayload { schema })
                            .unwrap()
                            .build()
                            .unwrap();
                        get_request.complete(response).await.unwrap();
                    }
                    Err(_) => {
                        // Handle error
                        todo!();
                    }
                },
                None => todo!(),
            }
        }
    }

    async fn put_schema_runner(
        mut put_command_executor: PutCommandExecutor<C>,
        schemas: Arc<Mutex<HashMap<SchemaKey, Schema>>>,
    ) {
        loop {
            match put_command_executor.recv().await {
                Some(incoming_request) => match incoming_request {
                    Ok(put_request) => {
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
                            let mut schemas = schemas.lock().unwrap();
                            schemas.insert(schema_key.clone(), schema.clone());
                        }

                        // Send the response
                        let response = rpc_command::executor::ResponseBuilder::default()
                            .payload(PutResponsePayload { schema })
                            .unwrap()
                            .build()
                            .unwrap();
                        put_request.complete(response).await.unwrap();
                    }
                    Err(_) => {
                        todo!();
                    }
                },
                None => todo!(),
            }
        }
    }
}
