#!/usr/bin/env bash
# =============================================================================
# RustAssistant Docker Build Script
# =============================================================================
# Builds the RustAssistant API server Docker image.
#
# Usage:
#   ./scripts/build-docker.sh [OPTIONS]
#
# Options:
#   --tag <tag>              Tag for the image (default: latest)
#   --push                   Push to Docker Hub after building
#   --platform <platforms>   Target platforms (default: linux/amd64)
#   --registry <registry>    Docker registry (default: nuniesmith)
#   --no-cache               Build without Docker layer cache
#   --load                   Load image into local docker (for single-platform)
#   --openclaw               Also build the OpenClaw custom image
#   --help                   Show this help message
#
# Examples:
#   ./scripts/build-docker.sh
#   ./scripts/build-docker.sh --tag v1.0.0
#   ./scripts/build-docker.sh --push --tag latest
#   ./scripts/build-docker.sh --platform linux/amd64,linux/arm64 --push
#   ./scripts/build-docker.sh --openclaw
# =============================================================================

set -euo pipefail

# Default values
TAG="latest"
PUSH=false
PLATFORM="linux/amd64"
REGISTRY="nuniesmith"
REPO_NAME="rustassistant"
NO_CACHE=false
LOAD=false
BUILD_OPENCLAW=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()    { printf "${BLUE}ℹ${NC} %s\n" "$*"; }
ok()      { printf "${GREEN}✓${NC} %s\n" "$*"; }
err()     { printf "${RED}✗${NC} %s\n" "$*" >&2; }
warn()    { printf "${YELLOW}⚠${NC} %s\n" "$*"; }

show_help() {
    sed -n '2,/^# ====/{ /^# ====/d; s/^# \{0,1\}//; p; }' "$0"
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --tag)       TAG="$2";       shift 2 ;;
        --push)      PUSH=true;      shift ;;
        --platform)  PLATFORM="$2";  shift 2 ;;
        --registry)  REGISTRY="$2";  shift 2 ;;
        --no-cache)  NO_CACHE=true;  shift ;;
        --load)      LOAD=true;      shift ;;
        --openclaw)  BUILD_OPENCLAW=true; shift ;;
        --help|-h)   show_help ;;
        *)
            err "Unknown option: $1"
            echo ""
            show_help
            ;;
    esac
done

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

IMAGE_NAME="${REGISTRY}/${REPO_NAME}:${TAG}"

# Print build configuration
echo ""
printf "${BOLD}RustAssistant Docker Build${NC}\n"
echo "=========================================="
echo "Image:     $IMAGE_NAME"
echo "Platform:  $PLATFORM"
echo "Push:      $PUSH"
echo "No-cache:  $NO_CACHE"
echo "Load:      $LOAD"
echo "OpenClaw:  $BUILD_OPENCLAW"
echo "=========================================="
echo ""

# ── Pre-flight checks ───────────────────────────────────────────────────────
DOCKERFILE="docker/rustassistant/Dockerfile"

if [[ ! -f "$DOCKERFILE" ]]; then
    err "Dockerfile not found at $DOCKERFILE"
    err "Are you running from the project root?"
    exit 1
fi

if [[ ! -f "Cargo.toml" ]]; then
    err "Cargo.toml not found — run this script from the project root"
    exit 1
fi

# Ensure .sqlx directory exists (even if empty — no compile-time query macros)
mkdir -p .sqlx

# ── Build RustAssistant image ────────────────────────────────────────────────
info "Building RustAssistant API server image..."

BUILD_ARGS=(
    --file "$DOCKERFILE"
    --platform "$PLATFORM"
    --tag "$IMAGE_NAME"
)

# Also tag as :latest if building a versioned tag
if [[ "$TAG" != "latest" ]]; then
    BUILD_ARGS+=(--tag "${REGISTRY}/${REPO_NAME}:latest")
fi

if [[ "$NO_CACHE" == true ]]; then
    BUILD_ARGS+=(--no-cache)
fi

if [[ "$PUSH" == true ]]; then
    BUILD_ARGS+=(--push)
fi

if [[ "$LOAD" == true ]]; then
    BUILD_ARGS+=(--load)
fi

# For multi-platform builds, --push or --load is required (buildx constraint).
# For single-platform local builds, default to --load so the image is usable.
PLATFORM_COUNT=$(echo "$PLATFORM" | tr ',' '\n' | wc -l)
if [[ "$PUSH" == false ]] && [[ "$LOAD" == false ]]; then
    if [[ "$PLATFORM_COUNT" -gt 1 ]]; then
        warn "Multi-platform build without --push or --load."
        warn "The image will only exist in the build cache."
        warn "Add --push to push to registry, or use a single platform with --load."
    else
        info "Single platform detected — auto-adding --load"
        BUILD_ARGS+=(--load)
    fi
fi

if docker buildx build "${BUILD_ARGS[@]}" .; then
    ok "Built $IMAGE_NAME"
else
    err "Failed to build $IMAGE_NAME"
    exit 1
fi

# ── Optionally build OpenClaw custom image ───────────────────────────────────
if [[ "$BUILD_OPENCLAW" == true ]]; then
    echo ""
    info "Building OpenClaw custom image..."

    OPENCLAW_BUILD_SCRIPT="docker/openclaw/build.sh"
    if [[ -x "$OPENCLAW_BUILD_SCRIPT" ]]; then
        "$OPENCLAW_BUILD_SCRIPT" --layer-only
    else
        err "OpenClaw build script not found or not executable at $OPENCLAW_BUILD_SCRIPT"
        err "Run: chmod +x $OPENCLAW_BUILD_SCRIPT"
        exit 1
    fi
fi

# ── Summary ──────────────────────────────────────────────────────────────────
echo ""
printf "${BOLD}${GREEN}Build complete!${NC}\n"
echo ""

info "Built images:"
docker images --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}" \
    | grep "$REPO_NAME" | head -10 || true
echo ""

if [[ "$PUSH" == false ]]; then
    warn "Image was built locally but not pushed to registry."
    info "To push: $0 --tag $TAG --push"
fi

info "To run the stack:"
echo "  docker compose up -d"
echo ""
info "To verify health:"
echo "  curl -sf http://\$(hostname -I | awk '{print \$1}'):3500/health"
echo ""
