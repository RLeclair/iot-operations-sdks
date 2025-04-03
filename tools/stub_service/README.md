# Stub Service

The Stub Service provides a local implementation of services that typically require a full deployment of AIO. It is designed for development purposes and will be part of the VS Code extension created by the DevEx team.

The Stub Service will also be used for basic happy path testing on the SDKs.

## Features

- **Service Simulation**: Simulates basic service behavior based on DTDL contracts.
  - See [Schema Registry Stub Service Behavior](#schema-registry).
- **Session Isolation**: Each service operates in its own session with a unique MQTT client ID.
- **State and Logs**: Writes state and logs to a folder specified by the environment variable `STUB_SERVICE_OUTPUT_DIR`. 
  - The folder structure follows this format: `stub_service_[timestamp]`.
  - Each service creates its own subfolder within the main folder to store its state.
  - See [Example Output Folder](#output-folder-structure).
- **Unified Execution**: All stub services run from the same crate. A critical failure in any service causes a crash, with the error returned from `main`.
- **Logging**: Adheres to [ADR 0005](../../doc/dev/adr/0005-logging.md).

## Stub Services

### Schema Registry

*Note: Will replace the previous dotnet schema registry stub implementation which was built for testing.*

#### State Management

- Stores schemas in an internal hashmap.
- The key is derived from the schema's version number and name.

#### Supported Operations

1. **Get Schema**
   - Retrieves a schema by version.
   - Returns `None` if the schema does not exist.
   - *Errors*: Not implemented.

2. **Put Schema**
   - Stores a schema with a specific version.
   - Returns the stored schema along with its hash, name, and namespace.
   - *Errors*: Not implemented.

## Examples

### Output Folder Structure

```text
folder [STUB_SERVICE_OUTPUT_DIR]
  folder stub_service_1743702989
  ├── folder schema_registry
  │   ├── foo_schema.json
  │   ├── bar_schema.json
  └── folder diagnostics_service
      ├── foo_counter.json
      ├── bar_gauge.json
  folder stub_service_1743700000
  ├── folder schema_registry
  │   ├── foo_schema.json
  │   ├── bar_schema.json
  └── folder diagnostics_service
      ├── foo_counter.json
      ├── bar_gauge.json
```

## Open Questions

- Will the local environment need Kubernetes to run MQ? *(Answer: Yes.)*
- Should the stub service run as a Kubernetes pod or externally?
- What format should schema states use? *(Proposed: JSON.)*
- How will the `STUB_SERVICE_OUTPUT_DIR` variable be obtained?
- Will the ADR service need a stub? Or will it be included in the mini-MQ deployment in the same way as state store?
