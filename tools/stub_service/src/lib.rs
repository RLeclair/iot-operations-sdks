use std::time::Duration;

use azure_iot_operations_mqtt::{
    MqttConnectionSettingsBuilder,
    session::{Session, SessionOptionsBuilder},
};

pub mod schema_registry;

const HOSTNAME: &str = "localhost";
const PORT: u16 = 1883;

fn create_service_session(client_id: String) -> Result<Session, Box<dyn std::error::Error>> {
    // Create a Session
    let connection_settings = MqttConnectionSettingsBuilder::default()
        .client_id(client_id)
        .hostname(HOSTNAME)
        .tcp_port(PORT)
        .keep_alive(Duration::from_secs(5))
        .use_tls(false)
        .build()?;

    let session_options = SessionOptionsBuilder::default()
        .connection_settings(connection_settings)
        .build()?;

    Ok(Session::new(session_options)?)
}
