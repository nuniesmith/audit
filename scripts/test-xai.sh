#!/bin/bash
# Test xAI API integration locally
# Usage: XAI_API_KEY=xai-... ./scripts/test-xai.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 xAI API Integration Test Suite${NC}"
echo "========================================"
echo ""

# Check prerequisites
if [ -z "$XAI_API_KEY" ]; then
  echo -e "${RED}❌ XAI_API_KEY not set${NC}"
  echo "Usage: XAI_API_KEY=xai-... ./scripts/test-xai.sh"
  exit 1
fi

if ! command -v jq &> /dev/null; then
  echo -e "${RED}❌ jq not installed${NC}"
  echo "Install: brew install jq (macOS) or apt-get install jq (Linux)"
  exit 1
fi

if ! command -v bc &> /dev/null; then
  echo -e "${YELLOW}⚠️  bc not installed - cost calculations will be skipped${NC}"
fi

API_URL="https://api.x.ai/v1/responses"
MODEL="grok-4-1-fast-reasoning"

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: Basic connectivity
echo -e "${BLUE}Test 1: Basic Connectivity${NC}"
echo "Testing POST to $API_URL..."

RESPONSE=$(curl -s -X POST "$API_URL" \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"$MODEL\",
    \"input\": [{\"role\": \"user\", \"content\": \"Reply with just: OK\"}],
    \"max_tokens\": 10
  }")

if echo "$RESPONSE" | jq -e '.output[0].content[0].text' > /dev/null 2>&1; then
  RESULT=$(echo "$RESPONSE" | jq -r '.output[0].content[0].text')
  echo -e "${GREEN}✅ API responding: $RESULT${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${RED}❌ API not responding or wrong format${NC}"
  echo "Response:"
  echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
  TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 2: Response format validation
echo -e "${BLUE}Test 2: Response Format${NC}"
echo "Validating response structure..."

RESPONSE=$(curl -s -X POST "$API_URL" \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"$MODEL\",
    \"input\": [{\"role\": \"user\", \"content\": \"Return this exact JSON: {\\\"test\\\": true}\"}],
    \"max_tokens\": 50
  }")

# Check for correct path
if echo "$RESPONSE" | jq -e '.output[0].content[0].text' > /dev/null 2>&1; then
  echo -e "${GREEN}✅ Response path correct (.output[0].content[0].text)${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))

  # Validate content is JSON
  CONTENT=$(echo "$RESPONSE" | jq -r '.output[0].content[0].text')
  if echo "$CONTENT" | jq empty 2>/dev/null; then
    echo -e "${GREEN}✅ Content is valid JSON${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    echo -e "${RED}❌ Content is not valid JSON${NC}"
    echo "Content: $CONTENT"
    TESTS_FAILED=$((TESTS_FAILED + 1))
  fi
else
  echo -e "${RED}❌ Response format invalid${NC}"
  echo "Available keys:"
  echo "$RESPONSE" | jq 'keys' 2>/dev/null || echo "Not valid JSON"
  TESTS_FAILED=$((TESTS_FAILED + 2))
fi
echo ""

# Test 3: Usage tracking
echo -e "${BLUE}Test 3: Usage Tracking${NC}"
echo "Checking token usage and cost information..."

if echo "$RESPONSE" | jq -e '.usage' > /dev/null 2>&1; then
  echo -e "${GREEN}✅ Usage statistics present${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))

  PROMPT_TOKENS=$(echo "$RESPONSE" | jq -r '.usage.prompt_tokens // 0')
  COMPLETION_TOKENS=$(echo "$RESPONSE" | jq -r '.usage.completion_tokens // 0')
  TOTAL_TOKENS=$(echo "$RESPONSE" | jq -r '.usage.total_tokens // 0')
  CACHED_TOKENS=$(echo "$RESPONSE" | jq -r '.usage.prompt_tokens_details.cached_tokens // 0')

  echo "  Prompt tokens: $PROMPT_TOKENS"
  echo "  Completion tokens: $COMPLETION_TOKENS"
  echo "  Total tokens: $TOTAL_TOKENS"
  echo "  Cached tokens: $CACHED_TOKENS"

  # Calculate cost if bc is available
  if command -v bc &> /dev/null; then
    INPUT_COST=$(echo "scale=4; ($PROMPT_TOKENS - $CACHED_TOKENS) / 1000000 * 0.20" | bc)
    CACHED_COST=$(echo "scale=4; $CACHED_TOKENS / 1000000 * 0.05" | bc)
    OUTPUT_COST=$(echo "scale=4; $COMPLETION_TOKENS / 1000000 * 0.50" | bc)
    TOTAL_COST=$(echo "scale=4; $INPUT_COST + $CACHED_COST + $OUTPUT_COST" | bc)

    echo ""
    echo "  Cost breakdown:"
    echo "    Input: \$$INPUT_COST"
    echo "    Cached: \$$CACHED_COST"
    echo "    Output: \$$OUTPUT_COST"
    echo -e "    ${GREEN}Total: \$$TOTAL_COST${NC}"
  fi
else
  echo -e "${RED}❌ Usage statistics missing${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 4: Audit-style request
echo -e "${BLUE}Test 4: Audit-Style Request${NC}"
echo "Testing with structured audit prompt..."

RESPONSE=$(curl -s -X POST "$API_URL" \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"$MODEL\",
    \"input\": [
      {
        \"role\": \"system\",
        \"content\": \"You are an expert code auditor. Return valid JSON only.\"
      },
      {
        \"role\": \"user\",
        \"content\": \"Analyze this code: fn test() { let x = vec![1].unwrap(); }\\n\\nReturn JSON: {\\\"critical_findings\\\": [{\\\"id\\\": \\\"CODE-001\\\", \\\"title\\\": \\\"unwrap() usage\\\", \\\"severity\\\": \\\"medium\\\"}], \\\"summary\\\": \\\"Found 1 issue\\\"}\"
      }
    ],
    \"max_tokens\": 500,
    \"temperature\": 0.2
  }")

if echo "$RESPONSE" | jq -e '.output[0].content[0].text' > /dev/null 2>&1; then
  CONTENT=$(echo "$RESPONSE" | jq -r '.output[0].content[0].text')

  # Check if content is valid JSON
  if echo "$CONTENT" | jq empty 2>/dev/null; then
    echo -e "${GREEN}✅ Audit response is valid JSON${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))

    # Check for expected fields
    if echo "$CONTENT" | jq -e '.critical_findings // .summary' > /dev/null 2>&1; then
      echo -e "${GREEN}✅ Required audit fields present${NC}"
      TESTS_PASSED=$((TESTS_PASSED + 1))

      echo ""
      echo "Sample audit output:"
      echo "$CONTENT" | jq '.' | head -20
    else
      echo -e "${YELLOW}⚠️  Audit fields missing (but JSON is valid)${NC}"
      TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
  else
    echo -e "${RED}❌ Audit response is not valid JSON${NC}"
    echo "Content: $CONTENT"
    TESTS_FAILED=$((TESTS_FAILED + 2))
  fi
else
  echo -e "${RED}❌ Failed to get audit response${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 2))
fi
echo ""

# Test 5: Error handling
echo -e "${BLUE}Test 5: Error Handling${NC}"
echo "Testing with invalid request..."

ERROR_RESPONSE=$(curl -s -X POST "$API_URL" \
  -H "Authorization: Bearer invalid-key-test" \
  -H "Content-Type: application/json" \
  -d "{\"model\": \"$MODEL\", \"input\": []}")

if echo "$ERROR_RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
  ERROR_MSG=$(echo "$ERROR_RESPONSE" | jq -r '.error.message // .error')
  echo -e "${GREEN}✅ Error handling works: $ERROR_MSG${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${YELLOW}⚠️  Unexpected error response format${NC}"
  echo "$ERROR_RESPONSE" | jq '.' 2>/dev/null || echo "$ERROR_RESPONSE"
  TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Summary
echo "========================================"
echo -e "${BLUE}Test Summary${NC}"
echo "========================================"
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
  echo -e "${GREEN}✅ All tests passed! xAI integration is working correctly.${NC}"
  echo ""
  echo "Next steps:"
  echo "  1. Run Rust CLI: AUDIT_DEBUG_DIR=./debug cargo run --bin audit-cli -- --help"
  echo "  2. Test CI workflow: gh workflow run llm-audit.yml -f mode=quick -f llm_provider=xai"
  echo "  3. Check cost tracking: cat debug/llm-response.json | jq '.usage'"
  exit 0
else
  echo -e "${RED}❌ Some tests failed. Please review errors above.${NC}"
  echo ""
  echo "Common issues:"
  echo "  - Invalid API key: Check XAI_API_KEY is set correctly"
  echo "  - Rate limit: Wait a few seconds and retry"
  echo "  - Network: Check connectivity to api.x.ai"
  exit 1
fi
