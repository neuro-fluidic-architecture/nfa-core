.PHONY: all build test clean fmt clippy proto

all: build

build:
	@echo "Building NFA Core..."
	cargo build --workspace
	cd go && go build ./...

test:
	@echo "Running tests..."
	cargo test --workspace
	cd go && go test ./...

clean:
	@echo "Cleaning..."
	cargo clean
	cd go && go clean

fmt:
	@echo "Formatting code..."
	cargo fmt --all
	cd go && go fmt ./...

clippy:
	@echo "Running clippy..."
	cargo clippy --workspace -- -D warnings

proto:
	@echo "Generating protobuf code..."
	./scripts/codegen.sh

docker:
	@echo "Building Docker images..."
	docker build -t nfa-broker:latest -f docker/broker.Dockerfile .
	docker build -t nfa-runtime:latest -f docker/runtime.Dockerfile .

dev:
	@echo "Setting up development environment..."
	./scripts/setup-dev.sh

.PHONY: help
help:
	@echo "NFA Core Makefile commands:"
	@echo "  all       - Build everything (default)"
	@echo "  build     - Build Rust and Go code"
	@echo "  test      - Run tests"
	@echo "  clean     - Clean build artifacts"
	@echo "  fmt       - Format code"
	@echo "  clippy    - Run clippy linting"
	@echo "  proto     - Generate protobuf code"
	@echo "  docker    - Build Docker images"
	@echo "  dev       - Set up development environment"