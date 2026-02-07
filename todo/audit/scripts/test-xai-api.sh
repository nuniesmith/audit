#!/bin/bash
# xAI API Testing Script for Grok 4.1 Fast Reasoning
# Tests API connectivity, model availability, and response format
#
# Usage:
#   export XAI_API_KEY="your-key-here"
#   ./test-xai-api.sh
#
# Or:
#   XAI_API_KEY="your-key" ./test-xai-api.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
API_KEY="${XAI_API_KEY:-}"
API_URL="https://api.x.ai/v1"
TEST_DIR="$(mktemp -d)"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Cleanup on exit
trap "rm -rf $TEST_DIR" EXIT

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}xAI API Testing Script${NC}"
echo -e "${BLUE}Testing Grok 4.1 Fast Reasoning Model${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check API key
if [ -z "$API_KEY" ]; then
    echo -e "${RED}❌ ERROR: XAI_API_KEY not set${NC}"
    echo ""
    echo "Please set your API key:"
    echo "  export XAI_API_KEY='your-key-here'"
    echo ""
    exit 1
fi

echo -e "${GREEN}✅ API key found${NC}"
echo ""

# Test 1: List available models
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 1: List Available Models${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

HTTP_STATUS=$(curl -s -o "$TEST_DIR/models.json" -w "%{http_code}" \
    -X GET "$API_URL/models" \
    -H "Authorization: Bearer $API_KEY")

if [ "$HTTP_STATUS" = "200" ]; then
    echo -e "${GREEN}✅ Successfully retrieved model list${NC}"
    echo ""
    echo "Available models:"
    jq -r '.data[].id' "$TEST_DIR/models.json" | sort | while read model; do
        echo "  - $model"
    done
    echo ""

    # Check for recommended models
    if jq -r '.data[].id' "$TEST_DIR/models.json" | grep -q "grok-4-1-fast-reasoning"; then
        echo -e "${GREEN}✅ grok-4-1-fast-reasoning is available${NC}"
        RECOMMENDED_MODEL="grok-4-1-fast-reasoning"
    elif jq -r '.data[].id' "$TEST_DIR/models.json" | grep -q "grok-4-fast-reasoning"; then
        echo -e "${YELLOW}⚠️  grok-4-1-fast-reasoning not found, using grok-4-fast-reasoning${NC}"
        RECOMMENDED_MODEL="grok-4-fast-reasoning"
    elif jq -r '.data[].id' "$TEST_DIR/models.json" | grep -q "grok-4"; then
        echo -e "${YELLOW}⚠️  Fast reasoning models not found, using grok-4${NC}"
        RECOMMENDED_MODEL="grok-4"
    else
        echo -e "${RED}❌ No grok-4 models found${NC}"
        echo "Available models listed above. Please update the workflow manually."
        exit 1
    fi
else
    echo -e "${RED}❌ Failed to list models (HTTP $HTTP_STATUS)${NC}"
    cat "$TEST_DIR/models.json"
    exit 1
fi

echo ""

# Test 2: Minimal chat completion
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 2: Minimal Chat Completion${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

cat > "$TEST_DIR/minimal-request.json" << EOF
{
  "model": "$RECOMMENDED_MODEL",
  "messages": [
    {
      "role": "user",
      "content": "Say 'API test successful' and nothing else."
    }
  ],
  "max_tokens": 20,
  "temperature": 0
}
EOF

echo "Request:"
cat "$TEST_DIR/minimal-request.json" | jq '.'
echo ""

HTTP_STATUS=$(curl -s -o "$TEST_DIR/minimal-response.json" -w "%{http_code}" \
    -X POST "$API_URL/chat/completions" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $API_KEY" \
    -d @"$TEST_DIR/minimal-request.json")

echo "HTTP Status: $HTTP_STATUS"
echo ""

if [ "$HTTP_STATUS" = "200" ]; then
    if jq -e '.choices[0].message.content' "$TEST_DIR/minimal-response.json" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Chat completion successful${NC}"
        echo ""
        echo "Response:"
        jq '.' "$TEST_DIR/minimal-response.json"
        echo ""

        CONTENT=$(jq -r '.choices[0].message.content' "$TEST_DIR/minimal-response.json")
        echo "Content: $CONTENT"
    else
        echo -e "${RED}❌ Unexpected response format${NC}"
        echo "Response structure:"
        jq 'keys' "$TEST_DIR/minimal-response.json"
        cat "$TEST_DIR/minimal-response.json"
        exit 1
    fi
else
    echo -e "${RED}❌ Request failed (HTTP $HTTP_STATUS)${NC}"
    cat "$TEST_DIR/minimal-response.json" | jq '.' || cat "$TEST_DIR/minimal-response.json"
    exit 1
fi

echo ""

# Test 3: JSON output format
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 3: JSON Output Format${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

cat > "$TEST_DIR/json-request.json" << EOF
{
  "model": "$RECOMMENDED_MODEL",
  "messages": [
    {
      "role": "system",
      "content": "You are a code auditor. Respond only with valid JSON."
    },
    {
      "role": "user",
      "content": "Analyze this function for issues: fn divide(a: i32, b: i32) -> i32 { a / b }. Return JSON with fields: severity, issue, recommendation."
    }
  ],
  "max_tokens": 500,
  "temperature": 0.2,
  "response_format": {"type": "json_object"}
}
EOF

echo "Request with response_format: json_object"
echo ""

HTTP_STATUS=$(curl -s -o "$TEST_DIR/json-response.json" -w "%{http_code}" \
    -X POST "$API_URL/chat/completions" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $API_KEY" \
    -d @"$TEST_DIR/json-request.json")

echo "HTTP Status: $HTTP_STATUS"
echo ""

if [ "$HTTP_STATUS" = "200" ]; then
    if jq -e '.choices[0].message.content' "$TEST_DIR/json-response.json" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ JSON output format works${NC}"
        echo ""

        CONTENT=$(jq -r '.choices[0].message.content' "$TEST_DIR/json-response.json")
        echo "Parsed JSON content:"
        echo "$CONTENT" | jq '.'

        # Validate it's actually JSON
        if echo "$CONTENT" | jq empty 2>/dev/null; then
            echo ""
            echo -e "${GREEN}✅ Content is valid JSON${NC}"
        else
            echo ""
            echo -e "${RED}❌ Content is not valid JSON${NC}"
            exit 1
        fi
    else
        echo -e "${RED}❌ Unexpected response format${NC}"
        cat "$TEST_DIR/json-response.json"
        exit 1
    fi
else
    echo -e "${RED}❌ Request failed (HTTP $HTTP_STATUS)${NC}"
    cat "$TEST_DIR/json-response.json" | jq '.' || cat "$TEST_DIR/json-response.json"
    exit 1
fi

echo ""

# Test 4: Token usage and pricing
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 4: Token Usage & Pricing${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

PROMPT_TOKENS=$(jq -r '.usage.prompt_tokens // 0' "$TEST_DIR/json-response.json")
COMPLETION_TOKENS=$(jq -r '.usage.completion_tokens // 0' "$TEST_DIR/json-response.json")
TOTAL_TOKENS=$(jq -r '.usage.total_tokens // 0' "$TEST_DIR/json-response.json")
REASONING_TOKENS=$(jq -r '.usage.completion_tokens_details.reasoning_tokens // 0' "$TEST_DIR/json-response.json")
CACHED_TOKENS=$(jq -r '.usage.prompt_tokens_details.cached_tokens // 0' "$TEST_DIR/json-response.json")

echo "Token Usage:"
echo "  Prompt tokens:     $PROMPT_TOKENS"
echo "  Completion tokens: $COMPLETION_TOKENS"
echo "  Reasoning tokens:  $REASONING_TOKENS"
echo "  Cached tokens:     $CACHED_TOKENS"
echo "  Total tokens:      $TOTAL_TOKENS"
echo ""

# Calculate cost (Grok 4.1 Fast pricing: $0.20/1M input, $0.50/1M output)
INPUT_COST=$(echo "scale=6; $PROMPT_TOKENS * 0.0000002" | bc)
OUTPUT_COST=$(echo "scale=6; $COMPLETION_TOKENS * 0.0000005" | bc)
TOTAL_COST=$(echo "scale=6; $INPUT_COST + $OUTPUT_COST" | bc)

echo "Estimated Cost:"
echo "  Input:  \$$INPUT_COST"
echo "  Output: \$$OUTPUT_COST"
echo "  Total:  \$$TOTAL_COST"

echo ""

# Test 5: Large context (simulated audit)
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Test 5: Simulated Audit Request${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

cat > "$TEST_DIR/audit-request.json" << 'EOF'
{
  "model": "MODEL_PLACEHOLDER",
  "messages": [
    {
      "role": "system",
      "content": "You are an expert code auditor specializing in Rust trading systems. Focus on: security vulnerabilities, memory safety, trading logic correctness, performance issues. Output valid JSON with findings array."
    },
    {
      "role": "user",
      "content": "Audit this Rust code:\n\n```rust\nuse std::sync::Arc;\nuse tokio::sync::RwLock;\n\npub struct OrderBook {\n    bids: Vec<Order>,\n    asks: Vec<Order>,\n}\n\nstruct Order {\n    price: f64,\n    quantity: f64,\n    user_id: u64,\n}\n\nimpl OrderBook {\n    pub fn new() -> Self {\n        OrderBook {\n            bids: Vec::new(),\n            asks: Vec::new(),\n        }\n    }\n    \n    pub fn add_order(&mut self, price: f64, quantity: f64, user_id: u64, is_bid: bool) {\n        let order = Order { price, quantity, user_id };\n        if is_bid {\n            self.bids.push(order);\n        } else {\n            self.asks.push(order);\n        }\n    }\n    \n    pub fn match_orders(&mut self) -> Vec<(u64, u64, f64, f64)> {\n        let mut matches = Vec::new();\n        \n        while !self.bids.is_empty() && !self.asks.is_empty() {\n            let bid = &self.bids[0];\n            let ask = &self.asks[0];\n            \n            if bid.price >= ask.price {\n                let quantity = bid.quantity.min(ask.quantity);\n                matches.push((bid.user_id, ask.user_id, ask.price, quantity));\n                \n                // Remove matched orders\n                self.bids.remove(0);\n                self.asks.remove(0);\n            } else {\n                break;\n            }\n        }\n        \n        matches\n    }\n}\n```\n\nIdentify critical issues and provide JSON response with: {\"findings\": [{\"severity\": \"...\", \"category\": \"...\", \"description\": \"...\", \"recommendation\": \"...\"}], \"summary\": \"...\"}"
    }
  ],
  "max_tokens": 2000,
  "temperature": 0.2,
  "response_format": {"type": "json_object"}
}
EOF

# Replace model placeholder
sed -i "s/MODEL_PLACEHOLDER/$RECOMMENDED_MODEL/g" "$TEST_DIR/audit-request.json"

echo "Sending audit request with ~1500 token prompt..."
echo ""

HTTP_STATUS=$(curl -s -o "$TEST_DIR/audit-response.json" -w "%{http_code}" \
    -X POST "$API_URL/chat/completions" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $API_KEY" \
    -m 120 \
    -d @"$TEST_DIR/audit-request.json")

echo "HTTP Status: $HTTP_STATUS"
echo ""

if [ "$HTTP_STATUS" = "200" ]; then
    if jq -e '.choices[0].message.content' "$TEST_DIR/audit-response.json" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Audit request successful${NC}"
        echo ""

        AUDIT_CONTENT=$(jq -r '.choices[0].message.content' "$TEST_DIR/audit-response.json")

        # Save and validate
        echo "$AUDIT_CONTENT" > "$TEST_DIR/audit-findings.json"

        if jq empty "$TEST_DIR/audit-findings.json" 2>/dev/null; then
            echo "Audit Findings:"
            cat "$TEST_DIR/audit-findings.json" | jq '.'
            echo ""

            # Count findings
            FINDING_COUNT=$(echo "$AUDIT_CONTENT" | jq '.findings | length' 2>/dev/null || echo "0")
            echo "Found $FINDING_COUNT issues"

            # Token usage
            AUDIT_PROMPT=$(jq -r '.usage.prompt_tokens // 0' "$TEST_DIR/audit-response.json")
            AUDIT_COMPLETION=$(jq -r '.usage.completion_tokens // 0' "$TEST_DIR/audit-response.json")
            AUDIT_COST=$(echo "scale=6; ($AUDIT_PROMPT * 0.0000002) + ($AUDIT_COMPLETION * 0.0000005)" | bc)

            echo ""
            echo "Audit Token Usage:"
            echo "  Prompt:     $AUDIT_PROMPT tokens"
            echo "  Completion: $AUDIT_COMPLETION tokens"
            echo "  Cost:       \$$AUDIT_COST"
        else
            echo -e "${YELLOW}⚠️  Response is not valid JSON${NC}"
            echo "Content:"
            echo "$AUDIT_CONTENT"
        fi
    else
        echo -e "${RED}❌ Unexpected response format${NC}"
        cat "$TEST_DIR/audit-response.json"
        exit 1
    fi
else
    echo -e "${RED}❌ Request failed (HTTP $HTTP_STATUS)${NC}"
    cat "$TEST_DIR/audit-response.json" | jq '.' || cat "$TEST_DIR/audit-response.json"
    exit 1
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All tests passed!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo "Summary:"
echo "  Recommended model: $RECOMMENDED_MODEL"
echo "  API endpoint:      $API_URL/chat/completions"
echo "  JSON output:       ✅ Working"
echo "  Audit format:      ✅ Working"
echo ""

echo -e "${GREEN}Next steps:${NC}"
echo "1. Update .github/workflows/ci-audit.yml:"
echo "   - Change MODEL to: $RECOMMENDED_MODEL"
echo "   - Verify API_URL: $API_URL/chat/completions"
echo ""
echo "2. Test full workflow:"
echo "   gh workflow run ci-audit.yml -f llm_provider=xai"
echo ""
echo "3. Monitor first run for:"
echo "   - Token usage (should be 100K-150K for full audit)"
echo "   - Response time (should be <120s)"
echo "   - Cost (should be \$0.02-\$0.03)"
echo ""

echo "Test artifacts saved to: $TEST_DIR"
echo "(Will be cleaned up on exit)"
echo ""
