#!/bin/bash
# RustAssistant Dashboard - Quick Status Overview
# Shows repository status, queue, and system health at a glance

set -e

# Configuration
DB_PATH="${DB_PATH:-./data/rustassistant.db}"
API_URL="${API_URL:-http://localhost:3000}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# Unicode symbols
CHECK="✓"
CROSS="✗"
ARROW="→"
STAR="★"
DOT="•"

# Clear screen
clear

# Header
echo -e "${BOLD}${CYAN}"
echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║                                                                    ║"
echo "║                   RustAssistant Dashboard                          ║"
echo "║                                                                    ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo -e "${DIM}Last updated: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
echo ""

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    echo -e "${RED}${CROSS} Database not found at: $DB_PATH${NC}"
    exit 1
fi

# Check if service is running
echo -e "${BOLD}${BLUE}System Status${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

SERVICE_STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/health" 2>/dev/null || echo "000")

if [ "$SERVICE_STATUS" = "200" ]; then
    echo -e "${GREEN}${CHECK} RustAssistant Server:${NC} Running"
    echo -e "  ${ARROW} Web UI:   ${CYAN}$API_URL${NC}"
    echo -e "  ${ARROW} API:      ${CYAN}$API_URL/api${NC}"
else
    echo -e "${RED}${CROSS} RustAssistant Server:${NC} Not responding"
    echo -e "  ${YELLOW}${ARROW} Try: docker compose up -d${NC}"
fi

echo -e "  ${ARROW} Database: ${GREEN}$DB_PATH${NC}"
echo ""

# Repository Statistics
echo -e "${BOLD}${BLUE}Repository Overview${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

STATS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) as total, SUM(CASE WHEN auto_scan_enabled = 1 THEN 1 ELSE 0 END) as scanning, SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) as active FROM repositories WHERE path LIKE '/home/%' OR path LIKE '~/%';")

TOTAL=$(echo "$STATS" | cut -d'|' -f1)
SCANNING=$(echo "$STATS" | cut -d'|' -f2)
ACTIVE=$(echo "$STATS" | cut -d'|' -f3)

echo -e "  Total repositories:     ${BOLD}$TOTAL${NC}"
echo -e "  Active:                 ${GREEN}$ACTIVE${NC}"
echo -e "  Auto-scanning enabled:  ${CYAN}$SCANNING${NC}"
echo ""

# Active Scans
if [ "$SCANNING" -gt 0 ]; then
    echo -e "${BOLD}${BLUE}Active Scans${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    sqlite3 "$DB_PATH" "SELECT name || '|' || scan_interval_minutes FROM repositories WHERE auto_scan_enabled = 1 ORDER BY scan_interval_minutes, name;" | while IFS='|' read -r name interval; do
        # Color code by interval
        if [ "$interval" -le 30 ]; then
            COLOR="$GREEN"
        elif [ "$interval" -le 60 ]; then
            COLOR="$CYAN"
        else
            COLOR="$YELLOW"
        fi
        echo -e "  ${DOT} ${BOLD}${name}${NC} ${DIM}(${COLOR}${interval}m${NC}${DIM})${NC}"
    done
    echo ""
fi

# Task Queue
echo -e "${BOLD}${BLUE}Task Queue${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

QUEUE_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM queue;" 2>/dev/null || echo "0")

if [ "$QUEUE_COUNT" -eq 0 ]; then
    echo -e "  ${DIM}${CROSS} Queue is empty${NC}"
    echo ""
else
    echo -e "  ${GREEN}${QUEUE_COUNT} tasks in queue${NC}"
    echo ""

    # Show top 5 tasks
    echo -e "${BOLD}  Top Priority Tasks:${NC}"

    sqlite3 "$DB_PATH" "SELECT priority || '|' || title || '|' || stage FROM queue ORDER BY priority, created_at LIMIT 5;" | while IFS='|' read -r priority title stage; do
        case $priority in
            1) PCOLOR="$RED"; PLABEL="CRITICAL" ;;
            2) PCOLOR="$YELLOW"; PLABEL="HIGH    " ;;
            3) PCOLOR="$BLUE"; PLABEL="MEDIUM  " ;;
            4) PCOLOR="$NC"; PLABEL="LOW     " ;;
            *) PCOLOR="$NC"; PLABEL="UNKNOWN " ;;
        esac

        # Truncate title if too long
        if [ ${#title} -gt 50 ]; then
            title="${title:0:47}..."
        fi

        echo -e "  ${PCOLOR}[${PLABEL}]${NC} ${title} ${DIM}(${stage})${NC}"
    done

    if [ "$QUEUE_COUNT" -gt 5 ]; then
        echo -e "  ${DIM}... and $((QUEUE_COUNT - 5)) more${NC}"
    fi
    echo ""
fi

# Recent Activity
echo -e "${BOLD}${BLUE}Recent Activity${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

RECENT_SCANS=$(sqlite3 "$DB_PATH" "SELECT name || '|' || last_scan_check FROM repositories WHERE last_scan_check IS NOT NULL ORDER BY last_scan_check DESC LIMIT 3;")

if [ -z "$RECENT_SCANS" ]; then
    echo -e "  ${DIM}${CROSS} No scans recorded yet${NC}"
else
    echo "$RECENT_SCANS" | while IFS='|' read -r name timestamp; do
        # Convert timestamp to human readable (if within last hour)
        NOW=$(date +%s)
        DIFF=$((NOW - timestamp))

        if [ $DIFF -lt 60 ]; then
            TIME_AGO="${DIFF}s ago"
        elif [ $DIFF -lt 3600 ]; then
            TIME_AGO="$((DIFF / 60))m ago"
        elif [ $DIFF -lt 86400 ]; then
            TIME_AGO="$((DIFF / 3600))h ago"
        else
            TIME_AGO="$((DIFF / 86400))d ago"
        fi

        echo -e "  ${DOT} ${BOLD}${name}${NC} ${DIM}scanned ${TIME_AGO}${NC}"
    done
fi
echo ""

# Quick Actions
echo -e "${BOLD}${BLUE}Quick Actions${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "  ${CYAN}1.${NC} View repositories:       ${YELLOW}./scripts/manage_repos.sh list${NC}"
echo -e "  ${CYAN}2.${NC} Interactive menu:        ${YELLOW}./scripts/manage_repos.sh interactive${NC}"
echo -e "  ${CYAN}3.${NC} View queue:              ${YELLOW}python3 scripts/repo_manager.py queue${NC}"
echo -e "  ${CYAN}4.${NC} Generate task summary:   ${YELLOW}python3 scripts/repo_manager.py summary${NC}"
echo -e "  ${CYAN}5.${NC} Open Web UI:             ${YELLOW}$API_URL${NC}"
echo ""

# Footer
echo -e "${BOLD}${BLUE}Links${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "  ${STAR} Dashboard:    ${CYAN}$API_URL${NC}"
echo -e "  ${STAR} Repositories: ${CYAN}$API_URL/repos${NC}"
echo -e "  ${STAR} Queue:        ${CYAN}$API_URL/queue${NC}"
echo ""

# Tips
TIPS=(
    "Make a change in a monitored repo to generate tasks"
    "Use 'Copy for IDE' in Web UI to integrate with AI assistants"
    "Adjust scan intervals based on development activity"
    "Run 'python3 scripts/repo_manager.py summary' for AI context"
    "Check queue regularly - don't let tasks pile up!"
    "Use interactive mode for bulk operations"
    "Shorter scan intervals = faster issue detection"
)

# Pick random tip
RANDOM_TIP=${TIPS[$RANDOM % ${#TIPS[@]}]}
echo -e "${DIM}${ARROW} Tip: ${RANDOM_TIP}${NC}"
echo ""

# Watch mode
if [ "$1" = "-w" ] || [ "$1" = "--watch" ]; then
    echo -e "${DIM}Watching... (Press Ctrl+C to exit)${NC}"
    sleep 5
    exec "$0" -w
fi
