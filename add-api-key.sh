#!/bin/bash
# ============================================================================
# RustAssistant - Add XAI API Key to .env
# ============================================================================
# Quick script to add or update XAI_API_KEY in .env file
#
# Usage:
#   ./add-api-key.sh YOUR_API_KEY_HERE
#   or
#   ./add-api-key.sh    (will prompt for key)
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
echo "║         RustAssistant - Add XAI API Key                       ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if we're in the right directory
if [ ! -f "docker-compose.prod.yml" ]; then
    echo -e "${RED}Error: docker-compose.prod.yml not found${NC}"
    echo "Please run this script from the rustassistant directory"
    echo ""
    echo "Example:"
    echo "  cd ~/rustassistant"
    echo "  ./add-api-key.sh"
    exit 1
fi

# Get API key from argument or prompt
if [ -n "$1" ]; then
    XAI_API_KEY="$1"
else
    echo -e "${BLUE}Enter your XAI API Key:${NC}"
    read -r XAI_API_KEY
    echo ""
fi

# Validate API key is not empty
if [ -z "$XAI_API_KEY" ]; then
    echo -e "${RED}Error: API key cannot be empty${NC}"
    exit 1
fi

# Check if .env exists
if [ ! -f ".env" ]; then
    echo -e "${YELLOW}!${NC} .env file not found, creating..."
    cat > .env <<EOF
# RustAssistant Environment Configuration
RUST_LOG=info
XAI_API_KEY=${XAI_API_KEY}
EOF
    chmod 600 .env
    echo -e "${GREEN}✓${NC} Created .env file with XAI_API_KEY"
else
    # Check if XAI_API_KEY already exists in .env
    if grep -q "^XAI_API_KEY=" .env; then
        echo -e "${BLUE}Updating existing XAI_API_KEY...${NC}"
        # Create backup
        cp .env .env.backup
        # Update the key
        sed -i "s|^XAI_API_KEY=.*|XAI_API_KEY=${XAI_API_KEY}|" .env
        echo -e "${GREEN}✓${NC} Updated XAI_API_KEY in .env"
        echo -e "${GREEN}✓${NC} Backup saved to .env.backup"
    else
        echo -e "${BLUE}Adding XAI_API_KEY to existing .env...${NC}"
        echo "" >> .env
        echo "# XAI API Configuration" >> .env
        echo "XAI_API_KEY=${XAI_API_KEY}" >> .env
        echo -e "${GREEN}✓${NC} Added XAI_API_KEY to .env"
    fi
fi

# Ensure proper permissions
chmod 600 .env
echo -e "${GREEN}✓${NC} Set .env permissions to 600"
echo ""

# Ask if user wants to restart containers
echo -e "${YELLOW}?${NC} Do you want to restart the containers to apply changes? (y/n)"
read -r RESTART

if [ "$RESTART" = "y" ] || [ "$RESTART" = "Y" ]; then
    echo ""
    echo -e "${BLUE}Restarting containers...${NC}"
    docker compose -f docker-compose.prod.yml restart rustassistant-web
    echo ""
    echo -e "${GREEN}✓${NC} Container restarted"
    echo ""
    echo "Waiting 5 seconds for container to start..."
    sleep 5
    echo ""
    echo "Container status:"
    docker ps --filter name=rustassistant-web --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    echo ""
    echo "Recent logs:"
    echo "─────────────────────────────────────────────────────────────────"
    docker logs rustassistant-web --tail=15
    echo "─────────────────────────────────────────────────────────────────"
    echo ""
    echo -e "${GREEN}✅ Done! XAI_API_KEY is now configured.${NC}"
    echo ""
    echo "Test the API:"
    echo "  curl http://localhost:3001/health"
else
    echo ""
    echo -e "${YELLOW}!${NC} Containers not restarted."
    echo ""
    echo "To apply changes, restart manually:"
    echo "  docker compose -f docker-compose.prod.yml restart rustassistant-web"
fi

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  ✅ XAI_API_KEY Configuration Complete                        ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Show reminder
echo -e "${BLUE}Tip:${NC} To view current environment variables:"
echo "  cat .env | grep XAI_API_KEY"
echo ""
