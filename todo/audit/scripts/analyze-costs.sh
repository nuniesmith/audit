#!/usr/bin/env bash
# ============================================================================
# LLM Audit Cost Analysis Script
# ============================================================================
# Analyzes historical cost tracking data from LLM audits
# Usage: ./analyze-costs.sh [path-to-cost-tracking-dir]

set -euo pipefail

COST_DIR="${1:-./cost-tracking}"
REPORT_FILE="COST_ANALYSIS_REPORT.md"

echo "🔍 Analyzing LLM audit costs from: $COST_DIR"

if [ ! -d "$COST_DIR" ]; then
    echo "❌ Cost tracking directory not found: $COST_DIR"
    echo "💡 Download cost-tracking artifacts from GitHub Actions first"
    exit 1
fi

# Count available runs
TOTAL_RUNS=$(find "$COST_DIR" -name "run-*.json" 2>/dev/null | wc -l)

if [ "$TOTAL_RUNS" -eq 0 ]; then
    echo "❌ No cost tracking data found in $COST_DIR"
    exit 1
fi

echo "📊 Found $TOTAL_RUNS audit runs"
echo ""

# Initialize aggregation variables
TOTAL_COST=0
TOTAL_TOKENS=0
TOTAL_FILES=0
declare -A PROVIDER_COSTS
declare -A PROVIDER_RUNS
declare -A DEPTH_COSTS
declare -A DEPTH_RUNS

# Process each run
for run_file in "$COST_DIR"/run-*.json; do
    if [ -f "$run_file" ]; then
        # Extract data using jq if available, otherwise use grep
        if command -v jq &> /dev/null; then
            COST=$(jq -r '.estimated_cost_usd // 0' "$run_file")
            TOKENS=$(jq -r '.estimated_tokens // 0' "$run_file")
            FILES=$(jq -r '.files_analyzed // 0' "$run_file")
            PROVIDER=$(jq -r '.provider // "unknown"' "$run_file")
            DEPTH=$(jq -r '.depth // "unknown"' "$run_file")
        else
            COST=$(grep -o '"estimated_cost_usd": [0-9.]*' "$run_file" | grep -o '[0-9.]*' || echo "0")
            TOKENS=$(grep -o '"estimated_tokens": [0-9]*' "$run_file" | grep -o '[0-9]*' || echo "0")
            FILES=$(grep -o '"files_analyzed": [0-9]*' "$run_file" | grep -o '[0-9]*' || echo "0")
            PROVIDER=$(grep -o '"provider": "[^"]*"' "$run_file" | cut -d'"' -f4 || echo "unknown")
            DEPTH=$(grep -o '"depth": "[^"]*"' "$run_file" | cut -d'"' -f4 || echo "unknown")
        fi

        # Aggregate totals
        TOTAL_COST=$(echo "$TOTAL_COST + $COST" | bc)
        TOTAL_TOKENS=$((TOTAL_TOKENS + TOKENS))
        TOTAL_FILES=$((TOTAL_FILES + FILES))

        # Aggregate by provider
        PROVIDER_COSTS[$PROVIDER]=$(echo "${PROVIDER_COSTS[$PROVIDER]:-0} + $COST" | bc)
        PROVIDER_RUNS[$PROVIDER]=$((${PROVIDER_RUNS[$PROVIDER]:-0} + 1))

        # Aggregate by depth
        DEPTH_COSTS[$DEPTH]=$(echo "${DEPTH_COSTS[$DEPTH]:-0} + $COST" | bc)
        DEPTH_RUNS[$DEPTH]=$((${DEPTH_RUNS[$DEPTH]:-0} + 1))
    fi
done

# Calculate averages
AVG_COST=$(echo "scale=4; $TOTAL_COST / $TOTAL_RUNS" | bc)
AVG_TOKENS=$((TOTAL_TOKENS / TOTAL_RUNS))
AVG_FILES=$((TOTAL_FILES / TOTAL_RUNS))

# Generate report
cat > "$REPORT_FILE" <<EOF
# LLM Audit Cost Analysis Report

**Generated:** $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Analysis Period:** Last $TOTAL_RUNS audit runs

---

## 📊 Overall Statistics

| Metric | Total | Average per Run |
|--------|-------|-----------------|
| **Audit Runs** | $TOTAL_RUNS | - |
| **Total Cost** | \$$TOTAL_COST | \$$AVG_COST |
| **Total Tokens** | $(printf "%'d" $TOTAL_TOKENS) | $(printf "%'d" $AVG_TOKENS) |
| **Files Analyzed** | $(printf "%'d" $TOTAL_FILES) | $(printf "%'d" $AVG_FILES) |

---

## 🏢 Cost by Provider

| Provider | Runs | Total Cost | Avg Cost/Run | % of Total |
|----------|------|------------|--------------|------------|
EOF

for provider in "${!PROVIDER_COSTS[@]}"; do
    runs=${PROVIDER_RUNS[$provider]}
    cost=${PROVIDER_COSTS[$provider]}
    avg=$(echo "scale=4; $cost / $runs" | bc)
    pct=$(echo "scale=1; ($cost / $TOTAL_COST) * 100" | bc)
    echo "| $provider | $runs | \$$cost | \$$avg | ${pct}% |" >> "$REPORT_FILE"
done

cat >> "$REPORT_FILE" <<EOF

---

## 📏 Cost by Analysis Depth

| Depth | Runs | Total Cost | Avg Cost/Run | % of Total |
|-------|------|------------|--------------|------------|
EOF

for depth in critical quick standard deep; do
    if [ -n "${DEPTH_COSTS[$depth]:-}" ]; then
        runs=${DEPTH_RUNS[$depth]}
        cost=${DEPTH_COSTS[$depth]}
        avg=$(echo "scale=4; $cost / $runs" | bc)
        pct=$(echo "scale=1; ($cost / $TOTAL_COST) * 100" | bc)
        echo "| $depth | $runs | \$$cost | \$$avg | ${pct}% |" >> "$REPORT_FILE"
    fi
done

cat >> "$REPORT_FILE" <<EOF

---

## 💡 Optimization Recommendations

EOF

# Calculate potential savings
if [ -n "${DEPTH_COSTS[deep]:-}" ] && [ -n "${DEPTH_COSTS[critical]:-}" ]; then
    DEEP_COST=${DEPTH_COSTS[deep]}
    CRITICAL_COST=${DEPTH_COSTS[critical]:-0}
    DEEP_RUNS=${DEPTH_RUNS[deep]}
    CRITICAL_RUNS=${DEPTH_RUNS[critical]:-1}

    DEEP_AVG=$(echo "scale=4; $DEEP_COST / $DEEP_RUNS" | bc)
    CRITICAL_AVG=$(echo "scale=4; $CRITICAL_COST / $CRITICAL_RUNS" | bc)
    SAVINGS=$(echo "scale=1; (($DEEP_AVG - $CRITICAL_AVG) / $DEEP_AVG) * 100" | bc)

    cat >> "$REPORT_FILE" <<EOF
### Switch from Deep to Critical Depth

- **Deep analysis avg cost:** \$$DEEP_AVG per run
- **Critical analysis avg cost:** \$$CRITICAL_AVG per run
- **Potential savings:** ${SAVINGS}% per run

**Recommendation:** Use \`critical\` depth for regular audits, reserve \`deep\` for:
- Pre-release comprehensive audits
- Major refactoring reviews
- Security incident investigations

EOF
fi

# Add provider optimization
XAI_COST=${PROVIDER_COSTS[xai]:-0}
GOOGLE_COST=${PROVIDER_COSTS[google]:-0}

if [ "$XAI_COST" != "0" ] && [ "$GOOGLE_COST" != "0" ]; then
    XAI_RUNS=${PROVIDER_RUNS[xai]}
    GOOGLE_RUNS=${PROVIDER_RUNS[google]}
    XAI_AVG=$(echo "scale=4; $XAI_COST / $XAI_RUNS" | bc)
    GOOGLE_AVG=$(echo "scale=4; $GOOGLE_COST / $GOOGLE_RUNS" | bc)

    if (( $(echo "$XAI_AVG > $GOOGLE_AVG" | bc -l) )); then
        CHEAPER="Google Gemini"
        SAVINGS_PCT=$(echo "scale=1; (($XAI_AVG - $GOOGLE_AVG) / $XAI_AVG) * 100" | bc)
    else
        CHEAPER="XAI Grok"
        SAVINGS_PCT=$(echo "scale=1; (($GOOGLE_AVG - $XAI_AVG) / $GOOGLE_AVG) * 100" | bc)
    fi

    cat >> "$REPORT_FILE" <<EOF
### Provider Selection

- **XAI Grok avg:** \$$XAI_AVG per run
- **Google Gemini avg:** \$$GOOGLE_AVG per run
- **More cost-effective:** $CHEAPER (${SAVINGS_PCT}% cheaper)

EOF
fi

cat >> "$REPORT_FILE" <<EOF

### Monthly Cost Projections

Based on current usage patterns:

| Frequency | Est. Monthly Runs | Est. Monthly Cost |
|-----------|-------------------|-------------------|
| Daily audits | 30 | \$$(echo "scale=2; $AVG_COST * 30" | bc) |
| Weekly audits | 4 | \$$(echo "scale=2; $AVG_COST * 4" | bc) |
| Bi-weekly | 2 | \$$(echo "scale=2; $AVG_COST * 2" | bc) |
| On-demand only | ~1-2 | \$$(echo "scale=2; $AVG_COST * 1.5" | bc) |

**Recommended:** Weekly critical audits + on-demand deep audits = ~\$$(echo "scale=2; $AVG_COST * 5" | bc)/month

---

## 📈 Recent Trends

EOF

# Show last 5 runs
echo "### Last 5 Audit Runs" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "| Run # | Date | Provider | Depth | Files | Tokens | Cost |" >> "$REPORT_FILE"
echo "|-------|------|----------|-------|-------|--------|------|" >> "$REPORT_FILE"

for run_file in $(ls -t "$COST_DIR"/run-*.json | head -5); do
    if command -v jq &> /dev/null; then
        RUN_NUM=$(jq -r '.run_number' "$run_file")
        TIMESTAMP=$(jq -r '.timestamp' "$run_file" | cut -d'T' -f1)
        PROVIDER=$(jq -r '.provider' "$run_file")
        DEPTH=$(jq -r '.depth' "$run_file")
        FILES=$(jq -r '.files_analyzed' "$run_file")
        TOKENS=$(jq -r '.estimated_tokens' "$run_file")
        COST=$(jq -r '.estimated_cost_usd' "$run_file")
    else
        RUN_NUM=$(basename "$run_file" .json | cut -d'-' -f2)
        TIMESTAMP=$(grep -o '"timestamp": "[^"]*"' "$run_file" | cut -d'"' -f4 | cut -d'T' -f1)
        PROVIDER=$(grep -o '"provider": "[^"]*"' "$run_file" | cut -d'"' -f4)
        DEPTH=$(grep -o '"depth": "[^"]*"' "$run_file" | cut -d'"' -f4)
        FILES=$(grep -o '"files_analyzed": [0-9]*' "$run_file" | grep -o '[0-9]*')
        TOKENS=$(grep -o '"estimated_tokens": [0-9]*' "$run_file" | grep -o '[0-9]*')
        COST=$(grep -o '"estimated_cost_usd": [0-9.]*' "$run_file" | grep -o '[0-9.]*')
    fi

    echo "| #$RUN_NUM | $TIMESTAMP | $PROVIDER | $depth | $FILES | $(printf "%'d" $TOKENS) | \$$COST |" >> "$REPORT_FILE"
done

cat >> "$REPORT_FILE" <<EOF

---

## 🎯 Action Items

1. **Review depth settings:** Ensure most runs use \`critical\` or \`quick\` depth
2. **Optimize frequency:** Balance audit coverage with budget constraints
3. **Monitor trends:** Track if costs increase with codebase growth
4. **Provider testing:** Periodically test both providers for quality vs. cost
5. **Budget alerts:** Set up alerts if monthly costs exceed threshold

---

*Generated by analyze-costs.sh | For more details, see individual run files in $COST_DIR*
EOF

echo "✅ Cost analysis complete!"
echo ""
echo "📄 Report saved to: $REPORT_FILE"
echo ""
echo "📊 Summary:"
echo "  • Total runs analyzed: $TOTAL_RUNS"
echo "  • Total cost: \$$TOTAL_COST"
echo "  • Average cost per run: \$$AVG_COST"
echo "  • Average tokens per run: $(printf "%'d" $AVG_TOKENS)"
echo ""
echo "💡 View the full report: cat $REPORT_FILE"
