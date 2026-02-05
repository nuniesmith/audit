#!/usr/bin/env bash
# =============================================================================
# RustAssistant Docker Build Script
# =============================================================================
# Builds both API and Web UI Docker images with proper tagging
#
# Usage:
#   ./scripts/build-docker.sh [OPTIONS]
#
# Options:
#   --service <api|web|all>  Build specific service (default: all)
#   --tag <tag>              Tag for the image (default: latest)
#   --push                   Push to Docker Hub after building
#   --platform <platforms>   Target platforms (default: linux/amd64,linux/arm64)
#   --registry <registry>    Docker registry (default: nuniesmith)
#   --help                   Show this help message
#
# Examples:
#   ./scripts/build-docker.sh --service api
#   ./scripts/build-docker.sh --service web --tag v1.0.0
#   ./scripts/build-docker.sh --push --tag latest
#   ./scripts/build-docker.sh --platform linux/arm64 --service web
# =============================================================================

set -euo pipefail

# Default values
SERVICE="all"
TAG="latest"
PUSH=false
PLATFORM="linux/amd64,linux/arm64"
REGISTRY="nuniesmith"
REPO_NAME="rustassistant"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_info() {
    echo -e "${BLUE}ℹ ${1}${NC}"
}

print_success() {
    echo -e "${GREEN}✓ ${1}${NC}"
}

print_error() {
    echo -e "${RED}✗ ${1}${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ ${1}${NC}"
}

# Show help
show_help() {
    grep '^#' "$0" | grep -v '#!/usr/bin/env' | sed 's/^# //g' | sed 's/^#//g'
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --service)
            SERVICE="$2"
            shift 2
            ;;
        --tag)
            TAG="$2"
            shift 2
            ;;
        --push)
            PUSH=true
            shift
            ;;
        --platform)
            PLATFORM="$2"
            shift 2
            ;;
        --registry)
            REGISTRY="$2"
            shift 2
            ;;
        --help)
            show_help
            ;;
        *)
            print_error "Unknown option: $1"
            echo ""
            show_help
            ;;
    esac
done

# Validate service option
if [[ ! "$SERVICE" =~ ^(api|web|all)$ ]]; then
    print_error "Invalid service: $SERVICE (must be api, web, or all)"
    exit 1
fi

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# Print build configuration
echo ""
print_info "RustAssistant Docker Build Configuration"
echo "=========================================="
echo "Service:   $SERVICE"
echo "Tag:       $TAG"
echo "Platform:  $PLATFORM"
echo "Registry:  $REGISTRY/$REPO_NAME"
echo "Push:      $PUSH"
echo "=========================================="
echo ""

# Function to build a service
build_service() {
    local service_type=$1
    local service_port=$2
    local image_suffix=$3

    print_info "Building ${service_type} service..."

    local image_name="${REGISTRY}/${REPO_NAME}:${TAG}${image_suffix}"

    # Build the image
    if docker buildx build \
        --file docker/Dockerfile \
        --build-arg SERVICE_TYPE="$service_type" \
        --build-arg SERVICE_PORT="$service_port" \
        --platform "$PLATFORM" \
        --tag "$image_name" \
        ${PUSH:+--push} \
        .; then
        print_success "Built $image_name"
    else
        print_error "Failed to build $service_type"
        return 1
    fi

    # If not pushing with buildx, push separately
    if [[ "$PUSH" == true ]] && [[ "$PLATFORM" == "linux/amd64" ]]; then
        print_info "Pushing $image_name to registry..."
        if docker push "$image_name"; then
            print_success "Pushed $image_name"
        else
            print_error "Failed to push $image_name"
            return 1
        fi
    fi
}

# Build API service
if [[ "$SERVICE" == "api" ]] || [[ "$SERVICE" == "all" ]]; then
    build_service "api" "3000" "-api"
fi

# Build Web service
if [[ "$SERVICE" == "web" ]] || [[ "$SERVICE" == "all" ]]; then
    build_service "web" "3001" "-web"
fi

# If building all, also create a "latest" tag for web (default)
if [[ "$SERVICE" == "all" ]] && [[ "$TAG" != "latest" ]]; then
    print_info "Creating additional 'latest' tag for web service..."
    docker tag "${REGISTRY}/${REPO_NAME}:${TAG}-web" "${REGISTRY}/${REPO_NAME}:latest"

    if [[ "$PUSH" == true ]]; then
        print_info "Pushing latest tag..."
        docker push "${REGISTRY}/${REPO_NAME}:latest"
        print_success "Pushed ${REGISTRY}/${REPO_NAME}:latest"
    fi
fi

echo ""
print_success "Build complete!"
echo ""

# Show built images
print_info "Built images:"
docker images | grep "$REGISTRY/$REPO_NAME" | grep "$TAG" || true
echo ""

# Show next steps
if [[ "$PUSH" == false ]]; then
    print_warning "Images were built locally but not pushed to registry."
    print_info "To push images, run with --push flag"
fi

print_info "To run the services locally:"
echo "  docker compose up -d"
echo ""
print_info "To run in production:"
echo "  docker compose -f docker-compose.prod.yml up -d"
echo ""
