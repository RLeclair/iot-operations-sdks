// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Stub Schema Registry service.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use azure_iot_operations_mqtt::interface::ManagedClient;
use azure_iot_operations_protocol::application::ApplicationContext;

use crate::schema_registry::schema_registry_gen::schema_registry::service::{
    GetCommandExecutor, PutCommandExecutor,
};

use super::{Schema, schema_registry_gen::common_types::common_options::CommandOptionsBuilder};

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

    pub async fn run(self) {
        // Create a HashMap to store schemas
        let schemas = Arc::new(Mutex::new(HashMap::new()));

        let get_schema_runner_handle = tokio::spawn(Self::get_schema_runner(
            self.get_command_executor,
            schemas.clone(),
        ));
        let put_schema_runner_handle = tokio::spawn(Self::put_schema_runner(
            self.put_command_executor,
            schemas.clone(),
        ));

        let _ = tokio::try_join!(get_schema_runner_handle, put_schema_runner_handle,);
    }

    async fn get_schema_runner(
        mut get_command_executor: GetCommandExecutor<C>,
        schemas: Arc<Mutex<HashMap<String, Schema>>>,
    ) {
        loop {
            match get_command_executor.recv().await {
                Some(incoming_request) => match incoming_request {
                    Ok(get_request) => {
                        let schema_id = get_request.payload.get_schema_request;
                    }
                    Err(err) => {
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
        schemas: Arc<Mutex<HashMap<String, Schema>>>,
    ) {
        loop {
            match put_command_executor.recv().await {
                Some(incoming_request) => match incoming_request {
                    Ok(put_request) => {
                        let schema = put_request.payload.put_schema_request;
                    }
                    Err(err) => {
                        todo!();
                    }
                },
                None => todo!(),
            }
        }
    }
}
