#!/bin/bash

set -e

echo "Setting up NFA Core development environment..."

# Check for required tools
command -v rustup >/dev/null 2>&1 || { echo "Rust is required. Please install rustup."; exit 1; }
command -v go >/dev/null 2>&1 || { echo "Go is required. Please install Go."; exit 1; }
command -v protoc >/dev/null 2>&1 || { echo "Protobuf compiler is required. Please install protobuf-compiler."; exit 1; }

# Install Rust components
echo "Installing Rust components..."
rustup component add rustfmt clippy

# Install Go dependencies
echo "Installing Go dependencies..."
cd go
go mod download
cd ..

# Install protobuf tools for Go
echo "Installing protobuf Go tools..."
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

# Generate protobuf code
echo "Generating protobuf code..."
./scripts/codegen.sh

# Build the project
echo "Building project..."
make build

echo "Development environment setup complete!"
echo ""
echo "Next steps:"
echo "1. Run 'make test' to verify everything works"
echo "2. Check out the examples in examples/"
echo "3. Read the docs in docs/ for more information"