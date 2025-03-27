#!/bin/sh
../../codegen/src/Azure.Iot.Operations.ProtocolCompiler/bin/Debug/net9.0/Azure.Iot.Operations.ProtocolCompiler \
 --serverOnly --modelFile ../../eng/dtdl/SchemaRegistry-1.json --sdkPath ../ --lang=rust --noProj \
 --outDir src/schema_registry_gen
 
cargo fmt
