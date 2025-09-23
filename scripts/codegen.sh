#!/bin/bash

set -e

echo "Generating protobuf code..."

# Create output directories
mkdir -p crates/nfa-broker/src/generated
mkdir -p go/protos

# Generate Rust code
PROTO_ROOT="protocols"

# Find all proto files
PROTO_FILES=$(find ${PROTO_ROOT} -name "*.proto")

for proto_file in ${PROTO_FILES}; do
    echo "Generating code for: ${proto_file}"
    
    # Generate Rust code
    protoc \
        --prost_out=crates/nfa-broker/src/generated \
        --proto_path=${PROTO_ROOT} \
        ${proto_file}
    
    # Generate Go code
    protoc \
        --go_out=go/protos \
        --go_opt=paths=source_relative \
        --go-grpc_out=go/protos \
        --go-grpc_opt=paths=source_relative \
        --proto_path=${PROTO_ROOT} \
        ${proto_file}
done

# Create mod.rs for generated Rust code
echo "// Generated code. Do not edit manually." > crates/nfa-broker/src/generated/mod.rs
for rust_file in $(find crates/nfa-broker/src/generated -name "*.rs"); do
    module_name=$(basename ${rust_file} .rs)
    echo "pub mod ${module_name};" >> crates/nfa-broker/src/generated/mod.rs
done

# Format generated code
echo "Formatting generated code..."
cargo fmt --all

echo "Code generation completed successfully!"