#!/bin/bash
# ============================================================================
# RustAssistant - Quick Fix for Database Permissions
# ============================================================================
# Fixes the "unable to open database file" error on Raspberry Pi
#
# Usage:
#   chmod +x fix-permissions.sh
#   ./fix-permissions.sh
# ============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║         RustAssistant - Database Permissions Fix              ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if we're in the right directory
if [ ! -f "docker-compose.prod.yml" ]; then
    echo -e "${RED}Error: docker-compose.prod.yml not found${NC}"
    echo "Please run this script from the rustassistant directory"
    echo ""
    echo "Example:"
    echo "  cd ~/rustassistant"
    echo "  ./fix-permissions.sh"
    exit 1
fi

echo -e "${BLUE}[1/5]${NC} Stopping containers..."
docker compose -f docker-compose.prod.yml down 2>/dev/null || true
echo -e "${GREEN}✓${NC} Containers stopped"
echo ""

echo -e "${BLUE}[2/5]${NC} Creating required directories..."
mkdir -p data config
echo -e "${GREEN}✓${NC} Directories created"
echo ""

echo -e "${BLUE}[3/5]${NC} Setting permissions..."
# Get current user's UID and GID
USER_UID=$(id -u)
USER_GID=$(id -g)

# Set ownership
chown -R ${USER_UID}:${USER_GID} data config
chmod -R 755 data config

echo -e "${GREEN}✓${NC} Permissions set"
echo "  - Owner: ${USER_UID}:${USER_GID}"
echo "  - Permissions: 755"
echo ""

echo -e "${BLUE}[4/5]${NC} Checking .env file..."
if [ ! -f ".env" ]; then
    echo -e "${YELLOW}!${NC} .env file not found, creating default..."
    cat > .env <<EOF
# RustAssistant Environment Configuration
RUST_LOG=info
XAI_API_KEY=
EOF
    chmod 600 .env
    echo -e "${GREEN}✓${NC} Created default .env file"
    echo -e "${YELLOW}!${NC} IMPORTANT: Add your XAI_API_KEY to .env"
    echo -e "${YELLOW}!${NC}   Edit: nano .env"
    echo -e "${YELLOW}!${NC}   Or the CI/CD pipeline will set it automatically on next deployment"
else
    echo -e "${GREEN}✓${NC} .env file exists"

    # Check if XAI_API_KEY is set
    if grep -q "^XAI_API_KEY=.\+" .env; then
        echo -e "${GREEN}✓${NC} XAI_API_KEY is configured"
    else
        echo -e "${YELLOW}!${NC} XAI_API_KEY is empty or not set"
        echo -e "${YELLOW}!${NC}   Add it manually: nano .env"
        echo -e "${YELLOW}!${NC}   Or push changes to trigger CI/CD (will auto-set from GitHub secret)"
    fi
fi
echo ""

echo -e "${BLUE}[5/5]${NC} Starting containers..."
docker compose -f docker-compose.prod.yml up -d
echo -e "${GREEN}✓${NC} Containers started"
echo ""

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                    Fix Complete!                               ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Checking container status..."
echo ""
docker compose -f docker-compose.prod.yml ps
echo ""

echo "Waiting 5 seconds for containers to initialize..."
sleep 5
echo ""

echo "Recent logs from rustassistant-web:"
echo "─────────────────────────────────────────────────────────────────"
docker logs rustassistant-web --tail=10
echo "─────────────────────────────────────────────────────────────────"
echo ""

# Check if container is running
if docker ps | grep -q rustassistant-web; then
    STATUS=$(docker inspect rustassistant-web --format='{{.State.Status}}')
    if [ "$STATUS" = "running" ]; then
        echo -e "${GREEN}✅ Container is running successfully!${NC}"
        echo ""
        echo "Next steps:"
        echo "  - Check logs: docker logs rustassistant-web -f"
        echo "  - Access UI: http://localhost:3000"
        echo "  - Check health: curl http://localhost:3000/"
        echo ""
        echo "Note: If XAI_API_KEY is not set, LLM features won't work."
        echo "      Edit .env or push to trigger CI/CD deployment."
    else
        echo -e "${YELLOW}⚠️  Container status: $STATUS${NC}"
        echo "Check logs with: docker logs rustassistant-web"
    fi
else
    echo -e "${RED}❌ Container is not running${NC}"
    echo "Check logs with: docker logs rustassistant-web"
fi
echo ""
