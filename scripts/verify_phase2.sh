#!/bin/bash
# Phase 2 Verification Script
# Tests all queue, scan, and report commands to ensure they work

set -e  # Exit on error

echo "🧪 RustAssistant Phase 2 Verification Script"
echo "=============================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if rustassistant is built
if [ ! -f "target/release/rustassistant" ] && [ ! -f "target/debug/rustassistant" ]; then
    echo -e "${YELLOW}⚠️  rustassistant not built. Building now...${NC}"
    cargo build --release
fi

# Determine which binary to use
if [ -f "target/release/rustassistant" ]; then
    RUSTASSISTANT="./target/release/rustassistant"
else
    RUSTASSISTANT="./target/debug/rustassistant"
fi

echo "Using: $RUSTASSISTANT"
echo ""

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Test function
test_command() {
    local description=$1
    shift
    local cmd="$@"

    echo -n "Testing: $description... "

    if $cmd > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗${NC}"
        echo "  Command: $cmd"
        ((TESTS_FAILED++))
        return 1
    fi
}

echo "=== Queue Commands ==="
test_command "queue add (thought)" $RUSTASSISTANT queue add "verification test thought" --source thought
test_command "queue add (note)" $RUSTASSISTANT queue add "verification test note" --source note --project test
test_command "queue status" $RUSTASSISTANT queue status
test_command "queue list inbox" $RUSTASSISTANT queue list inbox
test_command "queue list pending" $RUSTASSISTANT queue list pending --limit 5
echo ""

echo "=== Scan Commands ==="
if [ -n "$GITHUB_TOKEN" ]; then
    test_command "scan repos" $RUSTASSISTANT scan repos --token "$GITHUB_TOKEN"
else
    echo -e "${YELLOW}⚠️  Skipping 'scan repos' (GITHUB_TOKEN not set)${NC}"
fi

test_command "scan todos" $RUSTASSISTANT scan todos .
test_command "scan tree" $RUSTASSISTANT scan tree . --depth 2
test_command "scan unanalyzed" $RUSTASSISTANT scan unanalyzed . --limit 5
echo ""

echo "=== Report Commands ==="
test_command "report todos" $RUSTASSISTANT report todos
test_command "report todos (filtered)" $RUSTASSISTANT report todos --priority 2
test_command "report files" $RUSTASSISTANT report files
test_command "report health" $RUSTASSISTANT report health .
test_command "report standardization" $RUSTASSISTANT report standardization .
echo ""

echo "=== Core Commands ==="
test_command "stats" $RUSTASSISTANT stats
test_command "next" $RUSTASSISTANT next
echo ""

echo "=== Note Commands ==="
test_command "note add" $RUSTASSISTANT note add "verification test note" --tags test
test_command "note list" $RUSTASSISTANT note list --limit 5
test_command "note search" $RUSTASSISTANT note search "verification" --limit 5
echo ""

echo "=== Task Commands ==="
test_command "tasks list" $RUSTASSISTANT tasks list --limit 5
echo ""

echo "=== Repo Commands ==="
test_command "repo list" $RUSTASSISTANT repo list
echo ""

echo "=============================================="
echo "📊 Results:"
echo -e "  ${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "  ${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed! Queue system is working!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Check output above.${NC}"
    exit 1
fi
