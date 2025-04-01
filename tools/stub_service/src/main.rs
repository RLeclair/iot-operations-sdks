use azure_iot_operations_protocol::application::{ApplicationContext, ApplicationContextBuilder};
use env_logger::Builder;
use stub_service::{
    create_service_session,
    schema_registry::{self},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .filter_module("rumqttc", log::LevelFilter::Warn)
        .init();

    let application_context = ApplicationContextBuilder::default().build().unwrap();

    let sr_service_session =
        create_service_session(schema_registry::CLIENT_ID.to_string()).unwrap();
    let sr_service_stub = schema_registry::Service::new(
        application_context,
        sr_service_session.create_managed_client(),
    );

    // Run the services
    tokio::select! {
        r1 = sr_service_session.run() => r1?,
        r2 = sr_service_stub.run() => r2.map_err(|e| e as Box<dyn std::error::Error>)?,
    }

    Ok(())
}
