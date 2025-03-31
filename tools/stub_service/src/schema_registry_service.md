# Schema Registry Stub Service

# A Schema

# Schema Store

Stores schemas in a HashMap whose key is hashed between the name and the version. 

When doing a put this is the response from the service:

``` bash
Publish { dup: false, qos: AtLeastOnce, retain: false, topic: b"clients/sampleSchemaRegistry/adr/dtmi:ms:adr:SchemaRegistry;1/put", pkid: 1, payload: b"{\"schema\":{\"name\":\"6548f0acbbf9cad3a000f3d986a7fb957c25fa3e9040d63a9ae6d8bc84d6b69f\",\"format\":\"JsonSchema/draft-07\",\"schemaType\":\"MessageSchema\",\"version\":\"1\",\"schemaContent\":\"\\n{\\n  \\\"$schema\\\": \\\"http://json-schema.org/draft-07/schema#\\\",\\n  \\\"type\\\": \\\"object\\\",\\n  \\\"properties\\\": {\\n    \\\"humidity\\\": {\\n      \\\"type\\\": \\\"integer\\\"\\n    },\\n    \\\"temperature\\\": {\\n      \\\"type\\\": \\\"number\\\"\\n    }\\n  }\\n}\\n\",\"hash\":\"6548f0acbbf9cad3a000f3d986a7fb957c25fa3e9040d63a9ae6d8bc84d6b69f\",\"namespace\":\"aio-sr-ns-15c3826465\"}}", properties: Some(PublishProperties { payload_format_indicator: Some(1), message_expiry_interval: Some(10), topic_alias: None, response_topic: None, correlation_data: Some(b"\xec1\xfc\xf8\xe5\x13G\xb1\xb0\xf7^tS\xe5E\x9a"), user_properties: [("__ts", "001743203575423:00000:0195de76-3429-79cb-b043-f5b9f844e7d8"), ("__protVer", "1.0"), ("__stat", "200")], subscription_identifiers: [], content_type: Some("application/json") }) }
```

This is what we send when we get when we do a get:

``` bash
[INFO  schema_registry_client] Got schema: Some(Schema { description: None, display_name: None, format: Some(JsonSchemaDraft07), hash: Some("6548f0acbbf9cad3a000f3d986a7fb957c25fa3e9040d63a9ae6d8bc84d6b69f"), name: Some("6548f0acbbf9cad3a000f3d986a7fb957c25fa3e9040d63a9ae6d8bc84d6b69f"), namespace: Some("aio-sr-ns-15c3826465"), schema_content: Some("\n{\n  \"$schema\": \"http://json-schema.org/draft-07/schema#\",\n  \"type\": \"object\",\n  \"properties\": {\n    \"humidity\": {\n      \"type\": \"integer\"\n    },\n    \"temperature\": {\n      \"type\": \"number\"\n    }\n  }\n}\n"), schema_type: Some(MessageSchema), tags: None, version: Some("1") })
```

If I do a put with a different version I get:
``` bash
[DEBUG azure_iot_operations_mqtt::session::session] Incoming PUB: Publish { dup: false, qos: AtLeastOnce, retain: false, topic: b"clients/sampleSchemaRegistry/adr/dtmi:ms:adr:SchemaRegistry;1/put", pkid: 25, payload: b"{\"schema\":{\"name\":\"1df00ae814b3fb8f8cfd308d1b9eba006d439662ebebac0f8309ab00a1bd72b3\",\"format\":\"JsonSchema/draft-07\",\"schemaType\":\"MessageSchema\",\"version\":\"2\",\"schemaContent\":\"\\n{\\n  \\\"$schema\\\": \\\"http://json-schema.org/draft-07/schema#\\\",\\n  \\\"type\\\": \\\"object\\\",\\n  \\\"properties\\\": {\\n    \\\"humidity\\\": {\\n      \\\"type\\\": \\\"number\\\"\\n    },\\n    \\\"temperature\\\": {\\n      \\\"type\\\": \\\"number\\\"\\n    }\\n  }\\n}\\n\",\"hash\":\"1df00ae814b3fb8f8cfd308d1b9eba006d439662ebebac0f8309ab00a1bd72b3\",\"namespace\":\"aio-sr-ns-15c3826465\"}}", properties: Some(PublishProperties { payload_format_indicator: Some(1), message_expiry_interval: Some(10), topic_alias: None, response_topic: None, correlation_data: Some(b"ABBo\x83\xb3J\xad\xadR\xc0\xca\xa2\xd8{\x17"), user_properties: [("__protVer", "1.0"), ("__stat", "200"), ("__ts", "001743204265106:00000:0195de76-3429-79cb-b043-f5b9f844e7d8")], subscription_identifiers: [], content_type: Some("application/json") }) }
```

Schema Registry Stub Service:
- Create a HashMap that can store the contents of the schema registry and the key

Schema: Hello
Hash: 1234
Version: 1

Schema: Hello 
Hash: 1234
Version: 2

The name is the hash

HashMap<String, String>

The key has to be a hash of the hash of hash of contents + version