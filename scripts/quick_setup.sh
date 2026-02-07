#!/bin/bash
# Quick Setup Script for RustAssistant Repository Monitoring
# One-command setup for your priority repositories

set -e

# Configuration
DB_PATH="${DB_PATH:-./data/rustassistant.db}"
GITHUB_BASE="${GITHUB_BASE:-/home/jordan/github}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BOLD}${CYAN}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                                                            ║"
echo "║     RustAssistant Repository Monitoring Setup              ║"
echo "║                                                            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    echo -e "${RED}✗ Database not found at: $DB_PATH${NC}"
    echo -e "${YELLOW}  Please ensure RustAssistant is initialized first${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Database found${NC}"
echo ""

# Function to add repository
add_repo() {
    local path="$1"
    local name="$2"
    local interval="${3:-60}"

    # Generate UUID (works on Linux)
    local id=$(cat /proc/sys/kernel/random/uuid 2>/dev/null || echo "repo-$(date +%s)-$RANDOM")
    local now=$(date +%s)

    # Check if already exists
    local exists=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM repositories WHERE path = '$path';")

    if [ "$exists" -gt 0 ]; then
        echo -e "${YELLOW}⚠ Repository already exists: ${BOLD}$name${NC}"
        # Update instead
        sqlite3 "$DB_PATH" <<EOF
UPDATE repositories
SET name = '$name',
    status = 'active',
    updated_at = $now
WHERE path = '$path';
EOF
    else
        # Insert new
        sqlite3 "$DB_PATH" <<EOF
INSERT INTO repositories
    (id, path, name, status, created_at, updated_at, auto_scan_enabled, scan_interval_minutes)
VALUES
    ('$id', '$path', '$name', 'active', $now, $now, 0, $interval);
EOF
        echo -e "${GREEN}✓ Added repository: ${BOLD}$name${NC}"
    fi
}

# Function to enable scanning
enable_scan() {
    local name="$1"
    local interval="${2:-60}"
    local now=$(date +%s)

    sqlite3 "$DB_PATH" <<EOF
UPDATE repositories
SET auto_scan_enabled = 1,
    scan_interval_minutes = $interval,
    updated_at = $now
WHERE name = '$name';
EOF

    echo -e "${CYAN}  → Auto-scan enabled (${interval}m interval)${NC}"
}

# Cleanup invalid repos first
echo -e "${BOLD}Step 1: Cleaning up invalid repositories...${NC}"
cleaned=$(sqlite3 "$DB_PATH" "DELETE FROM repositories WHERE path LIKE 'http%'; SELECT changes();")
if [ "$cleaned" -gt 0 ]; then
    echo -e "${GREEN}✓ Removed $cleaned invalid repositories${NC}"
else
    echo -e "${GREEN}✓ No cleanup needed${NC}"
fi
echo ""

# Add priority repositories
echo -e "${BOLD}Step 2: Adding priority repositories...${NC}"
echo ""

# rustscape - Runescape project (personal)
if [ -d "$GITHUB_BASE/rustscape" ]; then
    echo -e "${BLUE}Adding: rustscape (RuneScape project)${NC}"
    add_repo "$GITHUB_BASE/rustscape" "rustscape" 30
    enable_scan "rustscape" 30
else
    echo -e "${YELLOW}⚠ Directory not found: $GITHUB_BASE/rustscape${NC}"
fi

# actions - Shared actions
if [ -d "$GITHUB_BASE/actions" ]; then
    echo -e "${BLUE}Adding: actions (Shared GitHub Actions)${NC}"
    add_repo "$GITHUB_BASE/actions" "actions" 60
    enable_scan "actions" 60
else
    echo -e "${YELLOW}⚠ Directory not found: $GITHUB_BASE/actions${NC}"
fi

# scripts - Shared scripts
if [ -d "$GITHUB_BASE/scripts" ]; then
    echo -e "${BLUE}Adding: scripts (Shared scripts)${NC}"
    add_repo "$GITHUB_BASE/scripts" "scripts" 60
    enable_scan "scripts" 60
else
    echo -e "${YELLOW}⚠ Directory not found: $GITHUB_BASE/scripts${NC}"
fi

# servers_sullivan - Sullivan server
if [ -d "$GITHUB_BASE/servers_sullivan" ]; then
    echo -e "${BLUE}Adding: servers_sullivan (Sullivan home server)${NC}"
    add_repo "$GITHUB_BASE/servers_sullivan" "servers_sullivan" 120
    enable_scan "servers_sullivan" 120
else
    echo -e "${YELLOW}⚠ Directory not found: $GITHUB_BASE/servers_sullivan${NC}"
fi

# servers_freddy - Freddy server
if [ -d "$GITHUB_BASE/servers_freddy" ]; then
    echo -e "${BLUE}Adding: servers_freddy (Freddy home server)${NC}"
    add_repo "$GITHUB_BASE/servers_freddy" "servers_freddy" 120
    enable_scan "servers_freddy" 120
else
    echo -e "${YELLOW}⚠ Directory not found: $GITHUB_BASE/servers_freddy${NC}"
fi

# Also add rustassistant itself (more frequent scanning)
if [ -d "$GITHUB_BASE/rustassistant" ]; then
    echo -e "${BLUE}Adding: rustassistant (this project)${NC}"
    add_repo "$GITHUB_BASE/rustassistant" "rustassistant" 15
    enable_scan "rustassistant" 15
fi

# And fks if it exists
if [ -d "$GITHUB_BASE/fks" ]; then
    echo -e "${BLUE}Adding: fks${NC}"
    add_repo "$GITHUB_BASE/fks" "fks" 30
    enable_scan "fks" 30
fi

echo ""
echo -e "${BOLD}Step 3: Summary${NC}"
echo ""

# Show statistics
sqlite3 "$DB_PATH" <<EOF
.mode column
.headers on
SELECT
    COUNT(*) as total_repos,
    SUM(CASE WHEN auto_scan_enabled = 1 THEN 1 ELSE 0 END) as scanning,
    SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) as active
FROM repositories
WHERE path LIKE '/home/%';
EOF

echo ""
echo -e "${BOLD}Repositories with auto-scan enabled:${NC}"
sqlite3 "$DB_PATH" <<EOF
.mode column
.headers on
SELECT
    name,
    scan_interval_minutes as interval_mins,
    CASE WHEN auto_scan_enabled = 1 THEN '✓ Enabled' ELSE '✗ Disabled' END as status
FROM repositories
WHERE auto_scan_enabled = 1 AND (path LIKE '/home/%' OR path LIKE '~/%')
ORDER BY scan_interval_minutes, name;
EOF

echo ""
echo -e "${GREEN}${BOLD}✓ Setup complete!${NC}"
echo ""
echo -e "${CYAN}Next steps:${NC}"
echo -e "  1. ${BOLD}Start/restart RustAssistant:${NC}"
echo -e "     ${YELLOW}docker compose restart rustassistant${NC}"
echo ""
echo -e "  2. ${BOLD}Open Web UI:${NC}"
echo -e "     ${YELLOW}http://localhost:3000${NC}"
echo ""
echo -e "  3. ${BOLD}View repositories:${NC}"
echo -e "     ${YELLOW}http://localhost:3000/repos${NC}"
echo ""
echo -e "  4. ${BOLD}Monitor the queue:${NC}"
echo -e "     ${YELLOW}http://localhost:3000/queue${NC}"
echo ""
echo -e "  5. ${BOLD}Make a change in any monitored repo to test scanning${NC}"
echo ""
echo -e "${CYAN}Advanced management:${NC}"
echo -e "  • Run interactive menu:  ${YELLOW}./scripts/manage_repos.sh interactive${NC}"
echo -e "  • Use Python manager:    ${YELLOW}python3 scripts/repo_manager.py${NC}"
echo ""
