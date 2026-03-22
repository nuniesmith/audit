#!/usr/bin/env bash
# =============================================================================
# test-content-format.sh — Verify OaiMessage content deserialization fix
# =============================================================================
# Tests that the ra-app /v1/chat/completions proxy correctly handles both:
#   - Plain string content:  "content": "Hello"
#   - Array-of-parts content: "content": [{"type":"text","text":"Hello"}]
#
# The array form is sent by OpenClaw (and newer OpenAI client libraries) for
# multi-turn conversations. Before the fix, ra-app returned HTTP 422:
#   "invalid type: sequence, expected a string"
#
# Usage:
#   ./scripts/test-content-format.sh
#   ./scripts/test-content-format.sh --host 100.69.78.116 --port 3500
#   ./scripts/test-content-format.sh --verbose
# =============================================================================

set -euo pipefail

# ── Colours ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

pass()  { echo -e "${GREEN}  ✓ PASS${NC}  $*"; }
fail()  { echo -e "${RED}  ✗ FAIL${NC}  $*"; FAILURES=$((FAILURES + 1)); }
info()  { echo -e "${CYAN}  ℹ${NC}  $*"; }
warn()  { echo -e "${YELLOW}  ⚠${NC}  $*"; }
step()  { echo -e "\n${BOLD}$*${NC}"; }

# ── Defaults ─────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$PROJECT_ROOT/.env"

TARGET_HOST=""
PORT="3500"
API_KEY=""
VERBOSE=false
FAILURES=0

# ── Parse arguments ───────────────────────────────────────────────────────────
while [ $# -gt 0 ]; do
    case "$1" in
        --host)    TARGET_HOST="$2"; shift 2 ;;
        --port)    PORT="$2";    shift 2 ;;
        --key)     API_KEY="$2"; shift 2 ;;
        --verbose|-v) VERBOSE=true; shift ;;
        --help|-h)
            sed -n '2,/^# ====/{ /^# ====/d; s/^# \{0,1\}//; p }' "$0"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# ── Load .env if values not supplied via CLI ──────────────────────────────────
if [ -f "$ENV_FILE" ]; then
    set -a
    # shellcheck disable=SC1090
    source "$ENV_FILE" 2>/dev/null || true
    set +a
fi

TARGET_HOST="${TARGET_HOST:-${TAILSCALE_IP:-127.0.0.1}}"
API_KEY="${API_KEY:-${RA_PROXY_API_KEYS:-}}"
# If multiple keys are comma-separated, use the first one.
API_KEY="${API_KEY%%,*}"

BASE_URL="http://${TARGET_HOST}:${PORT}"
COMPLETIONS_URL="${BASE_URL}/v1/chat/completions"

# ── Preflight ─────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║  ra-app OaiMessage Content Format Test                       ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
info "Target:  $COMPLETIONS_URL  (host: ${TARGET_HOST})"
info "API key: ${API_KEY:+${API_KEY:0:8}…<redacted>}${API_KEY:-<none — open endpoint>}"
echo ""

# ── Helper: POST and parse ────────────────────────────────────────────────────
# Usage: do_request <label> <json_body>
# Prints pass/fail and (with --verbose) the raw response.
# Returns 0 on success, 1 on failure (but never exits — FAILURES counter used).
do_request() {
    local label="$1"
    local body="$2"

    local auth_header=""
    if [ -n "$API_KEY" ]; then
        auth_header="-H \"Authorization: Bearer ${API_KEY}\""
    fi

    local http_code response_body
    # Write body/code to temp files to avoid subshell quoting issues.
    local tmp_body tmp_code
    tmp_body="$(mktemp)"
    tmp_code="$(mktemp)"

    curl -s -o "$tmp_body" -w "%{http_code}" \
        -X POST "$COMPLETIONS_URL" \
        -H "Content-Type: application/json" \
        ${API_KEY:+-H "Authorization: Bearer ${API_KEY}"} \
        --data "$body" \
        > "$tmp_code" 2>/dev/null || true

    http_code="$(cat "$tmp_code" 2>/dev/null || echo 000)"
    response_body="$(cat "$tmp_body" 2>/dev/null || echo "")"
    rm -f "$tmp_body" "$tmp_code"

    if $VERBOSE; then
        echo "    HTTP $http_code"
        echo "    $(echo "$response_body" | head -c 500)"
        echo ""
    fi

    if [ "$http_code" = "000" ]; then
        fail "$label — could not connect to $COMPLETIONS_URL"
        return 1
    fi

    if [ "$http_code" = "422" ]; then
        local err_excerpt
        err_excerpt="$(echo "$response_body" | grep -o '"message":"[^"]*"' | head -1 || echo "$response_body" | head -c 200)"
        fail "$label — HTTP 422 (deserialization error): $err_excerpt"
        return 1
    fi

    if [ "$http_code" = "401" ] || [ "$http_code" = "403" ]; then
        fail "$label — HTTP $http_code (auth error — check RA_PROXY_API_KEYS)"
        return 1
    fi

    if [ "$http_code" != "200" ]; then
        local err_excerpt
        err_excerpt="$(echo "$response_body" | head -c 200)"
        fail "$label — HTTP $http_code: $err_excerpt"
        return 1
    fi

    # Extract the reply text from choices[0].message.content
    local content
    content="$(echo "$response_body" \
        | grep -o '"content":"[^"]*"' \
        | head -1 \
        | sed 's/"content":"//;s/"$//' \
        || echo "")"

    if [ -z "$content" ]; then
        warn "$label — HTTP 200 but could not extract response text (may still be OK)"
    fi

    pass "$label${content:+ → \"${content:0:60}\"}"
    return 0
}

# =============================================================================
# Test suite
# =============================================================================

step "1. Health check"
health_code="$(curl -sf -o /dev/null -w "%{http_code}" "${BASE_URL}/health" 2>/dev/null || echo 000)"
if [ "$health_code" = "200" ]; then
    pass "GET /health → 200"
else
    fail "GET /health → $health_code (is ra-app running?)"
    echo ""
    echo -e "${RED}Cannot reach ra-app — aborting remaining tests.${NC}"
    exit 1
fi

# ── 2. Plain string content ───────────────────────────────────────────────────
step "2. Plain string content  (baseline — should always work)"

do_request \
    "Single user message, string content" \
    '{"model":"auto","messages":[{"role":"user","content":"Reply with exactly one word: HELLO"}],"max_tokens":10}'

do_request \
    "System + user, both string content" \
    '{"model":"auto","messages":[{"role":"system","content":"You are a concise assistant."},{"role":"user","content":"Reply with exactly one word: HELLO"}],"max_tokens":10}'

# ── 3. Array-of-parts content ─────────────────────────────────────────────────
step "3. Array-of-parts content  (OpenClaw / new OpenAI SDK format — was broken)"

do_request \
    "Single text part array" \
    '{"model":"auto","messages":[{"role":"user","content":[{"type":"text","text":"Reply with exactly one word: HELLO"}]}],"max_tokens":10}'

do_request \
    "Multiple text parts (joined with newline)" \
    '{"model":"auto","messages":[{"role":"user","content":[{"type":"text","text":"Reply with exactly"},{"type":"text","text":"one word: HELLO"}]}],"max_tokens":10}'

do_request \
    "Non-text part silently ignored (image_url)" \
    '{"model":"auto","messages":[{"role":"user","content":[{"type":"image_url","image_url":{"url":"https://example.com/img.png"}},{"type":"text","text":"Reply with exactly one word: HELLO"}]}],"max_tokens":10}'

# ── 4. Mixed multi-turn (the exact scenario OpenClaw sends) ──────────────────
step "4. Mixed multi-turn  (string system + string history + array last turn)"

do_request \
    "System string + 2-turn history + array user turn" \
    '{"model":"auto","messages":[{"role":"system","content":"You are a concise assistant."},{"role":"user","content":"Hi there."},{"role":"assistant","content":"Hello! How can I help?"},{"role":"user","content":[{"type":"text","text":"Reply with exactly one word: HELLO"}]}],"max_tokens":10}'

do_request \
    "Deeper history (4 prior turns) with array final message" \
    '{"model":"auto","messages":[{"role":"system","content":"You are a concise assistant."},{"role":"user","content":"Turn 1"},{"role":"assistant","content":"Ack 1"},{"role":"user","content":"Turn 2"},{"role":"assistant","content":"Ack 2"},{"role":"user","content":[{"type":"text","text":"Reply with exactly one word: HELLO"}]}],"max_tokens":10}'

# ── 5. Edge cases ─────────────────────────────────────────────────────────────
step "5. Edge cases"

do_request \
    "Empty array content (degenerate — should not 422)" \
    '{"model":"auto","messages":[{"role":"user","content":[]},{"role":"user","content":"Reply with exactly one word: HELLO"}],"max_tokens":10}'

do_request \
    "Array with only non-text parts (empty text extracted — should not 422)" \
    '{"model":"auto","messages":[{"role":"user","content":[{"type":"image_url","image_url":{"url":"data:image/png;base64,abc"}}]},{"role":"user","content":"Reply with exactly one word: HELLO"}],"max_tokens":10}'

# =============================================================================
# Summary
# =============================================================================
echo ""
echo -e "${BOLD}══════════════════════════════════════════════════════════════${NC}"
if [ "$FAILURES" -eq 0 ]; then
    echo -e "${GREEN}${BOLD}  All tests passed ✓${NC}"
else
    echo -e "${RED}${BOLD}  $FAILURES test(s) failed ✗${NC}"
    echo ""
    echo "  Tips:"
    echo "    • Re-run with --verbose to see raw HTTP responses"
    echo "    • Check ra-app logs:  docker logs ra-app --tail 50"
    echo "    • Check if ra-app built with the fix:  docker logs ra-app | grep 'starting'"
fi
echo -e "${BOLD}══════════════════════════════════════════════════════════════${NC}"
echo ""

exit $FAILURES
