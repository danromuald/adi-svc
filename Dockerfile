# Multi-stage build for adi-svc

# Stage 1: Build
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY adi-svc/Cargo.toml adi-svc/Cargo.lock ./
COPY adi-svc/build.rs ./

# Copy proto files
COPY adi-svc/proto ./proto

# Copy source code
COPY adi-svc/src ./src

# Build for release
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 app

# Copy binary from builder
COPY --from=builder /app/target/release/adi-svc /usr/local/bin/adi-svc

# Create uploads directory
RUN mkdir -p /app/uploads && chown app:app /app/uploads

USER app
WORKDIR /app

# Expose ports
EXPOSE 50051 8080

# Run the service
CMD ["adi-svc"]

