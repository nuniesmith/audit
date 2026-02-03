#!/bin/bash
# ============================================================================
# RustAssistant - Pre-Deployment Test Script
# ============================================================================
# Tests the deployment process locally on Raspberry Pi before CI/CD runs
#
# Usage:
#   chmod +x test-deployment.sh
#   ./test-deployment.sh
#
# This script simulates what the CI/CD pipeline will do during deployment
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0

# ============================================================================
# Helper Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
    ((TESTS_PASSED++))
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
    ((TESTS_FAILED++))
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_step() {
    echo -e "${CYAN}==>${NC} $1"
}

# ============================================================================
# Test Functions
# ============================================================================

test_docker_installed() {
    log_step "Testing Docker installation..."
    if command -v docker &> /dev/null; then
        log_success "Docker is installed: $(docker --version)"
    else
        log_error "Docker is not installed"
        return 1
    fi
}

test_docker_compose() {
    log_step "Testing Docker Compose..."
    if docker compose version &> /dev/null; then
        log_success "Docker Compose is installed: $(docker compose version --short)"
    else
        log_error "Docker Compose is not installed"
        return 1
    fi
}

test_docker_permissions() {
    log_step "Testing Docker permissions..."
    if docker ps &> /dev/null; then
        log_success "User has Docker permissions"
    else
        log_error "User cannot run Docker commands (try: sudo usermod -aG docker $USER)"
        return 1
    fi
}

test_disk_space() {
    log_step "Testing available disk space..."
    AVAILABLE=$(df -BG / | awk 'NR==2 {print $4}' | sed 's/G//')
    if [ "$AVAILABLE" -gt 10 ]; then
        log_success "Sufficient disk space: ${AVAILABLE}GB available"
    else
        log_error "Low disk space: ${AVAILABLE}GB available (need at least 10GB)"
        return 1
    fi
}

test_compose_file() {
    log_step "Testing docker-compose.prod.yml exists..."
    if [ -f "docker-compose.prod.yml" ]; then
        log_success "docker-compose.prod.yml found"
    else
        log_error "docker-compose.prod.yml not found"
        return 1
    fi
}

test_compose_syntax() {
    log_step "Testing docker-compose.prod.yml syntax..."
    if docker compose -f docker-compose.prod.yml config &> /dev/null; then
        log_success "docker-compose.prod.yml syntax is valid"
    else
        log_error "docker-compose.prod.yml has syntax errors"
        return 1
    fi
}

test_env_file() {
    log_step "Testing .env file..."
    if [ -f ".env" ]; then
        log_success ".env file exists"

        # Check for critical variables
        if grep -q "XAI_API_KEY" .env; then
            log_success "XAI_API_KEY is defined in .env"
        else
            log_warning "XAI_API_KEY not found in .env (LLM features will not work)"
        fi
    else
        log_warning ".env file not found (will be created during deployment)"
    fi
}

test_docker_hub_connectivity() {
    log_step "Testing Docker Hub connectivity..."
    if docker pull hello-world &> /dev/null; then
        log_success "Can pull images from Docker Hub"
        docker rmi hello-world &> /dev/null || true
    else
        log_error "Cannot connect to Docker Hub"
        return 1
    fi
}

test_image_pull() {
    log_step "Testing ARM64 image availability..."
    IMAGE="nuniesmith/rustassistant:latest"

    log_info "Attempting to pull $IMAGE..."
    if docker pull --platform linux/arm64 "$IMAGE" 2>&1 | tee /tmp/pull_test.log; then
        log_success "Successfully pulled $IMAGE"

        # Check architecture
        ARCH=$(docker inspect "$IMAGE" --format='{{.Architecture}}')
        if [ "$ARCH" = "arm64" ]; then
            log_success "Image is correct architecture: $ARCH"
        else
            log_warning "Image architecture is $ARCH (expected arm64)"
        fi
    else
        if grep -q "manifest unknown" /tmp/pull_test.log; then
            log_warning "Image not found on Docker Hub (will be built by CI/CD)"
        else
            log_error "Failed to pull image from Docker Hub"
            cat /tmp/pull_test.log
            return 1
        fi
    fi
    rm -f /tmp/pull_test.log
}

test_deployment_simulation() {
    log_step "Simulating deployment process..."

    # Stop existing services
    log_info "1. Stopping existing services..."
    if docker compose -f docker-compose.prod.yml down 2>/dev/null; then
        log_success "Stop command succeeded"
    else
        log_warning "Stop command had issues (may be normal if nothing running)"
    fi

    # Pull images
    log_info "2. Pulling images..."
    if docker compose -f docker-compose.prod.yml pull 2>&1 | tee /tmp/deploy_test.log; then
        log_success "Pull command succeeded"
    else
        if grep -q "manifest unknown" /tmp/deploy_test.log; then
            log_warning "Images not yet available (will be built by CI/CD first)"
        else
            log_error "Pull command failed"
            cat /tmp/deploy_test.log
            rm -f /tmp/deploy_test.log
            return 1
        fi
    fi
    rm -f /tmp/deploy_test.log

    # Try to start (without actually starting)
    log_info "3. Validating service configuration..."
    if docker compose -f docker-compose.prod.yml config &> /dev/null; then
        log_success "Service configuration is valid"
    else
        log_error "Service configuration has errors"
        return 1
    fi
}

test_architecture() {
    log_step "Testing system architecture..."
    ARCH=$(uname -m)
    if [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
        log_success "System architecture is ARM64: $ARCH"
    else
        log_warning "System architecture is $ARCH (expected aarch64 or arm64)"
    fi
}

test_run_script() {
    log_step "Testing run.sh script..."
    if [ -f "run.sh" ]; then
        log_success "run.sh found"

        if [ -x "run.sh" ]; then
            log_success "run.sh is executable"
        else
            log_warning "run.sh is not executable (fixing...)"
            chmod +x run.sh
            log_success "Made run.sh executable"
        fi
    else
        log_warning "run.sh not found"
    fi
}

# ============================================================================
# Main Execution
# ============================================================================

main() {
    echo ""
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║           RustAssistant Deployment Test Suite                 ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""

    log_info "Testing deployment prerequisites on Raspberry Pi..."
    echo ""

    # Run all tests
    test_architecture || true
    echo ""

    test_docker_installed || true
    test_docker_compose || true
    test_docker_permissions || true
    echo ""

    test_disk_space || true
    echo ""

    test_compose_file || true
    test_compose_syntax || true
    test_env_file || true
    test_run_script || true
    echo ""

    test_docker_hub_connectivity || true
    test_image_pull || true
    echo ""

    test_deployment_simulation || true
    echo ""

    # Summary
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                      Test Summary                              ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "${GREEN}✓ Tests Passed:${NC} $TESTS_PASSED"
    echo -e "${RED}✗ Tests Failed:${NC} $TESTS_FAILED"
    echo ""

    if [ "$TESTS_FAILED" -eq 0 ]; then
        echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║  ✅ All tests passed! Ready for deployment.                   ║${NC}"
        echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
        echo ""
        log_info "Next steps:"
        echo "  1. Commit and push your changes to trigger CI/CD"
        echo "  2. Monitor the GitHub Actions workflow"
        echo "  3. Check deployment status with: docker compose -f docker-compose.prod.yml ps"
        echo ""
        exit 0
    else
        echo -e "${RED}╔════════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${RED}║  ❌ Some tests failed. Fix issues before deploying.           ║${NC}"
        echo -e "${RED}╚════════════════════════════════════════════════════════════════╝${NC}"
        echo ""
        log_warning "Fix the failed tests above before proceeding with deployment"
        echo ""
        exit 1
    fi
}

# Run main function
main "$@"
