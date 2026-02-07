#!/bin/bash
# Test script to verify workflow jq commands work correctly
# This simulates the GitHub Actions workflow parsing logic

set -e

echo "🧪 Testing Workflow JSON Parsing"
echo "=================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function
test_jq() {
    local description="$1"
    local json_input="$2"
    local jq_command="$3"
    local expected="$4"

    echo -n "Testing: $description ... "

    result=$(echo "$json_input" | jq -r "$jq_command" 2>/dev/null || echo "ERROR")

    if [ "$result" = "$expected" ]; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "  Expected: $expected"
        echo "  Got: $result"
        ((TESTS_FAILED++))
    fi
}

echo "📊 Static Analysis Parsing Tests"
echo "---------------------------------"

# Test 1: Parse critical count from real structure
STATIC_JSON='{
  "issues_by_severity": {
    "critical": 44,
    "high": 5,
    "medium": 77,
    "low": 11
  },
  "summary": {
    "total_files": 423,
    "total_issues": 137
  }
}'

test_jq "Critical count" "$STATIC_JSON" '.issues_by_severity.critical // 0' "44"
test_jq "High count" "$STATIC_JSON" '.issues_by_severity.high // 0' "5"
test_jq "Medium count" "$STATIC_JSON" '.issues_by_severity.medium // 0' "77"
test_jq "Low count" "$STATIC_JSON" '.issues_by_severity.low // 0' "11"

# Test 2: Handle missing fields
EMPTY_STATIC='{}'
test_jq "Missing critical (fallback)" "$EMPTY_STATIC" '.issues_by_severity.critical // 0' "0"
test_jq "Missing high (fallback)" "$EMPTY_STATIC" '.issues_by_severity.high // 0' "0"

echo ""
echo "🏷️  Tags Parsing Tests"
echo "---------------------"

# Test 3: Empty tags array
EMPTY_TAGS='[]'
test_jq "Empty tags count" "$EMPTY_TAGS" 'length' "0"

# Test 4: Tags with content
TAGS_WITH_CONTENT='[
  {"tag_type": "TODO", "file": "test.rs"},
  {"tag_type": "FIXME", "file": "main.rs"},
  {"tag_type": "AUDIT", "file": "core.rs"}
]'
test_jq "Tags count" "$TAGS_WITH_CONTENT" 'length' "3"

echo ""
echo "🔍 Edge Cases"
echo "-------------"

# Test 5: Malformed JSON handling
test_jq "Handle null" "null" 'length? // 0' "0"
test_jq "Handle empty object" '{}' '.issues_by_severity.critical // 0' "0"

echo ""
echo "📁 Actual File Tests"
echo "-------------------"

# Test with actual files if they exist
if [ -f "test-static.json" ]; then
    echo -n "Testing real static-analysis.json ... "
    CRITICAL=$(cat test-static.json | jq -r '.issues_by_severity.critical // 0')
    HIGH=$(cat test-static.json | jq -r '.issues_by_severity.high // 0')
    echo -e "${GREEN}✓${NC} Critical: $CRITICAL, High: $HIGH"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}⊘${NC} test-static.json not found (run CLI first)"
fi

if [ -f "test-tags.json" ]; then
    echo -n "Testing real audit-tags.json ... "
    TAG_COUNT=$(cat test-tags.json | jq -r 'length')
    echo -e "${GREEN}✓${NC} Tags: $TAG_COUNT"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}⊘${NC} test-tags.json not found (run CLI first)"
fi

echo ""
echo "=================================="
echo "📊 Test Results"
echo "=================================="
echo -e "${GREEN}Passed:${NC} $TESTS_PASSED"
echo -e "${RED}Failed:${NC} $TESTS_FAILED"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    echo "The workflow jq commands are correct."
    exit 0
else
    echo -e "${RED}❌ Some tests failed!${NC}"
    echo "Fix the jq commands before deploying."
    exit 1
fi
