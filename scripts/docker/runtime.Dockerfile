FROM golang:1.21-alpine as builder

WORKDIR /app

# Install protobuf compiler and dependencies
RUN apk add --no-cache \
    protobuf-dev \
    make \
    git

# Copy go mod files
COPY go/go.mod go/go.sum ./

# Download dependencies
RUN go mod download

# Copy source code
COPY . .

# Build the runtime
RUN go build -o nfa-runtime ./go/runtime

# Runtime image
FROM alpine:latest

RUN apk add --no-cache \
    ca-certificates

WORKDIR /app

# Copy built binary
COPY --from=builder /app/nfa-runtime /app/nfa-runtime

# Create non-root user
RUN adduser -D nfa
USER nfa

# Start the runtime
CMD ["/app/nfa-runtime"]