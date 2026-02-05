#!/bin/bash
# Repository Management Script for RustAssistant
# Helps with bulk adding, updating, and managing repositories

set -e

# Configuration
DB_PATH="${DB_PATH:-./data/rustassistant.db}"
GITHUB_BASE="${GITHUB_BASE:-/home/jordan/github}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
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

# Check if database exists
check_database() {
    if [ ! -f "$DB_PATH" ]; then
        log_error "Database not found at: $DB_PATH"
        log_info "Please ensure the RustAssistant database exists"
        exit 1
    fi
    log_success "Database found at: $DB_PATH"
}

# List all repositories
list_repos() {
    log_info "Listing all repositories..."
    echo ""
    sqlite3 "$DB_PATH" <<EOF
.headers on
.mode column
SELECT
    name,
    CASE WHEN auto_scan_enabled = 1 THEN '✓' ELSE '✗' END as scan,
    scan_interval_minutes as interval,
    status
FROM repositories
WHERE path LIKE '/home/%' OR path LIKE '~/%'
ORDER BY name;
EOF
}

# Add a repository
add_repo() {
    local path="$1"
    local name="$2"

    if [ -z "$path" ] || [ -z "$name" ]; then
        log_error "Usage: add_repo <path> <name>"
        return 1
    fi

    # Check if path exists
    if [ ! -d "$path" ]; then
        log_warning "Path does not exist: $path"
        read -p "Add anyway? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Skipping..."
            return 0
        fi
    fi

    local id=$(uuidgen 2>/dev/null || cat /proc/sys/kernel/random/uuid 2>/dev/null || echo "repo-$(date +%s)")
    local now=$(date +%s)

    sqlite3 "$DB_PATH" <<EOF
INSERT OR IGNORE INTO repositories
    (id, path, name, status, created_at, updated_at, auto_scan_enabled, scan_interval_minutes)
VALUES
    ('$id', '$path', '$name', 'active', $now, $now, 0, 60);
EOF

    if [ $? -eq 0 ]; then
        log_success "Added repository: $name at $path"
    else
        log_error "Failed to add repository (may already exist)"
    fi
}

# Enable auto-scan for a repository
enable_scan() {
    local name="$1"
    local interval="${2:-60}"

    if [ -z "$name" ]; then
        log_error "Usage: enable_scan <name> [interval_minutes]"
        return 1
    fi

    sqlite3 "$DB_PATH" <<EOF
UPDATE repositories
SET auto_scan_enabled = 1,
    scan_interval_minutes = $interval,
    updated_at = $(date +%s)
WHERE name = '$name';
EOF

    if [ $? -eq 0 ]; then
        log_success "Enabled auto-scan for: $name (interval: ${interval}m)"
    else
        log_error "Failed to enable auto-scan for: $name"
    fi
}

# Disable auto-scan for a repository
disable_scan() {
    local name="$1"

    if [ -z "$name" ]; then
        log_error "Usage: disable_scan <name>"
        return 1
    fi

    sqlite3 "$DB_PATH" <<EOF
UPDATE repositories
SET auto_scan_enabled = 0,
    updated_at = $(date +%s)
WHERE name = '$name';
EOF

    if [ $? -eq 0 ]; then
        log_success "Disabled auto-scan for: $name"
    else
        log_error "Failed to disable auto-scan for: $name"
    fi
}

# Bulk add repositories from GitHub directory
bulk_add_github_repos() {
    log_info "Scanning $GITHUB_BASE for repositories..."

    if [ ! -d "$GITHUB_BASE" ]; then
        log_error "GitHub base directory not found: $GITHUB_BASE"
        return 1
    fi

    local count=0
    for dir in "$GITHUB_BASE"/*; do
        if [ -d "$dir" ]; then
            local name=$(basename "$dir")
            local path="$dir"

            # Check if already exists
            local exists=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM repositories WHERE path = '$path';")

            if [ "$exists" -eq 0 ]; then
                log_info "Adding: $name"
                add_repo "$path" "$name"
                ((count++))
            else
                log_info "Already exists: $name"
            fi
        fi
    done

    log_success "Bulk add complete. Added $count new repositories."
}

# Enable scanning for specific repos
enable_priority_repos() {
    log_info "Enabling auto-scan for priority repositories..."

    # Priority repos with intervals (in minutes)
    declare -A PRIORITY_REPOS=(
        ["rustscape"]=30
        ["actions"]=60
        ["scripts"]=60
        ["servers_sullivan"]=120
        ["servers_freddy"]=120
        ["fks"]=30
        ["rustassistant"]=15
    )

    for repo in "${!PRIORITY_REPOS[@]}"; do
        enable_scan "$repo" "${PRIORITY_REPOS[$repo]}"
    done

    log_success "Priority repositories configured!"
}

# Show repository statistics
show_stats() {
    log_info "Repository Statistics:"
    echo ""

    sqlite3 "$DB_PATH" <<EOF
.headers on
.mode column
SELECT
    COUNT(*) as total_repos,
    SUM(CASE WHEN auto_scan_enabled = 1 THEN 1 ELSE 0 END) as scanning,
    SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END) as active
FROM repositories;
EOF

    echo ""
    log_info "Scan intervals breakdown:"
    sqlite3 "$DB_PATH" <<EOF
.headers on
.mode column
SELECT
    scan_interval_minutes as interval_mins,
    COUNT(*) as repos
FROM repositories
WHERE auto_scan_enabled = 1
GROUP BY scan_interval_minutes
ORDER BY scan_interval_minutes;
EOF
}

# Generate tasks from queue
show_queue() {
    log_info "Current task queue:"
    echo ""

    sqlite3 "$DB_PATH" <<EOF
.headers on
.mode column
SELECT
    id,
    title,
    priority,
    stage,
    created_at
FROM queue
ORDER BY priority, created_at
LIMIT 20;
EOF
}

# Clean up duplicate or invalid repos
cleanup_repos() {
    log_warning "Cleaning up repositories..."

    # Remove repos with invalid paths (URLs instead of local paths)
    local cleaned=$(sqlite3 "$DB_PATH" "DELETE FROM repositories WHERE path LIKE 'http%'; SELECT changes();")

    if [ "$cleaned" -gt 0 ]; then
        log_success "Removed $cleaned invalid repositories"
    else
        log_info "No cleanup needed"
    fi
}

# Interactive mode
interactive_mode() {
    while true; do
        echo ""
        echo "========================================="
        echo "  RustAssistant Repository Manager"
        echo "========================================="
        echo "1. List repositories"
        echo "2. Add repository"
        echo "3. Enable auto-scan"
        echo "4. Disable auto-scan"
        echo "5. Bulk add from GitHub directory"
        echo "6. Enable priority repos (recommended)"
        echo "7. Show statistics"
        echo "8. Show task queue"
        echo "9. Cleanup invalid repos"
        echo "0. Exit"
        echo ""
        read -p "Select option: " choice

        case $choice in
            1) list_repos ;;
            2)
                read -p "Repository path: " path
                read -p "Repository name: " name
                add_repo "$path" "$name"
                ;;
            3)
                read -p "Repository name: " name
                read -p "Scan interval (minutes) [60]: " interval
                interval=${interval:-60}
                enable_scan "$name" "$interval"
                ;;
            4)
                read -p "Repository name: " name
                disable_scan "$name"
                ;;
            5) bulk_add_github_repos ;;
            6) enable_priority_repos ;;
            7) show_stats ;;
            8) show_queue ;;
            9) cleanup_repos ;;
            0)
                log_info "Goodbye!"
                exit 0
                ;;
            *)
                log_error "Invalid option"
                ;;
        esac
    done
}

# Quick setup command
quick_setup() {
    log_info "Running quick setup for priority repositories..."

    # Clean up first
    cleanup_repos

    # Add priority repos
    declare -A REPOS=(
        ["rustscape"]="/home/jordan/github/rustscape"
        ["actions"]="/home/jordan/github/actions"
        ["scripts"]="/home/jordan/github/scripts"
        ["servers_sullivan"]="/home/jordan/github/servers_sullivan"
        ["servers_freddy"]="/home/jordan/github/servers_freddy"
    )

    for name in "${!REPOS[@]}"; do
        path="${REPOS[$name]}"
        add_repo "$path" "$name"
    done

    # Enable scanning
    enable_priority_repos

    log_success "Quick setup complete!"
    show_stats
}

# Main menu
show_usage() {
    cat << EOF
RustAssistant Repository Management Script

Usage: $0 [command] [options]

Commands:
    list                    List all repositories
    add <path> <name>       Add a repository
    enable <name> [int]     Enable auto-scan (optional interval in minutes)
    disable <name>          Disable auto-scan
    bulk                    Bulk add from GitHub directory
    priority                Enable priority repos (recommended)
    stats                   Show statistics
    queue                   Show task queue
    cleanup                 Remove invalid repositories
    quick                   Quick setup (add + enable priority repos)
    interactive, -i         Interactive mode
    help, -h                Show this help

Environment Variables:
    DB_PATH                 Path to database (default: ./data/rustassistant.db)
    GITHUB_BASE             Base directory for repos (default: /home/jordan/github)

Examples:
    $0 list
    $0 add /home/jordan/github/myrepo myrepo
    $0 enable rustscape 30
    $0 quick
    $0 interactive

EOF
}

# Main execution
main() {
    check_database

    case "${1:-interactive}" in
        list) list_repos ;;
        add) add_repo "$2" "$3" ;;
        enable) enable_scan "$2" "$3" ;;
        disable) disable_scan "$2" ;;
        bulk) bulk_add_github_repos ;;
        priority) enable_priority_repos ;;
        stats) show_stats ;;
        queue) show_queue ;;
        cleanup) cleanup_repos ;;
        quick) quick_setup ;;
        interactive|-i) interactive_mode ;;
        help|-h|--help) show_usage ;;
        *)
            log_error "Unknown command: $1"
            show_usage
            exit 1
            ;;
    esac
}

main "$@"
