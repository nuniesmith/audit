# ============================================================================
# Multi-stage Dockerfile for Rustassistant Server
# ============================================================================

# ----------------------------------------------------------------------------
# Stage 1: Builder
# ----------------------------------------------------------------------------
FROM rust:1.75-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY static ./static
COPY askama.toml ./askama.toml

# Build release binary
RUN cargo build --release --bin rustassistant-server

# ----------------------------------------------------------------------------
# Stage 2: Runtime
# ----------------------------------------------------------------------------
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/rustassistant-server /usr/local/bin/rustassistant-server

# Copy static assets and config
COPY static ./static
COPY config ./config

# Create data directory
RUN mkdir -p /app/data

# Set environment
ENV RUST_LOG=info,rustassistant=debug \
    HOST=0.0.0.0 \
    PORT=3000 \
    DATABASE_URL=sqlite:/app/data/rustassistant.db

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the server
CMD ["rustassistant-server"]
