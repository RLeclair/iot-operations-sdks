/* Code generated by Azure.Iot.Operations.ProtocolCompiler v0.10.0.0; DO NOT EDIT. */

mod akri_metric;
mod akri_metric_operation;
mod akri_metric_type;
mod akri_publish_metrics_response;
mod publish_metrics_command_executor;
mod publish_metrics_request_payload;
mod publish_metrics_request_payload_serialization;
mod publish_metrics_response_payload;
mod publish_metrics_response_payload_serialization;

pub use azure_iot_operations_protocol::common::aio_protocol_error::AIOProtocolError;

pub use super::common_types::common_options::{CommandOptions, TelemetryOptions};

pub const MODEL_ID: &str = "dtmi:jsonTest:AkriObservabilityService;1";
pub const REQUEST_TOPIC_PATTERN: &str = "akri/observability/{ex:connectorClientId}/metrics";

pub mod service {
    pub use super::akri_metric::*;
    pub use super::akri_metric_operation::*;
    pub use super::akri_metric_type::*;
    pub use super::akri_publish_metrics_response::*;
    pub use super::publish_metrics_command_executor::*;
    pub use super::publish_metrics_request_payload::*;
    pub use super::publish_metrics_response_payload::*;
}
