// Code generated by Azure.Iot.Operations.ProtocolCompiler v0.10.0.0; DO NOT EDIT.
package schemaregistry

import (
	"encoding/json"
	"errors"
)

/// Supported schema types
type SchemaType int32

const (
	MessageSchema SchemaType = iota
)

func (v SchemaType) String() string {
	switch v {
	case MessageSchema:
		return "MessageSchema"
	default:
		return ""
	}
}

func (v SchemaType) MarshalJSON() ([]byte, error) {
	var s string
	switch v {
	case MessageSchema:
		s = "MessageSchema"
	default:
		return []byte{}, errors.New("unable to marshal unrecognized enum value to JSON")
	}

	return json.Marshal(s)
}

func (v *SchemaType) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return err
	}

	switch s {
	case "MessageSchema":
		*v = MessageSchema
	default:
		return errors.New("unable to unmarshal unrecognized enum value from JSON")
	}

	return nil
}
