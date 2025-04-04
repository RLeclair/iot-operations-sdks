use std::{
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use azure_iot_operations_mqtt::{
    MqttConnectionSettingsBuilder,
    session::{Session, SessionOptionsBuilder},
};

pub mod schema_registry;

const HOSTNAME: &str = "localhost";
const PORT: u16 = 1883;
const STUB_SERVICE_OUTPUT_DIR_NAME: &str = "stub_service_";
const STUB_SERVICE_ENVIRONMENT_VARIABLE: &str = "STUB_SERVICE_OUTPUT_DIR";

pub fn create_service_session(client_id: String) -> Result<Session, Box<dyn std::error::Error>> {
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

pub struct State {
    pub output_stub_service_path: String,
}

impl State {
    pub fn new() -> Self {
        // Read output directory from environment variable
        let output_dir = std::env::var(STUB_SERVICE_ENVIRONMENT_VARIABLE).expect(&format!(
            "{} must be set",
            STUB_SERVICE_ENVIRONMENT_VARIABLE
        ));

        // Create output directory for the stub service
        let output_stub_service_path = Path::new(&output_dir).join(format!(
            "{}_{}",
            STUB_SERVICE_OUTPUT_DIR_NAME,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        // Create the directory
        std::fs::create_dir_all(&output_stub_service_path)
            .expect("Failed to create output directory");

        Self {
            output_stub_service_path: output_stub_service_path
                .to_str()
                .expect("Failed to convert path to string")
                .to_string(),
        }
    }

    pub fn create_new_service_state_dir(&self, service_name: &str) -> String {
        let service_dir = self.output_stub_service_path.clone()
            + "/"
            + service_name
            + "_"
            + &SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string();

        std::fs::create_dir_all(&service_dir).expect("Failed to create service directory");

        service_dir
    }
}
