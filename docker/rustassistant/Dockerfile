# ============================================================================
# RustAssistant — API Server Dockerfile
# ============================================================================
#
# Multi-stage build for the RustAssistant API-only server.
# Produces a minimal runtime image without build tools or source code.
#
# Target: Oryx server (AMD64 primary, ARM64 optional)
#
# Build:
#   docker build -f docker/rustassistant/Dockerfile -t rustassistant:latest .
#   docker build -f docker/rustassistant/Dockerfile --build-arg SERVICE_PORT=3500 -t rustassistant:latest .
#
# Requires:
#   - Cargo.toml, Cargo.lock, src/, migrations/ in build context (project root)
#   - SQLX_OFFLINE=true (no live DB needed — project uses runtime sqlx::query())
# ============================================================================

# ----------------------------------------------------------------------------
# Stage 1: Builder — Compile RustAssistant
# ----------------------------------------------------------------------------
# Use debian:trixie-slim (glibc 2.41) as the builder base to satisfy ort-sys/ONNX Runtime
# prebuilt binaries which require glibc 2.38+ (__isoc23_strtoll etc.).
# debian:bookworm only ships glibc 2.36 which causes undefined symbol linker errors.
FROM debian:trixie-slim AS builder

# Install Rust toolchain + build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    pkg-config \
    libssl-dev \
    ca-certificates \
    git \
    g++ \
    make \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --no-modify-path --profile minimal --default-toolchain 1.92.0 && \
    chmod -R a+w /usr/local/rustup /usr/local/cargo

WORKDIR /build

# ── Dependency caching layer ─────────────────────────────────────────────────
# Copy manifests first so deps are only rebuilt when Cargo.toml/lock change.
COPY Cargo.toml Cargo.lock ./

# Create a minimal dummy source tree that satisfies Cargo's workspace/bin layout.
# This lets `cargo build` download and compile all dependencies without the real
# source, producing a cacheable layer.
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/server.rs && \
    echo "fn main() {}" > src/bin/cli.rs && \
    echo "fn main() {}" > src/bin/github-sync-daemon.rs && \
    echo "fn main() {}" > src/main.rs && \
    touch src/lib.rs

# Build dependencies only (this layer is cached until Cargo.toml/lock change)
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin rustassistant-server 2>&1 || true

# Remove dummy source and any fingerprints so Cargo rebuilds with real source
RUN rm -rf src target/release/rustassistant* target/release/.fingerprint/rustassistant* \
    target/release/deps/rustassistant* target/release/deps/librust*

# ── Real source build ────────────────────────────────────────────────────────
COPY src ./src
COPY migrations ./migrations

ENV SQLX_OFFLINE=true
RUN cargo build --release --bin rustassistant-server && \
    strip /build/target/release/rustassistant-server

# ----------------------------------------------------------------------------
# Stage 2: Runtime — Minimal production image
# ----------------------------------------------------------------------------
# Runtime must also use trixie (glibc 2.38+) since the binary was linked against
# trixie's glibc 2.41 and ort-sys embeds ONNX symbols requiring __isoc23_strtoll.
FROM debian:trixie-slim

ARG SERVICE_PORT=3500

# Install runtime dependencies only
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3t64 \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1000 -s /bin/bash rustassistant && \
    mkdir -p /app/data /app/config /app/workspace /app/docs && \
    chown -R rustassistant:rustassistant /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder --chown=rustassistant:rustassistant \
    /build/target/release/rustassistant-server \
    /usr/local/bin/rustassistant-server

# Copy config directory if present
COPY --chown=rustassistant:rustassistant config ./config

# Copy migrations for runtime auto-migration
COPY --from=builder --chown=rustassistant:rustassistant \
    /build/migrations ./migrations

# Switch to non-root user
USER rustassistant

# Environment variables with sane defaults.
# All can be overridden by docker-compose environment{} or .env.
# NOTE: The binary (src/bin/server.rs) reads HOST and PORT, not SERVER_HOST/SERVER_PORT.
ENV HOST=0.0.0.0 \
    PORT=${SERVICE_PORT} \
    RUST_LOG=info,rustassistant=debug \
    DATABASE_URL=postgres://rustassistant:changeme@localhost:5432/rustassistant \
    CACHE_ENABLED=true \
    SQLX_OFFLINE=true \
    MULTI_TENANT_MODE=false \
    TELEMETRY_ENABLED=false \
    ANALYTICS_ENABLED=true

EXPOSE ${SERVICE_PORT}

# Health check — uses /health (available in all image versions).
# /healthz alias exists in newer builds but /health is the safe default.
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -sf http://127.0.0.1:${PORT:-3500}/health || exit 1

CMD ["rustassistant-server"]
