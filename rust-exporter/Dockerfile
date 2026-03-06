# Multi-stage build for Rust exporter
FROM rust:latest AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY tests ./tests

# Build release binary
RUN cargo build --release

# Runtime stage
FROM ubuntu:24.04

# Install ca-certificates for HTTPS
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/rust-exporter /app/rust-exporter

# Expose ports
EXPOSE 8001 8002

# Run the binary
CMD ["/app/rust-exporter"]
