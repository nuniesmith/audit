#!/usr/bin/env bash
# ============================================================================
# RustAssistant - Project Management Script
# ============================================================================
# Wrapper for managing the RustAssistant Docker environment, running tests,
# performing database backups, and general development tasks.
#
# Usage:
#   ./run.sh up [prod]         # Start services (dev or prod mode)
#   ./run.sh down              # Stop all services
#   ./run.sh status            # Show service status
#   ./run.sh logs [service]    # Tail service logs
#   ./run.sh health            # Check service health
#   ./run.sh test              # Run Rust tests
#   ./run.sh ci                # Run full CI pipeline (fmt + clippy + test)
#   ./run.sh db backup         # Backup SQLite database
#   ./run.sh diagnose          # System diagnostics
#   ./run.sh help              # Show full usage
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Docker Compose files
COMPOSE_FILE="docker-compose.yml"
PROD_COMPOSE_FILE="docker-compose.prod.yml"
ADVANCED_COMPOSE_FILE="docker-compose.advanced.yml"
ENV_FILE=".env"
ENV_EXAMPLE=".env.example"

# Project settings
PROJECT_NAME="rustassistant"
DC="docker compose -p $PROJECT_NAME --env-file $ENV_FILE"
DEFAULT_PORT=3000

# ============================================================================
# Helper Functions
# ============================================================================

print_header() {
    echo ""
    echo -e "${BLUE}===================================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}===================================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

print_step() {
    echo -e "${CYAN}→ $1${NC}"
}

# ============================================================================
# Environment Setup
# ============================================================================

generate_secret() {
    openssl rand -hex 16 2>/dev/null || head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n' | head -c 32
}

setup_env_file() {
    if [ -f "$ENV_FILE" ]; then
        print_info ".env file exists, validating..."

        local needs_update=false

        # Check critical keys
        local xai_key
        xai_key=$(grep "^XAI_API_KEY=" "$ENV_FILE" 2>/dev/null | cut -d'=' -f2- | tr -d ' ')
        if [ -z "$xai_key" ] || [ "$xai_key" = "xai-your-api-key-here" ]; then
            print_warning "XAI_API_KEY is not set — LLM analysis will not work"
            print_info "  Set it in .env: XAI_API_KEY=xai-..."
            needs_update=true
        fi

        if [ "$needs_update" = false ]; then
            print_success ".env validated — all required keys present"
        fi
        return 0
    fi

    print_header "Generating .env Configuration"

    if [ -f "$ENV_EXAMPLE" ]; then
        cp "$ENV_EXAMPLE" "$ENV_FILE"
        print_success "Created .env from .env.example"
    else
        cat > "$ENV_FILE" << 'ENVEOF'
# ============================================================================
# RustAssistant Environment Configuration
# ============================================================================

# ----------------------------------------------------------------------------
# XAI API Configuration (Grok) — REQUIRED for LLM analysis
# ----------------------------------------------------------------------------
XAI_API_KEY=xai-your-api-key-here
XAI_BASE_URL=https://api.x.ai/v1
XAI_MODEL=grok-4-1-fast-reasoning
XAI_MAX_TOKENS=4096
XAI_TEMPERATURE=0.3

# ----------------------------------------------------------------------------
# Server Configuration
# ----------------------------------------------------------------------------
HOST=0.0.0.0
PORT=3000
RUST_LOG=info,rustassistant=debug
RUST_BACKTRACE=1

# ----------------------------------------------------------------------------
# GitHub Integration (Optional — for cloning private repos)
# ----------------------------------------------------------------------------
GITHUB_TOKEN=
GITHUB_BASE_URL=https://api.github.com

# ----------------------------------------------------------------------------
# Database Configuration
# ----------------------------------------------------------------------------
DATABASE_URL=sqlite:/app/data/rustassistant.db
CACHE_DB_PATH=/app/data/rustassistant_cache.db

# ----------------------------------------------------------------------------
# Auto-Scanner Configuration
# ----------------------------------------------------------------------------
AUTO_SCAN_ENABLED=true
AUTO_SCAN_INTERVAL=60
AUTO_SCAN_MAX_CONCURRENT=2
AUTO_SCAN_COST_BUDGET=10.00

# ----------------------------------------------------------------------------
# Storage Paths (inside container)
# ----------------------------------------------------------------------------
REPOS_DIR=/app/repos
DATA_DIR=/app/data

# ----------------------------------------------------------------------------
# Feature Flags
# ----------------------------------------------------------------------------
ENABLE_WEB_UI=true
ENVEOF
        print_success "Created default .env file"
    fi

    echo ""
    print_warning "IMPORTANT: Edit .env and set your XAI_API_KEY before starting"
    print_info "  nano .env   (or your preferred editor)"
}

# ============================================================================
# Pre-flight Checks
# ============================================================================

preflight_check() {
    print_header "Pre-flight Checks"

    local errors=0

    # Check Docker daemon
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker daemon is not running"
        ((errors++))
    else
        print_success "Docker daemon is running"
    fi

    # Check Docker Compose
    if ! docker compose version > /dev/null 2>&1; then
        print_error "Docker Compose V2 is not available"
        ((errors++))
    else
        print_success "Docker Compose is available"
    fi

    # Check disk space
    local available_gb
    available_gb=$(df "$SCRIPT_DIR" | tail -1 | awk '{print int($4/1024/1024)}')
    if [ "$available_gb" -lt 2 ]; then
        print_error "Low disk space: ${available_gb}GB available (need at least 2GB)"
        ((errors++))
    else
        print_success "Disk space OK: ${available_gb}GB available"
    fi

    # Check port availability
    local port="${PORT:-$DEFAULT_PORT}"
    local container
    container=$(docker ps --filter "publish=$port" --format "{{.Names}}" 2>/dev/null)
    if [ -n "$container" ] && [ "$container" != "$PROJECT_NAME" ]; then
        print_error "Port $port is already in use by: $container"
        ((errors++))
    else
        print_success "Port $port is available"
    fi

    # Check .env file
    if [ ! -f "$ENV_FILE" ]; then
        print_warning ".env file not found — will generate on start"
    else
        print_success ".env file exists"
    fi

    if [ $errors -gt 0 ]; then
        echo ""
        print_error "Pre-flight failed with $errors error(s)"
        return 1
    else
        print_success "All checks passed"
        return 0
    fi
}

# ============================================================================
# Docker Commands
# ============================================================================

cmd_up() {
    local mode=$1
    shift 2>/dev/null || true

    if [ "$mode" = "prod" ]; then
        print_header "Starting RustAssistant (Production)"
        print_info "Mode: Production — pulling pre-built image"
    elif [ "$mode" = "advanced" ]; then
        print_header "Starting RustAssistant (Advanced)"
        print_info "Mode: Advanced — Postgres, Redis Sentinel, Jaeger, Grafana"
    else
        print_header "Starting RustAssistant (Development)"
        print_info "Mode: Development — building from source"
    fi

    # Pre-flight
    echo ""
    if ! preflight_check; then
        echo ""
        print_info "Fix the errors above, or try: ./run.sh force-clean"
        exit 1
    fi

    # Ensure .env exists
    echo ""
    setup_env_file

    # Stop existing containers
    echo ""
    print_step "Stopping existing containers..."
    if [ "$mode" = "prod" ]; then
        $DC -f "$PROD_COMPOSE_FILE" down --remove-orphans --timeout 10 2>/dev/null || true
    elif [ "$mode" = "advanced" ]; then
        $DC -f "$ADVANCED_COMPOSE_FILE" down --remove-orphans --timeout 10 2>/dev/null || true
    else
        $DC -f "$COMPOSE_FILE" down --remove-orphans --timeout 10 2>/dev/null || true
    fi

    # Build/Pull
    echo ""
    if [ "$mode" = "prod" ]; then
        print_step "Pulling production images..."
        $DC -f "$PROD_COMPOSE_FILE" pull "$@"
    elif [ "$mode" = "advanced" ]; then
        print_step "Building advanced stack..."
        $DC -f "$ADVANCED_COMPOSE_FILE" build "$@"
    else
        print_step "Building images from source..."
        $DC -f "$COMPOSE_FILE" build "$@"
    fi

    # Start
    echo ""
    if [ "$mode" = "prod" ]; then
        print_step "Starting production services..."
        $DC -f "$PROD_COMPOSE_FILE" up -d "$@"
    elif [ "$mode" = "advanced" ]; then
        print_step "Starting advanced services..."
        $DC -f "$ADVANCED_COMPOSE_FILE" up -d "$@"
    else
        print_step "Starting development services..."
        $DC -f "$COMPOSE_FILE" up -d "$@"
    fi

    print_success "Services started"
    print_info "Waiting for health checks..."
    sleep 5

    # Show status
    cmd_status "$mode"

    # Print access info
    local port="${PORT:-$DEFAULT_PORT}"
    echo ""
    print_success "RustAssistant is ready!"
    echo ""
    print_info "Access points:"
    echo "  Dashboard:        http://localhost:${port}/"
    echo "  Repos:            http://localhost:${port}/repos"
    echo "  Scanner:          http://localhost:${port}/scanner"
    echo "  Scan Progress:    http://localhost:${port}/scan/dashboard"
    echo "  DB Explorer:      http://localhost:${port}/db"
    echo "  Cache Viewer:     http://localhost:${port}/cache"
    echo "  Queue:            http://localhost:${port}/queue"
    echo "  Health:           http://localhost:${port}/health"
    echo ""
    print_info "Useful commands:"
    echo "  ./run.sh logs             # Tail logs"
    echo "  ./run.sh logs redis       # Tail Redis logs"
    echo "  ./run.sh health           # Check service health"
    echo "  ./run.sh status           # Container status"
    echo "  ./run.sh down             # Stop everything"
}

cmd_down() {
    local mode=$1
    shift 2>/dev/null || true

    print_header "Stopping RustAssistant"

    if [ "$mode" = "prod" ]; then
        $DC -f "$PROD_COMPOSE_FILE" down --remove-orphans "$@"
    elif [ "$mode" = "advanced" ]; then
        $DC -f "$ADVANCED_COMPOSE_FILE" down --remove-orphans "$@"
    else
        $DC -f "$COMPOSE_FILE" down --remove-orphans "$@"
    fi

    print_success "Services stopped"
}

cmd_restart() {
    local mode=$1
    shift 2>/dev/null || true

    print_header "Restarting RustAssistant"

    if [ "$mode" = "prod" ]; then
        $DC -f "$PROD_COMPOSE_FILE" restart "$@"
    elif [ "$mode" = "advanced" ]; then
        $DC -f "$ADVANCED_COMPOSE_FILE" restart "$@"
    else
        $DC -f "$COMPOSE_FILE" restart "$@"
    fi

    print_success "Services restarted"
}

cmd_logs() {
    local mode=$1
    shift 2>/dev/null || true

    if [ "$mode" = "prod" ]; then
        $DC -f "$PROD_COMPOSE_FILE" logs -f --tail=100 "$@"
    elif [ "$mode" = "advanced" ]; then
        $DC -f "$ADVANCED_COMPOSE_FILE" logs -f --tail=100 "$@"
    else
        $DC -f "$COMPOSE_FILE" logs -f --tail=100 "$@"
    fi
}

cmd_status() {
    local mode=$1

    print_header "Service Status"

    if [ "$mode" = "prod" ]; then
        $DC -f "$PROD_COMPOSE_FILE" ps
    elif [ "$mode" = "advanced" ]; then
        $DC -f "$ADVANCED_COMPOSE_FILE" ps
    else
        $DC -f "$COMPOSE_FILE" ps
    fi
}

cmd_build() {
    local mode=$1
    shift 2>/dev/null || true

    if [ "$mode" = "prod" ]; then
        print_header "Pulling Production Images"
        $DC -f "$PROD_COMPOSE_FILE" pull "$@"
    else
        print_header "Building Development Images"
        $DC -f "$COMPOSE_FILE" build "$@"
    fi

    print_success "Build complete"
}

cmd_shell() {
    local service=${1:-rustassistant}

    print_info "Opening shell in $service..."
    $DC -f "$COMPOSE_FILE" exec "$service" /bin/bash 2>/dev/null || \
    $DC -f "$COMPOSE_FILE" exec "$service" /bin/sh
}

# ============================================================================
# Health Checks
# ============================================================================

cmd_health() {
    print_header "Service Health"

    local port="${PORT:-$DEFAULT_PORT}"
    local all_healthy=true

    # RustAssistant
    echo -e "\n${BOLD}RustAssistant Server${NC}"
    local health_response
    health_response=$(curl -sf "http://localhost:${port}/health" 2>/dev/null)
    if [ $? -eq 0 ]; then
        print_success "Server is healthy (port $port)"
        if [ -n "$health_response" ]; then
            echo "  Response: $health_response"
        fi
    else
        print_error "Server is not responding on port $port"
        all_healthy=false
    fi

    # Web UI
    echo -e "\n${BOLD}Web UI${NC}"
    if curl -sf "http://localhost:${port}/" > /dev/null 2>&1; then
        print_success "Web UI is accessible"
    else
        print_error "Web UI is not accessible"
        all_healthy=false
    fi

    # Redis
    echo -e "\n${BOLD}Redis Cache${NC}"
    if $DC -f "$COMPOSE_FILE" exec -T redis redis-cli ping 2>/dev/null | grep -q PONG; then
        print_success "Redis is healthy"
        local redis_mem
        redis_mem=$($DC -f "$COMPOSE_FILE" exec -T redis redis-cli info memory 2>/dev/null | grep "used_memory_human" | cut -d: -f2 | tr -d '\r')
        if [ -n "$redis_mem" ]; then
            echo "  Memory used: $redis_mem"
        fi
    else
        print_error "Redis is not responding"
        all_healthy=false
    fi

    # Database check (via the health endpoint)
    echo -e "\n${BOLD}SQLite Database${NC}"
    if curl -sf "http://localhost:${port}/db" > /dev/null 2>&1; then
        print_success "Database is accessible via DB Explorer"
    else
        print_warning "DB Explorer not accessible (server may still be starting)"
    fi

    # Docker container status
    echo -e "\n${BOLD}Container Status${NC}"
    $DC -f "$COMPOSE_FILE" ps --format "table {{.Name}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null || \
    $DC -f "$COMPOSE_FILE" ps

    echo ""
    if [ "$all_healthy" = true ]; then
        print_success "All services healthy!"
    else
        print_error "Some services are unhealthy — check logs with: ./run.sh logs"
    fi
}

# ============================================================================
# Database Commands
# ============================================================================

cmd_db() {
    local subcmd=$1
    shift 2>/dev/null || true

    case "$subcmd" in
        backup)
            cmd_db_backup "$@"
            ;;
        restore)
            cmd_db_restore "$@"
            ;;
        shell|cli)
            cmd_db_shell
            ;;
        migrate)
            cmd_db_migrate
            ;;
        size)
            cmd_db_size
            ;;
        *)
            echo "Database commands:"
            echo "  ./run.sh db backup [path]    Backup SQLite database"
            echo "  ./run.sh db restore <file>   Restore from backup"
            echo "  ./run.sh db shell            Open SQLite CLI"
            echo "  ./run.sh db migrate          Run pending migrations"
            echo "  ./run.sh db size             Show database size"
            ;;
    esac
}

cmd_db_backup() {
    print_header "Database Backup"

    local backup_dir="${1:-./backups}"
    local timestamp
    timestamp=$(date +%Y%m%d-%H%M%S)
    local backup_file="${backup_dir}/rustassistant-${timestamp}.db"
    local cache_backup="${backup_dir}/rustassistant_cache-${timestamp}.db"

    mkdir -p "$backup_dir"

    # Check if container is running
    if docker ps --filter "name=$PROJECT_NAME" --filter "status=running" --format "{{.Names}}" | grep -q "$PROJECT_NAME"; then
        print_step "Backing up from running container..."

        # Use docker cp to extract the database files
        docker cp "${PROJECT_NAME}:/app/data/rustassistant.db" "$backup_file" 2>/dev/null
        if [ $? -eq 0 ]; then
            print_success "Main DB backed up to: $backup_file"
            local size
            size=$(du -h "$backup_file" | cut -f1)
            echo "  Size: $size"
        else
            print_error "Failed to backup main database"
        fi

        docker cp "${PROJECT_NAME}:/app/data/rustassistant_cache.db" "$cache_backup" 2>/dev/null
        if [ $? -eq 0 ]; then
            print_success "Cache DB backed up to: $cache_backup"
        else
            print_warning "Cache database not found (may not exist yet)"
        fi
    else
        # Container not running — try to get from Docker volume
        print_step "Container not running — extracting from volume..."

        docker run --rm \
            -v rustassistant_data:/data:ro \
            -v "$(pwd)/$backup_dir:/backup" \
            alpine cp /data/rustassistant.db "/backup/rustassistant-${timestamp}.db" 2>/dev/null

        if [ $? -eq 0 ]; then
            print_success "Main DB backed up to: $backup_file"
        else
            print_error "Failed to extract database from volume"
            print_info "Is the volume 'rustassistant_data' present?"
            return 1
        fi

        docker run --rm \
            -v rustassistant_data:/data:ro \
            -v "$(pwd)/$backup_dir:/backup" \
            alpine cp /data/rustassistant_cache.db "/backup/rustassistant_cache-${timestamp}.db" 2>/dev/null || true
    fi

    echo ""
    print_info "Backups in $backup_dir/:"
    ls -lh "$backup_dir"/*"${timestamp}"* 2>/dev/null
}

cmd_db_restore() {
    local backup_file=$1

    if [ -z "$backup_file" ]; then
        print_error "Usage: ./run.sh db restore <backup-file>"
        echo ""
        print_info "Available backups:"
        ls -lh backups/*.db 2>/dev/null || echo "  No backups found in ./backups/"
        return 1
    fi

    if [ ! -f "$backup_file" ]; then
        print_error "Backup file not found: $backup_file"
        return 1
    fi

    print_header "Database Restore"
    print_warning "This will replace the current database!"
    echo -n "Continue? (y/N) "
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        print_info "Restore cancelled"
        return 0
    fi

    # Stop the server first
    print_step "Stopping server..."
    $DC -f "$COMPOSE_FILE" stop rustassistant 2>/dev/null || true

    # Copy backup into the volume
    print_step "Restoring database..."
    docker run --rm \
        -v rustassistant_data:/data \
        -v "$(cd "$(dirname "$backup_file")" && pwd):/backup:ro" \
        alpine cp "/backup/$(basename "$backup_file")" /data/rustassistant.db

    if [ $? -eq 0 ]; then
        print_success "Database restored from: $backup_file"
        print_info "Start the server with: ./run.sh up"
    else
        print_error "Restore failed"
        return 1
    fi
}

cmd_db_shell() {
    print_info "Opening SQLite CLI on the database..."

    if docker ps --filter "name=$PROJECT_NAME" --filter "status=running" --format "{{.Names}}" | grep -q "$PROJECT_NAME"; then
        docker exec -it "$PROJECT_NAME" sqlite3 /app/data/rustassistant.db
    else
        print_warning "Container not running — mounting volume directly"
        docker run --rm -it \
            -v rustassistant_data:/data \
            keinos/sqlite3 /data/rustassistant.db
    fi
}

cmd_db_migrate() {
    print_header "Running Migrations"

    if docker ps --filter "name=$PROJECT_NAME" --filter "status=running" --format "{{.Names}}" | grep -q "$PROJECT_NAME"; then
        print_info "Migrations run automatically on server start."
        print_info "To force re-run, restart the server:"
        echo "  ./run.sh restart"
    else
        print_warning "Server is not running. Migrations will run on next start."
        print_info "Start with: ./run.sh up"
    fi
}

cmd_db_size() {
    print_header "Database Size"

    if docker ps --filter "name=$PROJECT_NAME" --filter "status=running" --format "{{.Names}}" | grep -q "$PROJECT_NAME"; then
        docker exec "$PROJECT_NAME" sh -c '
            echo "Main database:"
            ls -lh /app/data/rustassistant.db 2>/dev/null || echo "  Not found"
            echo ""
            echo "Cache database:"
            ls -lh /app/data/rustassistant_cache.db 2>/dev/null || echo "  Not found"
            echo ""
            echo "All data files:"
            du -sh /app/data/ 2>/dev/null || echo "  N/A"
            echo ""
            echo "Repos directory:"
            du -sh /app/repos/ 2>/dev/null || echo "  N/A"
        '
    else
        print_info "Checking volumes..."
        docker run --rm \
            -v rustassistant_data:/data:ro \
            -v rustassistant_repos_data:/repos:ro \
            alpine sh -c '
                echo "Main database:"
                ls -lh /data/rustassistant.db 2>/dev/null || echo "  Not found"
                echo ""
                echo "Cache database:"
                ls -lh /data/rustassistant_cache.db 2>/dev/null || echo "  Not found"
                echo ""
                echo "Total data:"
                du -sh /data/ 2>/dev/null || echo "  N/A"
                echo ""
                echo "Repos:"
                du -sh /repos/ 2>/dev/null || echo "  N/A"
            '
    fi
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
    print_step "Checking code formatting..."
    if cargo fmt --all -- --check; then
        print_success "Formatting OK"
    else
        print_error "Formatting check failed"
        print_info "Fix with: cargo fmt --all"
        ((failed++))
    fi

    # Clippy
    echo ""
    print_step "Running clippy lints..."
    if cargo clippy --all-targets -- -D warnings 2>&1; then
        print_success "Clippy OK"
    else
        print_error "Clippy found issues"
        ((failed++))
    fi

    # Tests
    echo ""
    print_step "Running tests..."
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
    echo -e "  Duration: ${duration}s"
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
# Cleanup Commands
# ============================================================================

cmd_clean() {
    print_header "Cleaning Docker Resources"

    echo "This will remove:"
    echo "  - Stopped RustAssistant containers"
    echo "  - Unused networks"
    echo "  - Dangling images"
    echo ""
    echo -n "Continue? (y/N) "
    read -r -n 1 REPLY
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_step "Stopping containers..."
        $DC -f "$COMPOSE_FILE" down --remove-orphans --timeout 10 2>/dev/null || true

        print_step "Removing orphan containers..."
        docker ps -a --filter "name=rustassistant" -q | xargs -r docker rm -f 2>/dev/null || true

        print_step "Pruning networks..."
        docker network prune -f 2>/dev/null || true

        print_success "Cleanup complete"
    else
        print_info "Cancelled"
    fi
}

cmd_force_clean() {
    print_header "Force Cleanup"

    print_warning "This will remove ALL RustAssistant containers, networks, AND volumes!"
    print_warning "All database data will be PERMANENTLY LOST!"
    echo ""
    echo -n "Type 'yes' to confirm: "
    read -r response

    if [[ "$response" = "yes" ]]; then
        print_step "Removing all containers..."
        docker ps -a --filter "name=rustassistant" -q | xargs -r docker rm -f 2>/dev/null || true

        print_step "Stopping compose..."
        $DC -f "$COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true
        $DC -f "$PROD_COMPOSE_FILE" down -v --remove-orphans 2>/dev/null || true

        print_step "Removing volumes..."
        docker volume ls --filter "name=rustassistant" -q | xargs -r docker volume rm 2>/dev/null || true

        print_step "Removing networks..."
        docker network ls --filter "name=rustassistant" -q | xargs -r docker network rm 2>/dev/null || true

        print_success "Force cleanup complete — all RustAssistant data removed"
        print_info "Start fresh with: ./run.sh up"
    else
        print_info "Cancelled"
    fi
}

# ============================================================================
# Diagnostics
# ============================================================================

cmd_diagnose() {
    print_header "RustAssistant Diagnostics"

    # Docker info
    echo -e "\n${BOLD}Docker${NC}"
    docker version --format '  Engine:  {{.Server.Version}}' 2>/dev/null || echo "  Engine: not running"
    docker compose version 2>/dev/null | sed 's/^/  /'

    # Port check
    echo -e "\n${BOLD}Port Usage${NC}"
    local port="${PORT:-$DEFAULT_PORT}"
    local container
    container=$(docker ps --filter "publish=$port" --format "{{.Names}}" 2>/dev/null)
    if [ -n "$container" ]; then
        echo -e "  Port $port: ${YELLOW}IN USE${NC} by $container"
    else
        echo -e "  Port $port: ${GREEN}FREE${NC}"
    fi
    container=$(docker ps --filter "publish=6379" --format "{{.Names}}" 2>/dev/null)
    if [ -n "$container" ]; then
        echo -e "  Port 6379 (Redis): ${YELLOW}IN USE${NC} by $container"
    else
        echo -e "  Port 6379 (Redis): ${GREEN}FREE${NC}"
    fi

    # Containers
    echo -e "\n${BOLD}RustAssistant Containers${NC}"
    local count
    count=$(docker ps -a --filter "name=rustassistant" --format "{{.Names}}" 2>/dev/null | wc -l)
    if [ "$count" -gt 0 ]; then
        docker ps -a --filter "name=rustassistant" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" 2>/dev/null
    else
        echo "  No containers found"
    fi

    # Volumes
    echo -e "\n${BOLD}Volumes${NC}"
    local vol_count
    vol_count=$(docker volume ls --filter "name=rustassistant" --format "{{.Name}}" 2>/dev/null | wc -l)
    if [ "$vol_count" -gt 0 ]; then
        docker volume ls --filter "name=rustassistant" --format "  {{.Name}} ({{.Driver}})"
    else
        echo "  No volumes found"
    fi

    # Networks
    echo -e "\n${BOLD}Networks${NC}"
    local net_count
    net_count=$(docker network ls --filter "name=rustassistant" --format "{{.Name}}" 2>/dev/null | wc -l)
    if [ "$net_count" -gt 0 ]; then
        docker network ls --filter "name=rustassistant" --format "  {{.Name}} ({{.Driver}})"
    else
        echo "  No networks found"
    fi

    # .env check
    echo -e "\n${BOLD}Configuration${NC}"
    if [ -f "$ENV_FILE" ]; then
        print_success ".env file exists"
        local xai_key
        xai_key=$(grep "^XAI_API_KEY=" "$ENV_FILE" 2>/dev/null | cut -d'=' -f2- | tr -d ' ')
        if [ -n "$xai_key" ] && [ "$xai_key" != "xai-your-api-key-here" ]; then
            print_success "XAI_API_KEY is configured"
        else
            print_warning "XAI_API_KEY is not set"
        fi
        local gh_token
        gh_token=$(grep "^GITHUB_TOKEN=" "$ENV_FILE" 2>/dev/null | cut -d'=' -f2- | tr -d ' ')
        if [ -n "$gh_token" ] && [ "$gh_token" != "ghp_your_personal_access_token_here" ]; then
            print_success "GITHUB_TOKEN is configured"
        else
            print_info "GITHUB_TOKEN is not set (optional)"
        fi
    else
        print_warning ".env file not found"
    fi

    # Rust toolchain
    echo -e "\n${BOLD}Rust Toolchain (local)${NC}"
    if command -v rustc &> /dev/null; then
        echo "  rustc: $(rustc --version)"
        echo "  cargo: $(cargo --version)"
    else
        echo "  Rust not installed locally (Docker builds will still work)"
    fi

    # Disk space
    echo -e "\n${BOLD}Disk Space${NC}"
    df -h "$SCRIPT_DIR" | tail -1 | awk '{printf "  Available: %s / %s (%s used)\n", $4, $2, $5}'
    local docker_space
    docker_space=$(docker system df --format "table {{.Type}}\t{{.TotalCount}}\t{{.Size}}\t{{.Reclaimable}}" 2>/dev/null)
    if [ -n "$docker_space" ]; then
        echo ""
        echo "  Docker disk usage:"
        echo "$docker_space" | sed 's/^/    /'
    fi
}

# ============================================================================
# Quick Development Helpers
# ============================================================================

cmd_dev() {
    print_header "Starting Development Server (local, no Docker)"

    if [ ! -f "$ENV_FILE" ]; then
        setup_env_file
    fi

    # Source .env for local development
    set -a
    # shellcheck disable=SC1090
    source "$ENV_FILE" 2>/dev/null || true
    set +a

    # Override paths for local development
    export DATABASE_URL="sqlite://data/rustassistant.db"
    export CACHE_DB_PATH="data/rustassistant_cache.db"
    export REPOS_DIR="data/repos"
    export HOST="${HOST:-127.0.0.1}"
    export PORT="${PORT:-3000}"

    # Ensure local data directories exist
    mkdir -p data/repos

    print_info "Starting server on http://${HOST}:${PORT}"
    print_info "Press Ctrl+C to stop"
    echo ""

    cargo run --bin rustassistant-server
}

cmd_watch() {
    print_header "Starting Development Server with Auto-Reload"

    if ! command -v cargo-watch &> /dev/null; then
        print_warning "cargo-watch not installed. Installing..."
        cargo install cargo-watch
    fi

    if [ ! -f "$ENV_FILE" ]; then
        setup_env_file
    fi

    set -a
    # shellcheck disable=SC1090
    source "$ENV_FILE" 2>/dev/null || true
    set +a

    export DATABASE_URL="sqlite://data/rustassistant.db"
    export CACHE_DB_PATH="data/rustassistant_cache.db"
    export REPOS_DIR="data/repos"
    export HOST="${HOST:-127.0.0.1}"
    export PORT="${PORT:-3000}"

    mkdir -p data/repos

    print_info "Watching for changes... (http://${HOST}:${PORT})"
    print_info "Press Ctrl+C to stop"
    echo ""

    cargo watch -x 'run --bin rustassistant-server' \
        -w src/ -w templates/ -w static/ -w migrations/ -w Cargo.toml
}

# ============================================================================
# Usage
# ============================================================================

show_usage() {
    cat << 'EOF'
RustAssistant - Project Management Script

Usage: ./run.sh [mode] <command> [options]

Docker Commands:
  up [services]         Start services (build from source)
  prod up [services]    Start services (pull pre-built image)
  advanced up           Start advanced stack (Postgres, Grafana, etc.)
  down [-v]             Stop services (-v removes volumes)
  restart [services]    Restart services
  logs [services]       Tail service logs
  status                Show container status
  build [services]      Build Docker images
  shell [service]       Open shell in container (default: rustassistant)
  health                Check health of all services

Development Commands:
  dev                   Run server locally (no Docker, cargo run)
  watch                 Run with auto-reload (cargo watch)
  check                 Run cargo check
  fmt                   Format code (cargo fmt)
  clippy                Run clippy lints
  build-local           Build release binary locally

Testing & CI:
  test [args]           Run tests (cargo test)
  ci                    Full CI pipeline: fmt + clippy + test

Database Commands:
  db backup [path]      Backup SQLite database to ./backups/
  db restore <file>     Restore database from backup
  db shell              Open SQLite CLI on the database
  db size               Show database and repo sizes

Maintenance:
  diagnose              Show system diagnostics (ports, volumes, config)
  clean                 Remove stopped containers and dangling images
  force-clean           DANGER: Remove ALL data, volumes, and containers

Examples:
  ./run.sh up                    # Start dev mode (builds from source)
  ./run.sh prod up               # Start prod mode (pulls latest image)
  ./run.sh logs rustassistant    # Tail server logs
  ./run.sh logs redis            # Tail Redis logs
  ./run.sh health                # Check all services
  ./run.sh dev                   # Run locally without Docker
  ./run.sh ci                    # Run full CI before pushing
  ./run.sh db backup             # Backup the database
  ./run.sh db shell              # Open SQLite CLI
  ./run.sh down -v               # Stop and remove all volumes
  ./run.sh diagnose              # Debug port/volume/config issues
  ./run.sh force-clean           # Nuclear option: wipe everything

Services:
  rustassistant    Main server (Web UI + API + Scanner)
  redis            LLM response cache (Redis 7 Alpine)

Access URLs (default port 3000):
  Dashboard:       http://localhost:3000/
  Repos:           http://localhost:3000/repos
  Scanner:         http://localhost:3000/scanner
  Scan Progress:   http://localhost:3000/scan/dashboard
  DB Explorer:     http://localhost:3000/db
  Cache Viewer:    http://localhost:3000/cache
  Queue:           http://localhost:3000/queue
  API Health:      http://localhost:3000/health

Configuration:
  .env             Environment variables (auto-generated from .env.example)
  XAI_API_KEY      Required — Grok API key for LLM analysis
  GITHUB_TOKEN     Optional — for cloning private repositories
  PORT             Server port (default: 3000)
  AUTO_SCAN_*      Scanner settings (interval, budget, concurrency)

Data:
  Docker volumes store all persistent data:
    rustassistant_data        SQLite databases
    rustassistant_repos_data  Cloned repositories
    rustassistant_redis_data  Redis cache

  Backup regularly with: ./run.sh db backup
EOF
}

# ============================================================================
# Main Dispatcher
# ============================================================================

main() {
    # No arguments — show usage
    if [ $# -lt 1 ]; then
        show_usage
        exit 0
    fi

    # Parse optional mode prefix: prod | advanced
    local mode="dev"
    local command="$1"

    if [ "$1" = "prod" ]; then
        mode="prod"
        shift
        command="${1:-up}"
    elif [ "$1" = "advanced" ]; then
        mode="advanced"
        shift
        command="${1:-up}"
    fi

    shift 2>/dev/null || true

    # Dispatch command
    case "$command" in
        # Docker commands
        up|start)
            cmd_up "$mode" "$@"
            ;;
        down|stop)
            cmd_down "$mode" "$@"
            ;;
        restart)
            cmd_restart "$mode" "$@"
            ;;
        logs|log)
            cmd_logs "$mode" "$@"
            ;;
        status|ps)
            cmd_status "$mode"
            ;;
        build)
            cmd_build "$mode" "$@"
            ;;
        shell|exec)
            cmd_shell "$@"
            ;;
        health)
            cmd_health
            ;;

        # Development commands
        dev|run)
            cmd_dev
            ;;
        watch)
            cmd_watch
            ;;
        check)
            cmd_check
            ;;
        fmt|format)
            cmd_fmt
            ;;
        clippy|lint)
            cmd_clippy
            ;;
        build-local|build-release)
            cmd_build_local
            ;;

        # Testing & CI
        test|tests)
            cmd_test "$@"
            ;;
        ci|ci-rust)
            cmd_ci
            ;;

        # Database
        db|database)
            cmd_db "$@"
            ;;

        # Maintenance
        diagnose|diag|debug)
            cmd_diagnose
            ;;
        clean|cleanup)
            cmd_clean
            ;;
        force-clean|nuke)
            cmd_force_clean
            ;;
        preflight)
            preflight_check
            ;;
        generate-env|env)
            setup_env_file
            ;;

        # Help
        help|--help|-h)
            show_usage
            ;;

        *)
            print_error "Unknown command: $command"
            echo ""
            echo "Run './run.sh help' for usage information."
            exit 1
            ;;
    esac
}

main "$@"
