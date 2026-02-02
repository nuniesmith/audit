#!/bin/bash
# ============================================================================
# Rustassistant Run Script
# ============================================================================
# Handles environment setup and service management
#
# Usage:
#   ./run.sh                    # Interactive mode (asks for missing values)
#   ./run.sh --non-interactive  # CI/CD mode (uses env vars or defaults)
#   ./run.sh build              # Build containers
#   ./run.sh up                 # Start services
#   ./run.sh down               # Stop services
#   ./run.sh logs               # View logs
#   ./run.sh clean              # Clean up containers and volumes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ENV_FILE=".env"
INTERACTIVE=true

# Parse arguments
for arg in "$@"; do
    case $arg in
        --non-interactive)
            INTERACTIVE=false
            shift
            ;;
        build|up|down|logs|clean|restart|status)
            COMMAND=$arg
            shift
            ;;
    esac
done

# ============================================================================
# Helper Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

generate_secret() {
    openssl rand -hex 32 2>/dev/null || cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 64 | head -n 1
}

# ============================================================================
# Environment Setup
# ============================================================================

setup_env() {
    log_info "Setting up environment..."

    # Check if .env exists
    if [ -f "$ENV_FILE" ]; then
        log_info "Found existing $ENV_FILE"
        source "$ENV_FILE"

        # Check if XAI_API_KEY is set
        if [ -z "$XAI_API_KEY" ]; then
            if [ "$INTERACTIVE" = true ]; then
                read -p "XAI API Key is missing. Enter your XAI API key: " XAI_API_KEY
                if [ -n "$XAI_API_KEY" ]; then
                    echo "XAI_API_KEY=$XAI_API_KEY" >> "$ENV_FILE"
                    log_success "Added XAI_API_KEY to $ENV_FILE"
                fi
            else
                log_warning "XAI_API_KEY not set. Server will start but API calls will fail."
            fi
        fi
    else
        log_info "Creating new $ENV_FILE..."

        # Get XAI API Key
        if [ "$INTERACTIVE" = true ]; then
            echo ""
            echo "╔════════════════════════════════════════════════════════════════╗"
            echo "║           Rustassistant Environment Configuration             ║"
            echo "╚════════════════════════════════════════════════════════════════╝"
            echo ""
            read -p "Enter your XAI API key (or press Enter to skip): " XAI_API_KEY
            echo ""
        else
            # In non-interactive mode, use environment variable or empty
            XAI_API_KEY="${XAI_API_KEY:-}"
        fi

        # Generate secrets
        log_info "Generating secure random secrets..."
        DB_ENCRYPTION_KEY=$(generate_secret)
        SESSION_SECRET=$(generate_secret)
        REDIS_PASSWORD=$(generate_secret)

        # Create .env file
        cat > "$ENV_FILE" << EOF
# ============================================================================
# Rustassistant Environment Configuration
# ============================================================================
# Generated on $(date)

# ----------------------------------------------------------------------------
# API Keys
# ----------------------------------------------------------------------------
XAI_API_KEY=${XAI_API_KEY}
XAI_BASE_URL=https://api.x.ai/v1

# ----------------------------------------------------------------------------
# Database
# ----------------------------------------------------------------------------
DATABASE_URL=sqlite:/home/jordan/github/rustassistant/data/rustassistant.db
DB_ENCRYPTION_KEY=${DB_ENCRYPTION_KEY}

# ----------------------------------------------------------------------------
# Server Configuration
# ----------------------------------------------------------------------------
HOST=127.0.0.1
PORT=3000
RUST_LOG=info,rustassistant=debug

# ----------------------------------------------------------------------------
# Security
# ----------------------------------------------------------------------------
SESSION_SECRET=${SESSION_SECRET}

# ----------------------------------------------------------------------------
# Redis Cache (Optional)
# ----------------------------------------------------------------------------
REDIS_URL=redis://:${REDIS_PASSWORD}@localhost:6379
REDIS_PASSWORD=${REDIS_PASSWORD}

# ----------------------------------------------------------------------------
# Docker Configuration
# ----------------------------------------------------------------------------
COMPOSE_PROJECT_NAME=rustassistant
DOCKER_BUILDKIT=1
EOF

        log_success "Created $ENV_FILE with secure random secrets"

        if [ "$INTERACTIVE" = true ]; then
            echo ""
            log_info "Environment file created at: $ENV_FILE"
            if [ -z "$XAI_API_KEY" ]; then
                log_warning "XAI_API_KEY is not set. Add it to $ENV_FILE before using LLM features."
            fi
            echo ""
        fi
    fi

    # Create data directory if it doesn't exist
    mkdir -p data
    log_success "Data directory ready"
}

# ============================================================================
# Docker Commands
# ============================================================================

docker_build() {
    log_info "Building Docker containers..."
    docker-compose build
    log_success "Build complete"
}

docker_up() {
    log_info "Starting services..."
    docker-compose up -d
    log_success "Services started"
    docker-compose ps
    echo ""
    log_info "API server available at: http://localhost:${PORT:-3000}"
    log_info "Health check: curl http://localhost:${PORT:-3000}/health"
}

docker_down() {
    log_info "Stopping services..."
    docker-compose down
    log_success "Services stopped"
}

docker_logs() {
    log_info "Showing logs (Ctrl+C to exit)..."
    docker-compose logs -f
}

docker_restart() {
    log_info "Restarting services..."
    docker-compose restart
    log_success "Services restarted"
}

docker_status() {
    log_info "Service status:"
    docker-compose ps
}

docker_clean() {
    log_warning "This will remove all containers, networks, and volumes."
    if [ "$INTERACTIVE" = true ]; then
        read -p "Are you sure? (yes/no): " confirm
        if [ "$confirm" != "yes" ]; then
            log_info "Cancelled"
            exit 0
        fi
    fi

    log_info "Cleaning up..."
    docker-compose down -v --remove-orphans
    log_success "Cleanup complete"
}

# ============================================================================
# CLI Usage
# ============================================================================

show_usage() {
    cat << EOF

╔════════════════════════════════════════════════════════════════╗
║                    Rustassistant Run Script                    ║
╚════════════════════════════════════════════════════════════════╝

Usage: ./run.sh [OPTIONS] [COMMAND]

OPTIONS:
    --non-interactive    Run in non-interactive mode (CI/CD)

COMMANDS:
    build       Build Docker containers
    up          Start services in detached mode
    down        Stop services
    logs        Show and follow service logs
    restart     Restart all services
    status      Show service status
    clean       Remove containers, networks, and volumes

EXAMPLES:
    # First time setup (interactive)
    ./run.sh

    # Start services
    ./run.sh up

    # CI/CD mode
    XAI_API_KEY=\${{ secrets.XAI_API_KEY }} ./run.sh --non-interactive up

    # View logs
    ./run.sh logs

    # Stop everything
    ./run.sh down

ENVIRONMENT VARIABLES (non-interactive mode):
    XAI_API_KEY         Your XAI API key (required for LLM features)
    PORT                Server port (default: 3000)
    RUST_LOG            Log level (default: info,rustassistant=debug)

EOF
}

# ============================================================================
# Main Execution
# ============================================================================

main() {
    echo ""
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                      Rustassistant                             ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""

    # Setup environment
    setup_env

    # Execute command
    case "${COMMAND:-up}" in
        build)
            docker_build
            ;;
        up)
            docker_build
            docker_up
            ;;
        down)
            docker_down
            ;;
        logs)
            docker_logs
            ;;
        restart)
            docker_restart
            ;;
        status)
            docker_status
            ;;
        clean)
            docker_clean
            ;;
        help|--help|-h)
            show_usage
            ;;
        *)
            # Default: start services
            docker_build
            docker_up
            ;;
    esac

    echo ""
}

# Run main function
main "$@"
