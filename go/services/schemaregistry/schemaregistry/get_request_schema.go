// Code generated by Azure.Iot.Operations.ProtocolCompiler v0.10.0.0; DO NOT EDIT.
package schemaregistry

type GetRequestSchema struct {

	// Schema name.
	Name *string `json:"name,omitempty"`

	// Version of the schema. Allowed between 0-9.
	Version *string `json:"version,omitempty"`
}
