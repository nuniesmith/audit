#!/usr/bin/env bash
# ============================================================================
# RustAssistant - Project Management Script
# ============================================================================
# Wrapper for managing the full RustAssistant stack:
#   ra-app (Rust API)  ·  ra-postgres  ·  ra-redis  ·  ra-ollama (optional)
#   openclaw-gateway   ·  openclaw-cli
#
# All services bind to the Tailscale IP defined in .env (TAILSCALE_IP).
#
# Usage:
#   ./run.sh up                    # Build OpenClaw + start core services
#   ./run.sh up --ollama           # Include Ollama local LLM
#   ./run.sh up --skip-build       # Start services without rebuilding
#   ./run.sh up ra-app             # Start only specific service(s)
#   ./run.sh down                  # Stop all services
#   ./run.sh status                # Show service status
#   ./run.sh logs [service]        # Tail service logs
#   ./run.sh health                # Check service health
#   ./run.sh openclaw <cmd>        # Run OpenClaw CLI commands
#   ./run.sh build [openclaw|app]  # Build specific images
#   ./run.sh test                  # Run Rust tests
#   ./run.sh ci                    # Run full CI pipeline
#   ./run.sh db backup             # Backup Postgres database
#   ./run.sh diagnose              # System diagnostics
#   ./run.sh help                  # Show full usage
# ============================================================================

set -euo pipefail

# ── Colours ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# ── Paths ────────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

COMPOSE_FILE="docker-compose.yml"
ENV_FILE=".env"
OPENCLAW_BUILD_SCRIPT="docker/openclaw/build.sh"

# ── Docker Compose shorthand ────────────────────────────────────────────────
DC_BASE="docker compose --env-file $ENV_FILE -f $COMPOSE_FILE"
DC="$DC_BASE"

# ── Load .env for variable interpolation (TAILSCALE_IP, ports, etc.) ────────
if [ -f "$ENV_FILE" ]; then
    set -a
    # shellcheck disable=SC1090
    source "$ENV_FILE" 2>/dev/null || true
    set +a
fi

TAILSCALE_IP="${TAILSCALE_IP:-}"
RA_PORT="3500"
OPENCLAW_PORT="18789"

# ============================================================================
# Helper Functions
# ============================================================================

print_header() {
    echo ""
    echo -e "${BLUE}===================================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}===================================================${NC}"
}

print_success() { echo -e "${GREEN}✓ $1${NC}"; }
print_warning() { echo -e "${YELLOW}⚠ $1${NC}"; }
print_error()   { echo -e "${RED}✗ $1${NC}"; }
print_info()    { echo -e "${BLUE}ℹ $1${NC}"; }
print_step()    { echo -e "${CYAN}→ $1${NC}"; }

# ============================================================================
# Environment Setup
# ============================================================================

validate_env() {
    if [ ! -f "$ENV_FILE" ]; then
        print_error ".env file not found."
        print_info "Copy .env.example to .env and fill in the required values:"
        print_info "  cp .env.example .env && \$EDITOR .env"
        return 1
    fi

    local warnings=0

    if [ -z "$TAILSCALE_IP" ]; then
        print_error "TAILSCALE_IP is not set in .env — all port bindings will fail."
        return 1
    fi

    local xai_key
    xai_key=$(grep "^XAI_API_KEY=" "$ENV_FILE" 2>/dev/null | cut -d'=' -f2- | tr -d ' ')
    if [ -z "$xai_key" ] || [ "$xai_key" = "xai-your-api-key-here" ]; then
        print_warning "XAI_API_KEY is not set — remote LLM routing will not work"
        ((warnings++)) || true
    fi

    local gw_token
    gw_token=$(grep "^OPENCLAW_GATEWAY_TOKEN=" "$ENV_FILE" 2>/dev/null | cut -d'=' -f2- | tr -d ' ')
    if [ -z "$gw_token" ]; then
        print_warning "OPENCLAW_GATEWAY_TOKEN is not set — OpenClaw auth will be disabled"
        ((warnings++)) || true
    fi

    local proxy_keys
    proxy_keys=$(grep "^RA_PROXY_API_KEYS=" "$ENV_FILE" 2>/dev/null | cut -d'=' -f2- | tr -d ' ')
    if [ -z "$proxy_keys" ]; then
        print_warning "RA_PROXY_API_KEYS is not set — proxy endpoint will be open (no auth)"
        ((warnings++)) || true
    fi

    if [ "$warnings" -eq 0 ]; then
        print_success ".env validated — all required keys present"
    else
        print_warning ".env has $warnings warning(s) — non-critical, services will still start"
    fi
    return 0
}

# ============================================================================
# Pre-flight Checks
# ============================================================================

preflight_check() {
    print_header "Pre-flight Checks"

    local errors=0

    # Docker daemon
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker daemon is not running"
        ((errors++))
    else
        print_success "Docker daemon is running"
    fi

    # Docker Compose
    if ! docker compose version > /dev/null 2>&1; then
        print_error "Docker Compose V2 is not available"
        ((errors++))
    else
        print_success "Docker Compose is available"
    fi

    # Disk space
    local available_gb
    available_gb=$(df "$SCRIPT_DIR" | tail -1 | awk '{print int($4/1024/1024)}')
    if [ "$available_gb" -lt 2 ]; then
        print_error "Low disk space: ${available_gb}GB available (need at least 2GB)"
        ((errors++))
    else
        print_success "Disk space OK: ${available_gb}GB available"
    fi

    # Tailscale
    if [ -n "$TAILSCALE_IP" ]; then
        if ip addr show | grep -q "$TAILSCALE_IP"; then
            print_success "Tailscale IP $TAILSCALE_IP is bound to an interface"
        else
            print_warning "Tailscale IP $TAILSCALE_IP not found on any interface"
            print_info "  Services may fail to bind. Check 'tailscale status'."
        fi
    fi

    # .env
    validate_env || ((errors++))

    if [ $errors -gt 0 ]; then
        echo ""
        print_error "Pre-flight failed with $errors error(s)"
        return 1
    else
        echo ""
        print_success "All checks passed"
        return 0
    fi
}

# ============================================================================
# Build Commands
# ============================================================================

build_openclaw() {
    local extra_args=("$@")

    if [ ! -f "$OPENCLAW_BUILD_SCRIPT" ]; then
        print_error "OpenClaw build script not found: $OPENCLAW_BUILD_SCRIPT"
        print_info "The build script clones upstream OpenClaw and builds the base + RA layer."
        return 1
    fi

    print_step "Building OpenClaw image (base + RustAssistant layer) ..."
    print_info "This may take several minutes on first build."
    echo ""

    bash "$OPENCLAW_BUILD_SCRIPT" "${extra_args[@]}"
}

build_app() {
    print_step "Building ra-app image ..."
    $DC build ra-app "$@"
    print_success "ra-app image built"
}

cmd_build() {
    local target="${1:-all}"
    shift 2>/dev/null || true

    case "$target" in
        openclaw|oc)
            print_header "Building OpenClaw"
            build_openclaw "$@"
            ;;
        app|ra-app)
            print_header "Building RustAssistant App"
            build_app "$@"
            ;;
        all)
            print_header "Building All Images"
            build_openclaw "$@"
            echo ""
            build_app "$@"
            print_success "All images built"
            ;;
        *)
            print_header "Building $target"
            $DC build "$target" "$@"
            print_success "$target built"
            ;;
    esac
}

# ============================================================================
# Docker Commands
# ============================================================================

cmd_up() {
    local skip_build=false
    local with_ollama=false
    local services=()

    # Parse flags
    while [ $# -gt 0 ]; do
        case "$1" in
            --skip-build|--no-build|-n)
                skip_build=true
                shift
                ;;
            --rebuild|--force-build)
                # Explicit build even if user might have set skip elsewhere
                skip_build=false
                shift
                ;;
            --ollama)
                with_ollama=true
                shift
                ;;
            --no-ollama)
                with_ollama=false
                shift
                ;;
            -*)
                print_error "Unknown flag: $1"
                return 1
                ;;
            *)
                services+=("$1")
                shift
                ;;
        esac
    done

    # Activate the ollama compose profile when requested
    if [ "$with_ollama" = true ]; then
        DC="$DC_BASE --profile ollama"
    fi

    print_header "Starting RustAssistant Stack"
    if [ "$with_ollama" = true ]; then
        print_info "Ollama local LLM: enabled"
    else
        print_info "Ollama local LLM: disabled (pass --ollama to enable)"
    fi

    # Pre-flight
    echo ""
    if ! preflight_check; then
        echo ""
        print_info "Fix the errors above and try again."
        exit 1
    fi

    # ── Build OpenClaw image ─────────────────────────────────────────────────
    if [ "$skip_build" = false ]; then
        echo ""
        # Only build openclaw if no specific services requested, or openclaw is in the list
        if [ ${#services[@]} -eq 0 ] || printf '%s\n' "${services[@]}" | grep -qE '^(openclaw-gateway|openclaw-cli)$'; then
            # Check if openclaw image exists; if not, must build
            if ! docker image inspect "openclaw:local" > /dev/null 2>&1; then
                print_step "openclaw:local image not found — building from source ..."
                build_openclaw
            else
                print_info "openclaw:local image exists. Rebuilding RA config layer ..."
                build_openclaw --layer-only
            fi
        fi

        # Build ra-app if requested or if starting everything
        if [ ${#services[@]} -eq 0 ] || printf '%s\n' "${services[@]}" | grep -q '^ra-app$'; then
            echo ""
            print_step "Ensuring ra-app image is up to date ..."
            $DC build ra-app 2>/dev/null || true
        fi
    else
        print_info "Skipping image builds (--skip-build)"
    fi

    # ── Start services ───────────────────────────────────────────────────────
    echo ""
    if [ ${#services[@]} -gt 0 ]; then
        print_step "Starting: ${services[*]} ..."
        $DC up -d "${services[@]}"
    else
        print_step "Starting all services ..."
        $DC up -d
    fi

    print_success "Services started"

    # ── Wait and show status ─────────────────────────────────────────────────
    print_info "Waiting for health checks ..."
    sleep 8

    cmd_status

    # ── Print access info ────────────────────────────────────────────────────
    echo ""
    print_success "RustAssistant stack is up!"
    echo ""
    print_info "Access points (Tailscale: ${TAILSCALE_IP:-<not set>}):"
    echo "  RA API:              http://${TAILSCALE_IP}:${RA_PORT}/health"
    echo "  RA Proxy (OpenAI):   http://${TAILSCALE_IP}:${RA_PORT}/v1/models"
    echo "  Postgres:            ${TAILSCALE_IP}:5433"
    echo "  Redis:               ${TAILSCALE_IP}:6380"
    if [ "$with_ollama" = true ]; then
        echo "  Ollama:              http://${TAILSCALE_IP}:11434"
    fi
    echo "  OpenClaw Gateway:    ws://${TAILSCALE_IP}:${OPENCLAW_PORT}"
    echo "  OpenClaw Control UI: https://$(tailscale status --self --json 2>/dev/null | grep -o '"DNSName":"[^"]*"' | cut -d'"' -f4 | sed 's/\.$//' || echo '<tailscale-hostname>')"
    echo ""
    print_info "Useful commands:"
    echo "  ./run.sh logs                    # Tail all logs"
    echo "  ./run.sh logs openclaw-gateway   # Tail OpenClaw gateway"
    echo "  ./run.sh health                  # Check all services"
    echo "  ./run.sh openclaw doctor         # OpenClaw diagnostics"
    echo "  ./run.sh down                    # Stop everything"
}

cmd_down() {
    print_header "Stopping RustAssistant Stack"

    $DC down --remove-orphans "$@"

    print_success "All services stopped"
}

cmd_restart() {
    local services=("$@")

    if [ ${#services[@]} -eq 0 ]; then
        print_header "Restarting All Services"
        $DC restart
    else
        print_header "Restarting: ${services[*]}"
        $DC restart "${services[@]}"
    fi

    print_success "Restart complete"
}

cmd_logs() {
    $DC logs -f --tail=100 "$@"
}

cmd_status() {
    print_header "Service Status"

    $DC ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || $DC ps
}

cmd_shell() {
    local service="${1:-ra-app}"

    print_info "Opening shell in $service ..."
    $DC exec "$service" /bin/bash 2>/dev/null || \
    $DC exec "$service" /bin/sh 2>/dev/null || \
    $DC exec "$service" sh
}

# ============================================================================
# OpenClaw CLI Passthrough
# ============================================================================

cmd_openclaw() {
    if [ $# -eq 0 ]; then
        print_info "OpenClaw CLI — pass any openclaw subcommand after 'openclaw'."
        echo ""
        echo "  ./run.sh openclaw --help"
        echo "  ./run.sh openclaw doctor"
        echo "  ./run.sh openclaw models status --plain"
        echo "  ./run.sh openclaw devices list"
        echo "  ./run.sh openclaw channels status"
        echo "  ./run.sh openclaw health"
        echo "  ./run.sh openclaw dashboard --no-open"
        echo "  ./run.sh openclaw agent --message 'Hello' --to '#general'"
        return 0
    fi

    $DC run --rm openclaw-cli "$@"
}

# ============================================================================
# Health Checks
# ============================================================================

cmd_health() {
    print_header "Service Health"

    local all_healthy=true

    # ── ra-app ────────────────────────────────────────────────────────────────
    echo -e "\n${BOLD}ra-app (RustAssistant API)${NC}"
    local health_response
    if health_response=$(curl -sf "http://${TAILSCALE_IP}:${RA_PORT}/health" 2>/dev/null); then
        print_success "Healthy (${TAILSCALE_IP}:${RA_PORT})"
        if [ -n "$health_response" ]; then
            echo "  $health_response" | head -3
        fi
    elif health_response=$(curl -sf "http://127.0.0.1:${RA_PORT}/health" 2>/dev/null); then
        print_success "Healthy (localhost:${RA_PORT})"
    else
        print_error "Not responding on port ${RA_PORT}"
        all_healthy=false
    fi

    # ── ra-postgres ───────────────────────────────────────────────────────────
    echo -e "\n${BOLD}ra-postgres (PostgreSQL 16)${NC}"
    if $DC exec -T ra-postgres pg_isready -U rustassistant > /dev/null 2>&1; then
        print_success "Healthy"
        local pg_size
        pg_size=$($DC exec -T ra-postgres psql -U rustassistant -d rustassistant -tAc \
            "SELECT pg_size_pretty(pg_database_size('rustassistant'));" 2>/dev/null | tr -d ' ')
        if [ -n "$pg_size" ]; then
            echo "  Database size: $pg_size"
        fi
    else
        print_error "Not responding"
        all_healthy=false
    fi

    # ── ra-redis ──────────────────────────────────────────────────────────────
    echo -e "\n${BOLD}ra-redis (Redis 7)${NC}"
    local redis_pass
    redis_pass=$(grep "^RA_REDIS_PASSWORD=" "$ENV_FILE" 2>/dev/null | cut -d'=' -f2-)
    if $DC exec -T ra-redis redis-cli -a "$redis_pass" ping 2>/dev/null | grep -q PONG; then
        print_success "Healthy"
        local redis_mem
        redis_mem=$($DC exec -T ra-redis redis-cli -a "$redis_pass" info memory 2>/dev/null \
            | grep "used_memory_human" | cut -d: -f2 | tr -d '\r')
        if [ -n "$redis_mem" ]; then
            echo "  Memory used: $redis_mem"
        fi
    else
        print_error "Not responding"
        all_healthy=false
    fi

    # ── ra-ollama (optional) ──────────────────────────────────────────────────
    if docker ps --format '{{.Names}}' | grep -q '^ra-ollama$'; then
        echo -e "\n${BOLD}ra-ollama (Ollama)${NC}"
        if curl -sf "http://${TAILSCALE_IP}:11434/api/tags" > /dev/null 2>&1; then
            print_success "Healthy"
            local models
            models=$(curl -sf "http://${TAILSCALE_IP}:11434/api/tags" 2>/dev/null \
                | grep -o '"name":"[^"]*"' | cut -d'"' -f4 | tr '\n' ', ' | sed 's/,$//')
            if [ -n "$models" ]; then
                echo "  Models: $models"
            else
                echo "  Models: (none pulled yet)"
            fi
        else
            print_error "Not responding on port 11434"
            all_healthy=false
        fi
    else
        echo -e "\n${BOLD}ra-ollama (Ollama)${NC}"
        print_info "Not running (start with: ./run.sh up --ollama)"
    fi

    # ── openclaw-gateway ──────────────────────────────────────────────────────
    echo -e "\n${BOLD}openclaw-gateway${NC}"
    if curl -sf "http://127.0.0.1:${OPENCLAW_PORT}/healthz" > /dev/null 2>&1 \
       || curl -sf "http://${TAILSCALE_IP}:${OPENCLAW_PORT}/healthz" > /dev/null 2>&1; then
        print_success "Healthy"
        local gw_model
        gw_model=$(docker logs openclaw-gateway 2>&1 | grep "agent model:" | tail -1 \
            | sed 's/.*agent model: //')
        if [ -n "$gw_model" ]; then
            echo "  Agent model: $gw_model"
        fi
    else
        print_error "Not responding on port ${OPENCLAW_PORT}"
        all_healthy=false
    fi

    # ── Container overview ────────────────────────────────────────────────────
    echo -e "\n${BOLD}Container Status${NC}"
    $DC ps --format "table {{.Name}}\t{{.Status}}" 2>/dev/null || $DC ps

    echo ""
    if [ "$all_healthy" = true ]; then
        print_success "All services healthy!"
    else
        print_error "Some services are unhealthy — check logs with: ./run.sh logs"
    fi
}

# ============================================================================
# Database Commands (Postgres)
# ============================================================================

cmd_db() {
    local subcmd="${1:-help}"
    shift 2>/dev/null || true

    case "$subcmd" in
        backup)     cmd_db_backup "$@" ;;
        restore)    cmd_db_restore "$@" ;;
        shell|psql) cmd_db_shell ;;
        size)       cmd_db_size ;;
        migrate)    cmd_db_migrate ;;
        *)
            echo "Database commands (Postgres):"
            echo "  ./run.sh db backup [path]    Dump database to ./backups/"
            echo "  ./run.sh db restore <file>   Restore from a .sql.gz backup"
            echo "  ./run.sh db shell            Open psql CLI"
            echo "  ./run.sh db size             Show database size"
            echo "  ./run.sh db migrate          Run pending migrations (via ra-app)"
            ;;
    esac
}

cmd_db_backup() {
    print_header "Database Backup (Postgres)"

    local backup_dir="${1:-./backups}"
    local timestamp
    timestamp=$(date +%Y%m%d-%H%M%S)
    local backup_file="${backup_dir}/rustassistant-${timestamp}.sql.gz"

    mkdir -p "$backup_dir"

    print_step "Dumping rustassistant database ..."

    if $DC exec -T ra-postgres pg_dump -U rustassistant rustassistant \
       | gzip > "$backup_file" 2>/dev/null; then
        local size
        size=$(du -h "$backup_file" | cut -f1)
        print_success "Backup saved: $backup_file ($size)"
    else
        rm -f "$backup_file"
        print_error "Backup failed — is ra-postgres running?"
        return 1
    fi

    echo ""
    print_info "Backups in $backup_dir/:"
    ls -lh "$backup_dir"/rustassistant-*.sql.gz 2>/dev/null | tail -5
}

cmd_db_restore() {
    local backup_file="$1"

    if [ -z "$backup_file" ]; then
        print_error "Usage: ./run.sh db restore <backup-file.sql.gz>"
        echo ""
        print_info "Available backups:"
        ls -lh backups/rustassistant-*.sql.gz 2>/dev/null || echo "  No backups found in ./backups/"
        return 1
    fi

    if [ ! -f "$backup_file" ]; then
        print_error "File not found: $backup_file"
        return 1
    fi

    print_header "Database Restore"
    print_warning "This will DROP and recreate the rustassistant database!"
    echo -n "Type 'yes' to confirm: "
    read -r response
    if [ "$response" != "yes" ]; then
        print_info "Restore cancelled"
        return 0
    fi

    print_step "Stopping ra-app to release connections ..."
    $DC stop ra-app 2>/dev/null || true

    print_step "Restoring from $backup_file ..."
    $DC exec -T ra-postgres psql -U rustassistant -d postgres \
        -c "DROP DATABASE IF EXISTS rustassistant;" \
        -c "CREATE DATABASE rustassistant OWNER rustassistant;" 2>/dev/null

    if gunzip -c "$backup_file" | $DC exec -T ra-postgres psql -U rustassistant -d rustassistant > /dev/null 2>&1; then
        print_success "Database restored from: $backup_file"
    else
        print_error "Restore failed"
        return 1
    fi

    print_step "Starting ra-app ..."
    $DC up -d ra-app
    print_success "Done — ra-app will run migrations on startup if needed"
}

cmd_db_shell() {
    print_info "Opening psql shell (\\q to quit) ..."
    $DC exec ra-postgres psql -U rustassistant -d rustassistant
}

cmd_db_size() {
    print_header "Database Size"

    $DC exec -T ra-postgres psql -U rustassistant -d rustassistant -c "
        SELECT
            pg_size_pretty(pg_database_size('rustassistant')) AS database_size;
    " 2>/dev/null

    echo ""
    $DC exec -T ra-postgres psql -U rustassistant -d rustassistant -c "
        SELECT
            schemaname AS schema,
            relname AS table,
            pg_size_pretty(pg_total_relation_size(relid)) AS total_size,
            n_live_tup AS row_count
        FROM pg_stat_user_tables
        ORDER BY pg_total_relation_size(relid) DESC
        LIMIT 15;
    " 2>/dev/null
}

cmd_db_migrate() {
    print_header "Running Migrations"
    print_info "Migrations run automatically when ra-app starts."
    print_info "Restarting ra-app to trigger migration check ..."

    $DC restart ra-app
    print_success "ra-app restarted — check logs for migration output:"
    echo "  ./run.sh logs ra-app"
}

# ============================================================================
# CI / Testing Commands
# ============================================================================

cmd_ci() {
    print_header "CI Pipeline (fmt → clippy → test)"

    local start_time
    start_time=$(date +%s)
    local failed=0

    # Format check
    echo ""
    print_step "Checking code formatting ..."
    if cargo fmt --all -- --check; then
        print_success "Formatting OK"
    else
        print_error "Formatting check failed — fix with: cargo fmt --all"
        ((failed++))
    fi

    # Clippy
    echo ""
    print_step "Running clippy lints ..."
    if cargo clippy --all-targets -- -D warnings 2>&1; then
        print_success "Clippy OK"
    else
        print_error "Clippy found issues"
        ((failed++))
    fi

    # Tests
    echo ""
    print_step "Running tests ..."
    if cargo test --lib --bins 2>&1; then
        print_success "Tests passed"
    else
        print_error "Tests failed"
        ((failed++))
    fi

    # Summary
    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    echo ""
    print_header "CI Summary"
    echo "  Duration: ${duration}s"
    echo ""

    if [ $failed -eq 0 ]; then
        print_success "All CI checks passed! Ready to commit."
        return 0
    else
        print_error "CI failed with $failed error(s). Fix before pushing."
        return 1
    fi
}

cmd_test() {
    print_header "Running Tests"
    cargo test --lib --bins "$@"
}

cmd_fmt() {
    print_header "Formatting Code"
    cargo fmt --all
    print_success "Code formatted"
}

cmd_clippy() {
    print_header "Running Clippy"
    cargo clippy --all-targets -- -D warnings
}

cmd_check() {
    print_header "Cargo Check"
    cargo check --all-targets
    print_success "Check passed"
}

cmd_build_local() {
    print_header "Building Release Binary (local)"
    cargo build --release --bin rustassistant-server
    print_success "Binary built: target/release/rustassistant-server"
}

# ============================================================================
# Development (local, no Docker)
# ============================================================================

cmd_dev() {
    print_header "Starting Development Server (local, no Docker)"

    if [ ! -f "$ENV_FILE" ]; then
        print_error ".env file not found — copy .env.example first."
        return 1
    fi

    # Override for local dev
    export DATABASE_URL="postgres://rustassistant:${RA_POSTGRES_PASSWORD:-dev}@localhost:5433/rustassistant"
    export REDIS_URL="redis://:${RA_REDIS_PASSWORD:-dev}@localhost:6380"
    export OLLAMA_BASE_URL="http://localhost:11434"
    export HOST="${HOST:-127.0.0.1}"
    export PORT="${PORT:-3500}"

    print_info "Ensure Postgres and Redis are running (Ollama optional):"
    echo "  ./run.sh up ra-postgres ra-redis            # remote LLM only"
    echo "  ./run.sh up --ollama ra-postgres ra-redis   # with local LLM"
    echo ""
    print_info "Starting server on http://${HOST}:${PORT}"
    print_info "Press Ctrl+C to stop"
    echo ""

    cargo run --bin rustassistant-server
}

cmd_watch() {
    print_header "Starting Development Server with Auto-Reload"

    if ! command -v cargo-watch &> /dev/null; then
        print_warning "cargo-watch not installed. Installing ..."
        cargo install cargo-watch
    fi

    # Same env as cmd_dev
    export DATABASE_URL="postgres://rustassistant:${RA_POSTGRES_PASSWORD:-dev}@localhost:5433/rustassistant"
    export REDIS_URL="redis://:${RA_REDIS_PASSWORD:-dev}@localhost:6380"
    export OLLAMA_BASE_URL="http://localhost:11434"
    export HOST="${HOST:-127.0.0.1}"
    export PORT="${PORT:-3500}"

    print_info "Watching for changes ... (http://${HOST}:${PORT})"
    print_info "Press Ctrl+C to stop"
    echo ""

    cargo watch -x 'run --bin rustassistant-server' \
        -w src/ -w templates/ -w static/ -w migrations/ -w Cargo.toml
}

# ============================================================================
# Cleanup Commands
# ============================================================================

cmd_clean() {
    print_header "Cleaning Docker Resources"

    echo "This will remove:"
    echo "  - Stopped RustAssistant / OpenClaw containers"
    echo "  - Unused networks"
    echo "  - Dangling images"
    echo ""
    echo -n "Continue? (y/N) "
    read -r -n 1 REPLY
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_step "Stopping containers ..."
        $DC down --remove-orphans --timeout 10 2>/dev/null || true

        print_step "Removing orphan containers ..."
        docker ps -a --filter "name=ra-" --filter "name=openclaw" -q \
            | xargs -r docker rm -f 2>/dev/null || true

        print_step "Pruning networks ..."
        docker network prune -f 2>/dev/null || true

        print_success "Cleanup complete"
    else
        print_info "Cancelled"
    fi
}

cmd_force_clean() {
    print_header "Force Cleanup"

    print_warning "This will remove ALL containers, networks, AND volumes!"
    print_warning "All database data, Redis cache, Ollama models (if any), and workspace files will be PERMANENTLY LOST!"
    echo ""
    echo -n "Type 'yes' to confirm: "
    read -r response

    if [ "$response" = "yes" ]; then
        print_step "Stopping and removing everything ..."
        $DC down -v --remove-orphans 2>/dev/null || true

        print_step "Removing orphan containers ..."
        docker ps -a --filter "name=ra-" --filter "name=openclaw" -q \
            | xargs -r docker rm -f 2>/dev/null || true

        print_step "Removing named volumes ..."
        docker volume ls --filter "name=rustassistant_" -q \
            | xargs -r docker volume rm 2>/dev/null || true

        print_step "Removing images ..."
        docker image rm openclaw:local openclaw-base:local 2>/dev/null || true

        print_success "Force cleanup complete — all data removed"
        print_info "Start fresh with: ./run.sh up"
    else
        print_info "Cancelled"
    fi
}

# ============================================================================
# Diagnostics
# ============================================================================

cmd_diagnose() {
    print_header "RustAssistant Stack Diagnostics"

    # Docker
    echo -e "\n${BOLD}Docker${NC}"
    docker version --format '  Engine:  {{.Server.Version}}' 2>/dev/null || echo "  Engine: not running"
    docker compose version 2>/dev/null | sed 's/^/  /'

    # Tailscale
    echo -e "\n${BOLD}Tailscale${NC}"
    if command -v tailscale &> /dev/null; then
        local ts_status
        ts_status=$(tailscale status --self --json 2>/dev/null)
        if [ -n "$ts_status" ]; then
            local ts_ip ts_name
            ts_ip=$(echo "$ts_status" | grep -o '"TailscaleIPs":\["[^"]*"' | cut -d'"' -f4)
            ts_name=$(echo "$ts_status" | grep -o '"DNSName":"[^"]*"' | cut -d'"' -f4 | sed 's/\.$//')
            echo "  Status:   Running"
            echo "  IP:       ${ts_ip:-unknown}"
            echo "  Hostname: ${ts_name:-unknown}"
            if [ -n "$TAILSCALE_IP" ] && [ "$TAILSCALE_IP" = "$ts_ip" ]; then
                print_success "  TAILSCALE_IP in .env matches active interface"
            elif [ -n "$TAILSCALE_IP" ]; then
                print_warning "  TAILSCALE_IP in .env ($TAILSCALE_IP) differs from active ($ts_ip)"
            fi
        else
            echo "  Status: Not connected"
        fi

        # Tailscale serve
        echo ""
        echo -e "${BOLD}Tailscale Serve${NC}"
        tailscale serve status 2>/dev/null | sed 's/^/  /' || echo "  Not configured"
    else
        echo "  Tailscale CLI not installed"
    fi

    # Ports
    echo -e "\n${BOLD}Port Bindings${NC}"
    local ports=("${RA_PORT}:ra-app" "5433:ra-postgres" "6380:ra-redis" "11434:ra-ollama" "${OPENCLAW_PORT}:openclaw-gateway" "18790:openclaw-gateway")
    for entry in "${ports[@]}"; do
        local port="${entry%%:*}"
        local svc="${entry#*:}"
        local container
        container=$(docker ps --filter "publish=$port" --format "{{.Names}}" 2>/dev/null | head -1)
        if [ -n "$container" ]; then
            echo -e "  ${TAILSCALE_IP}:${port} → ${GREEN}${container}${NC}"
        else
            echo -e "  ${TAILSCALE_IP}:${port} → ${YELLOW}not bound${NC} (expected: ${svc})"
        fi
    done

    # Containers
    echo -e "\n${BOLD}Containers${NC}"
    $DC ps --format "  {{.Name}}\t{{.Status}}" 2>/dev/null || $DC ps

    # Volumes
    echo -e "\n${BOLD}Volumes${NC}"
    docker volume ls --filter "name=rustassistant_" --format "  {{.Name}}" 2>/dev/null
    docker volume ls --filter "name=rustassistant_openclaw" --format "  {{.Name}}" 2>/dev/null

    # Images
    echo -e "\n${BOLD}Images${NC}"
    docker images --format "  {{.Repository}}:{{.Tag}}\t{{.Size}}\t({{.CreatedSince}})" \
        --filter "reference=openclaw*" --filter "reference=*rustassistant*" 2>/dev/null \
        || echo "  No matching images"

    # .env check
    echo -e "\n${BOLD}Configuration (.env)${NC}"
    if [ -f "$ENV_FILE" ]; then
        local keys=("TAILSCALE_IP" "XAI_API_KEY" "RA_POSTGRES_PASSWORD" "RA_REDIS_PASSWORD"
                     "RA_PROXY_API_KEYS" "OPENCLAW_GATEWAY_TOKEN" "DISCORD_BOT_TOKEN" "GITHUB_TOKEN")
        for key in "${keys[@]}"; do
            local val
            val=$(grep "^${key}=" "$ENV_FILE" 2>/dev/null | cut -d'=' -f2- | tr -d ' ')
            if [ -n "$val" ] && [[ ! "$val" =~ ^(xai-your|ghp_your|your-) ]]; then
                echo -e "  ${key}: ${GREEN}set${NC}"
            else
                echo -e "  ${key}: ${YELLOW}not set${NC}"
            fi
        done
    else
        print_warning ".env file not found"
    fi

    # Rust toolchain
    echo -e "\n${BOLD}Rust Toolchain (local)${NC}"
    if command -v rustc &> /dev/null; then
        echo "  rustc: $(rustc --version)"
        echo "  cargo: $(cargo --version)"
    else
        echo "  Not installed locally (Docker builds still work)"
    fi

    # Disk space
    echo -e "\n${BOLD}Disk Space${NC}"
    df -h "$SCRIPT_DIR" | tail -1 | awk '{printf "  Available: %s / %s (%s used)\n", $4, $2, $5}'
    docker system df --format "  {{.Type}}: {{.Size}} ({{.Reclaimable}} reclaimable)" 2>/dev/null
}

# ============================================================================
# Usage
# ============================================================================

show_usage() {
    cat << 'EOF'
RustAssistant - Project Management Script

Usage: ./run.sh <command> [options]

Stack Commands:
  up [flags] [services]         Build images + start services (default: all)
    --ollama                    Include Ollama local LLM
    --skip-build                Skip image builds
    --rebuild                   Force rebuild even if images exist
  down [-v]                     Stop services (-v removes volumes)
  restart [services]            Restart services
  logs [services]               Tail service logs
  status                        Show container status
  health                        Check health of all services
  shell [service]               Open shell in container (default: ra-app)

Build Commands:
  build [target]                Build images (openclaw | app | all)
  build openclaw                Build OpenClaw base + RA config layer
  build app                     Build ra-app image
  build-local                   Build release binary locally (cargo)

OpenClaw Commands:
  openclaw <subcommand>         Run any OpenClaw CLI command via Docker
  openclaw doctor               OpenClaw health check
  openclaw models status        Show configured model
  openclaw devices list         Show paired devices
  openclaw dashboard --no-open  Get tokenized dashboard URL
  openclaw channels status      Show channel health

Development Commands:
  dev                           Run server locally (needs Postgres/Redis up; Ollama optional)
  watch                         Run with auto-reload (cargo-watch)
  check                         Run cargo check
  fmt                           Format code (cargo fmt)
  clippy                        Run clippy lints

Testing & CI:
  test [args]                   Run tests (cargo test)
  ci                            Full CI pipeline: fmt + clippy + test

Database Commands (Postgres):
  db backup [path]              Dump database to ./backups/
  db restore <file>             Restore from .sql.gz backup
  db shell                      Open psql CLI
  db size                       Show database and table sizes
  db migrate                    Restart ra-app to trigger migrations

Maintenance:
  diagnose                      Full system diagnostics
  clean                         Remove stopped containers and dangling images
  force-clean                   DANGER: Remove ALL data, volumes, and containers
  preflight                     Run pre-flight checks only

Services:
  ra-app               RustAssistant API server (port 3500)
  ra-postgres           PostgreSQL 16 (port 5433)
  ra-redis              Redis 7 cache (port 6380)
  ra-ollama             Ollama local LLM (port 11434) — optional, use --ollama
  ra-ollama-init        One-shot model puller — optional, use --ollama
  openclaw-gateway      OpenClaw Gateway (port 18789/18790)
  openclaw-cli          OpenClaw CLI (ephemeral, run via ./run.sh openclaw)

All ports bind to TAILSCALE_IP (currently: ${TAILSCALE_IP:-<not set>}).
OpenClaw Control UI available via Tailscale Serve (HTTPS).

Examples:
  ./run.sh up                              # Build and start (no Ollama)
  ./run.sh up --ollama                     # Build and start with Ollama
  ./run.sh up --skip-build                 # Start without rebuilding
  ./run.sh up ra-app ra-postgres ra-redis  # Start specific services
  ./run.sh logs openclaw-gateway           # Tail gateway logs
  ./run.sh openclaw doctor                 # OpenClaw diagnostics
  ./run.sh openclaw models set openai/rustassistant
  ./run.sh health                          # Check all services
  ./run.sh db backup                       # Backup Postgres
  ./run.sh db shell                        # Open psql
  ./run.sh dev                             # Local dev (cargo run)
  ./run.sh ci                              # Full CI before pushing
  ./run.sh diagnose                        # Debug everything
  ./run.sh down -v                         # Stop + remove volumes
  ./run.sh force-clean                     # Nuclear option
EOF
}

# ============================================================================
# Main Dispatcher
# ============================================================================

main() {
    if [ $# -lt 1 ]; then
        show_usage
        exit 0
    fi

    local command="$1"
    shift

    case "$command" in
        # ── Stack commands ───────────────────────────────────────────────────
        up|start)           cmd_up "$@" ;;
        down|stop)          cmd_down "$@" ;;
        restart)            cmd_restart "$@" ;;
        logs|log)           cmd_logs "$@" ;;
        status|ps)          cmd_status ;;
        health)             cmd_health ;;
        shell|exec)         cmd_shell "$@" ;;

        # ── Build commands ───────────────────────────────────────────────────
        build)              cmd_build "$@" ;;
        build-local|build-release)
                            cmd_build_local ;;

        # ── OpenClaw passthrough ─────────────────────────────────────────────
        openclaw|oc|claw)   cmd_openclaw "$@" ;;

        # ── Development ──────────────────────────────────────────────────────
        dev|run)            cmd_dev ;;
        watch)              cmd_watch ;;
        check)              cmd_check ;;
        fmt|format)         cmd_fmt ;;
        clippy|lint)        cmd_clippy ;;

        # ── Testing & CI ─────────────────────────────────────────────────────
        test|tests)         cmd_test "$@" ;;
        ci)                 cmd_ci ;;

        # ── Database ─────────────────────────────────────────────────────────
        db|database)        cmd_db "$@" ;;

        # ── Maintenance ──────────────────────────────────────────────────────
        diagnose|diag|debug)
                            cmd_diagnose ;;
        clean|cleanup)      cmd_clean ;;
        force-clean|nuke)   cmd_force_clean ;;
        preflight)          preflight_check ;;

        # ── Help ─────────────────────────────────────────────────────────────
        help|--help|-h)     show_usage ;;

        *)
            print_error "Unknown command: $command"
            echo ""
            echo "Run './run.sh help' for usage information."
            exit 1
            ;;
    esac
}

main "$@"
