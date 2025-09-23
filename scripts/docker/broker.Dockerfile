FROM rust:1.70-slim-bullseye as builder

WORKDIR /app

# Install protobuf compiler
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin nfa-broker

# Runtime image
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built binary
COPY --from=builder /app/target/release/nfa-broker /app/nfa-broker

# Create non-root user
RUN useradd -m nfa
USER nfa

# Expose broker port
EXPOSE 50051

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:50051/health || exit 1

# Start the broker
CMD ["/app/nfa-broker"]