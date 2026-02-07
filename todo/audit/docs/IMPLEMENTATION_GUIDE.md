# xAI Integration Implementation Guide

**Quick start guide for implementing the recommended improvements**

---

## 🚀 Quick Start (15 minutes)

### 1. Test xAI Connection

```bash
cd src/audit

# Set your API key
export XAI_API_KEY="xai-your-key-here"

# Run the test suite
./scripts/test-xai.sh
```

**Expected output:**
```
✅ API responding: OK
✅ Response path correct (.output[0].content[0].text)
✅ Content is valid JSON
✅ Usage statistics present
✅ All tests passed!
```

### 2. Build and Test Rust Client

```bash
# Build the audit service
cargo build --release

# Test with a single file
./target/release/audit-cli analyze --file src/lib.rs --provider xai

# Enable debug output
export RUST_LOG=debug
export AUDIT_DEBUG_DIR=./debug
./target/release/audit-cli analyze --file src/lib.rs --provider xai

# Check debug output
cat debug/llm-response.json | jq '.usage'
```

### 3. Run CI Workflow

```bash
# Trigger manual audit
gh workflow run llm-audit.yml \
  -f mode=quick \
  -f llm_provider=xai \
  -f focus_areas="security"

# Watch the run
gh run watch

# Download artifacts
gh run download <run-id>
```

---

## 📋 Implementation Checklist

### Phase 1: Critical Fixes (Day 1) ✅

- [x] Updated `src/audit/src/llm.rs` with `/v1/responses` endpoint
- [x] Added xAI response types (`XaiResponse`, `XaiOutput`, etc.)
- [x] Added usage tracking and cost logging
- [x] Created test script (`scripts/test-xai.sh`)
- [x] Added cost analysis to CI workflow

**Status:** Complete - Ready to test!

### Phase 2: Validation (Day 2)

- [ ] Run test script and verify all tests pass
- [ ] Test Rust CLI with sample files
- [ ] Run CI workflow and check cost report
- [ ] Verify cached tokens on second run
- [ ] Review debug logs for errors

### Phase 3: Optimization (Week 2)

- [ ] Add prompt caching with cache keys
- [ ] Implement smart context selection
- [ ] Add cost budgeting limits
- [ ] Set up Prometheus metrics
- [ ] Create monthly cost reports

### Phase 4: Documentation (Week 3)

- [ ] Consolidate 50+ MD files into organized docs/
- [ ] Update main README with xAI guide
- [ ] Add troubleshooting runbook
- [ ] Create cost optimization guide

---

## 🔧 Key Changes Made

### 1. Rust Service (`src/audit/src/llm.rs`)

**Before:**
```rust
.post(format!("{}/chat/completions", self.base_url))
```

**After:**
```rust
.post(format!("{}/responses", self.base_url))
```

**Response parsing - Before:**
```rust
let completion: CompletionResponse = response.json().await?;
completion.choices.first()...
```

**Response parsing - After:**
```rust
let response_json: Value = response.json().await?;
response_json.pointer("/output/0/content/0/text")
    .or_else(|| response_json.pointer("/output/0/message/content"))
    .or_else(|| response_json.pointer("/choices/0/message/content"))
```

### 2. CI Workflow (`.github/workflows/llm-audit.yml`)

**Added cost tracking step:**
```yaml
- name: 💰 Cost Analysis
  if: inputs.llm_provider == 'xai'
  run: |
    # Extract usage stats
    PROMPT_TOKENS=$(jq -r '.usage.prompt_tokens // 0' llm-response.json)
    CACHED_TOKENS=$(jq -r '.usage.prompt_tokens_details.cached_tokens // 0' llm-response.json)
    
    # Calculate costs
    COST=$(echo "scale=4; $PROMPT_TOKENS / 1000000 * 0.20" | bc)
    
    # Generate report
    echo "| Metric | Value | Cost |" >> $GITHUB_STEP_SUMMARY
```

### 3. New Files Created

- `scripts/test-xai.sh` - Local testing suite
- `cost-history.jsonl` - Cost tracking database
- `docs/IMPLEMENTATION_GUIDE.md` - This file
- `docs/XAI_INTEGRATION_IMPROVEMENTS.md` - Detailed analysis

---

## 🧪 Testing Guide

### Local Testing

```bash
# 1. Environment setup
export XAI_API_KEY="xai-..."
export RUST_LOG=debug
export AUDIT_DEBUG_DIR=./debug
mkdir -p debug

# 2. Test API connection
./scripts/test-xai.sh

# 3. Test Rust client
cargo run --bin audit-cli -- analyze \
  --file src/scanner.rs \
  --provider xai

# 4. Check debug output
cat debug/llm-response.json | jq '.'
cat debug/llm-error-response.json  # If errors occurred

# 5. Review cost
cat debug/llm-response.json | jq '.usage | {
  prompt_tokens,
  cached_tokens: .prompt_tokens_details.cached_tokens,
  completion_tokens,
  reasoning_tokens: .completion_tokens_details.reasoning_tokens
}'
```

### CI Testing

```bash
# Quick audit (fast, cheap)
gh workflow run llm-audit.yml -f mode=quick -f llm_provider=xai

# Standard audit (balanced)
gh workflow run llm-audit.yml -f mode=standard -f llm_provider=xai

# Deep audit (comprehensive, expensive)
gh workflow run llm-audit.yml -f mode=deep -f llm_provider=xai

# Monitor run
gh run watch

# Download results
gh run download --name llm-audit-quick-123
```

### Verifying Cost Tracking

```bash
# After CI run completes
cd llm-audit-quick-123
cat cost-history.jsonl | jq .

# Expected output:
{
  "date": "2025-01-15T10:30:00Z",
  "run": "123",
  "tokens": {
    "prompt": 280000,
    "cached": 200000,
    "completion": 8330,
    "reasoning": 1200
  },
  "cost_usd": "0.0300"
}
```

---

## 📊 Cost Tracking

### Understanding Token Costs

| Token Type | Price | When Charged |
|------------|-------|--------------|
| Input (new) | $0.20/1M | First time seeing content |
| Input (cached) | $0.05/1M | Repeated static context |
| Output | $0.50/1M | LLM response |
| Reasoning | $0.50/1M | Internal reasoning (included in output) |

### Example Cost Breakdown

**Run #1 (No cache):**
```
280,000 prompt tokens × $0.20/1M = $0.056
  8,330 completion tokens × $0.50/1M = $0.004
Total: $0.060
```

**Run #2 (With cache):**
```
 80,000 new prompt tokens × $0.20/1M = $0.016
200,000 cached tokens × $0.05/1M = $0.010
  8,330 completion tokens × $0.50/1M = $0.004
Total: $0.030 (50% savings!)
```

### Checking Costs in CI

Look for this in your workflow summary:

```
## Cost Analysis

| Metric | Value | Cost |
|--------|-------|------|
| Prompt tokens | 280000 | $0.0560 |
| Cached tokens | 200000 | $0.0100 |
| Completion tokens | 8330 | $0.0042 |
| **Total** | **288330** | **$0.0702** |

✅ Cache hit: 71.4% of prompt tokens cached
```

---

## 🐛 Troubleshooting

### Issue: Test script fails with "Invalid API key"

**Solution:**
```bash
# Check your key is set
echo $XAI_API_KEY | cut -c1-10

# Should show: xai-...

# If empty, set it:
export XAI_API_KEY="xai-your-actual-key"
```

### Issue: Rust build fails

**Solution:**
```bash
# Clean and rebuild
cargo clean
cargo build --release

# Check for errors
cargo check
```

### Issue: CI workflow shows 0% cache hit

**Causes:**
1. First run (expected)
2. Static context changed
3. Cache expired (>10 minutes between runs)

**Solution:**
```bash
# Run two audits close together (<5 min apart)
gh workflow run llm-audit.yml -f mode=quick -f llm_provider=xai
sleep 60  # Wait 1 minute
gh workflow run llm-audit.yml -f mode=quick -f llm_provider=xai

# Second run should show 70%+ cache hit
```

### Issue: Response format errors

**Debug steps:**
```bash
# 1. Check response structure
cat debug/llm-response.json | jq 'keys'

# 2. Look for error field
cat debug/llm-response.json | jq '.error'

# 3. Check response path
cat debug/llm-response.json | jq '.output[0].content[0].text'

# 4. Review full response
cat debug/llm-response.json | jq '.'
```

### Issue: Costs higher than expected

**Diagnostic:**
```bash
# Check token counts
cat llm-response.json | jq '.usage'

# Compare against expected:
# - Quick mode: ~100K tokens = $0.02
# - Standard: ~280K tokens = $0.06
# - Deep: ~500K tokens = $0.10

# Check for cache misses
CACHED=$(jq -r '.usage.prompt_tokens_details.cached_tokens // 0' llm-response.json)
if [ "$CACHED" -eq 0 ]; then
  echo "Cache not working!"
fi
```

---

## 📈 Performance Metrics

### Expected Timings

| Step | Duration |
|------|----------|
| Build context | 5-10s |
| API request upload | 5-8s (compressed) |
| LLM processing | 10-20s |
| Download response | <1s |
| **Total** | **20-40s** |

### Benchmarking

```bash
# Time a local run
time cargo run --bin audit-cli -- analyze \
  --file src/scanner.rs \
  --provider xai

# Expected: 15-30 seconds
```

---

## 🎯 Next Steps

### Immediate (Today)

1. Run `./scripts/test-xai.sh` - verify API works
2. Test Rust CLI with sample file
3. Trigger CI workflow and check cost report

### Short-term (This Week)

4. Run 2+ audits to verify caching works
5. Review cost-history.jsonl for trends
6. Document any issues found

### Medium-term (Next Sprint)

7. Implement smart context selection
8. Add cost budgeting alerts
9. Set up Prometheus metrics
10. Create monthly cost reports

---

## 📚 Related Documentation

- **Main Analysis:** `docs/XAI_INTEGRATION_IMPROVEMENTS.md`
- **xAI Notes:** `docs/audit/notes_xai.md`
- **Troubleshooting:** `docs/audit/XAI_TROUBLESHOOTING.md`
- **API Reference:** https://docs.x.ai/

---

## ✅ Success Criteria

You'll know it's working when:

1. ✅ `test-xai.sh` shows all tests passing
2. ✅ Rust CLI returns valid audit JSON
3. ✅ CI workflow completes without errors
4. ✅ Cost report shows reasonable costs (<$0.10/run)
5. ✅ Second run shows >70% cache hit rate
6. ✅ Debug logs show proper response parsing

---

## 💡 Pro Tips

1. **Run test script before CI** - catch issues locally
2. **Monitor costs in xAI console** - verify CI calculations
3. **Use quick mode for testing** - saves money
4. **Keep static context stable** - maximizes caching
5. **Run audits close together** - cache expires after 10 min
6. **Review debug logs** - `RUST_LOG=debug` shows everything

---

**Last Updated:** January 2025  
**Status:** Ready for testing  
**Questions?** See troubleshooting section or open an issue