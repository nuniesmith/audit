# xAI API Troubleshooting Guide

**Quick Reference for Common Issues**

---

## ✅ Quick Health Check

Run these checks if your LLM audit is failing:

```bash
# 1. Check API key is set
echo $XAI_API_KEY | cut -c1-10

# 2. Test basic connectivity
curl -I https://api.x.ai/v1/responses

# 3. Verify request file is valid JSON
jq empty request.json

# 4. Check compressed file size
ls -lh request.json.gz
```

---

## Common Issues

### Issue 1: API Call Times Out

**Symptoms:**
```
curl: (28) Operation timed out after X milliseconds
```

**Causes:**
- Request too large (> 2M tokens)
- Network issues
- xAI API overloaded

**Solutions:**
1. Check request size:
   ```bash
   wc -c request.json  # Should be < 2MB for text
   ```

2. Verify timeout is set:
   ```bash
   grep "\-m 3600" .github/workflows/llm-audit.yml
   ```

3. Reduce payload size:
   - Use `quick` mode instead of `deep`
   - Reduce `MAX_FILES` in workflow
   - Trim source code context

---

### Issue 2: Upload Speed Degradation

**Symptoms:**
```
% Total    % Received % Xferd  Average Speed
  100  563k    0     0  100  563k    25k     0 --:--:--  0:00:23
```

**Causes:**
- No compression enabled
- Network congestion
- Large payload

**Solutions:**
1. Verify compression is enabled:
   ```bash
   grep "Content-Encoding: gzip" .github/workflows/llm-audit.yml
   grep "\-\-compressed" .github/workflows/llm-audit.yml
   ```

2. Check compressed size:
   ```bash
   ls -lh request.json.gz
   # Should be ~25% of original
   ```

3. If still slow, reduce context:
   ```yaml
   head -c 250000 context-bundle/source-files.txt  # Instead of 500000
   ```

---

### Issue 3: Invalid Response Format

**Symptoms:**
```
jq: error: .output[0] is null
```

**Causes:**
- Wrong endpoint (using legacy `/v1/chat/completions`)
- API error response
- Empty response

**Solutions:**
1. Check endpoint:
   ```bash
   grep "api.x.ai/v1" .github/workflows/llm-audit.yml
   # Should show: https://api.x.ai/v1/responses
   ```

2. Inspect raw response:
   ```bash
   cat llm-response.json | jq '.'
   ```

3. Check for error messages:
   ```bash
   cat llm-response.json | jq -r '.error // "No error"'
   ```

4. Verify response structure:
   ```bash
   # xAI format: .output[0].message.content
   # Legacy format: .choices[0].message.content
   cat llm-response.json | jq -r '.output[0].message.content // .choices[0].message.content'
   ```

---

### Issue 4: Authentication Failed

**Symptoms:**
```
{"error": {"message": "Invalid API key", "type": "invalid_request_error"}}
```

**Causes:**
- Missing API key
- Incorrect API key
- API key not set in GitHub Secrets

**Solutions:**
1. Verify secret is set in GitHub:
   - Go to: Settings → Secrets → Actions
   - Check `XAI_API_KEY` exists

2. Test API key locally:
   ```bash
   curl https://api.x.ai/v1/responses \
     -H "Authorization: Bearer $XAI_API_KEY" \
     -H "Content-Type: application/json" \
     -d '{
       "model": "grok-4-1-fast-reasoning",
       "input": [{"role": "user", "content": "test"}],
       "max_tokens": 10
     }'
   ```

3. Check key format:
   ```bash
   echo $XAI_API_KEY | grep "^xai-"
   # Should start with "xai-"
   ```

---

### Issue 5: Rate Limit Exceeded

**Symptoms:**
```
{"error": {"message": "Rate limit exceeded", "type": "rate_limit_error"}}
```

**Causes:**
- Too many requests (> 480/min)
- Too many tokens (> 4M/min)

**Solutions:**
1. Check current limits:
   - Visit: https://console.x.ai/models
   - View: Requests per minute / Tokens per minute

2. Add delay between runs:
   ```yaml
   - name: Wait between requests
     run: sleep 10
   ```

3. Request limit increase:
   - Go to: https://console.x.ai/
   - Click: "Request increased rate limits"

---

### Issue 6: No Cost Savings from Caching

**Symptoms:**
- All tokens charged at $0.20/1M
- `cached_tokens: 0` in response

**Causes:**
- Static context changing between runs
- Cache expired (typically 5-10 min)
- Caching not enabled

**Solutions:**
1. Check response for cached tokens:
   ```bash
   cat llm-response.json | jq '.usage.prompt_tokens_details.cached_tokens'
   # Should be > 0 on second+ run
   ```

2. Ensure static context is consistent:
   ```bash
   # Don't include timestamps or run numbers in system prompt
   # Keep static-analysis.json format stable
   ```

3. Run audits close together:
   - Cache typically expires after 5-10 minutes
   - Run multiple audits in sequence for max savings

---

### Issue 7: JSON Parsing Failed

**Symptoms:**
```
jq: parse error: Invalid numeric literal
```

**Causes:**
- LLM returned markdown instead of JSON
- Response wrapped in code blocks
- Malformed JSON

**Solutions:**
1. Check for markdown wrapper:
   ```bash
   grep '```json' llm-analysis-raw.txt
   ```

2. Extract JSON from markdown:
   ```bash
   sed -n '/```json/,/```/p' llm-analysis-raw.txt | sed '1d;$d' > llm-analysis.json
   ```

3. Validate JSON:
   ```bash
   jq empty llm-analysis.json
   ```

4. Add to prompt:
   ```
   "Return ONLY valid JSON. Do not wrap in markdown code blocks."
   ```

---

### Issue 8: Retry Logic Not Working

**Symptoms:**
```
❌ All retries failed
```

**Causes:**
- All 3 attempts exhausted
- Network completely down
- API endpoint unreachable

**Solutions:**
1. Check retry count:
   ```bash
   grep "MAX_RETRIES" .github/workflows/llm-audit.yml
   # Default: 3
   ```

2. Increase retries:
   ```bash
   MAX_RETRIES=5  # Instead of 3
   ```

3. Add longer backoff:
   ```bash
   sleep $((5 * (2 ** RETRY_COUNT)))  # 5s, 10s, 20s, 40s
   ```

4. Check curl error logs:
   ```bash
   cat curl-error.log
   ```

---

## Debugging Workflow

### Step 1: Enable Verbose Logging

Add to workflow:
```yaml
- name: 🧠 LLM Analysis
  run: |
    set -x  # Enable debug mode
    # ... rest of script
```

### Step 2: Capture Request/Response

```bash
# Save request for inspection
cp request.json debug-request.json

# Save raw response
cp llm-response.json debug-response.json

# Upload as artifact
- uses: actions/upload-artifact@v4
  with:
    name: debug-files
    path: |
      src/audit/debug-*.json
      src/audit/curl-error.log
```

### Step 3: Test Locally

```bash
cd src/audit

# Build context bundle
./scripts/build-context.sh

# Test API call
curl -X POST "https://api.x.ai/v1/responses" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -m 3600 \
  --data @request.json \
  -v  # Verbose output

# Check response
cat llm-response.json | jq '.'
```

---

## Performance Benchmarks

### Expected Timings

| Step | Duration | Note |
|------|----------|------|
| Build context | 5-10s | Depends on file count |
| Compress request | <1s | Should be instant |
| Upload request | 5-8s | With compression |
| LLM processing | 10-20s | Varies by model load |
| Download response | <1s | Response is small |
| **Total** | **20-40s** | Total LLM analysis step |

### Payload Sizes

| Item | Uncompressed | Compressed | Ratio |
|------|--------------|------------|-------|
| Request JSON | ~600 KB | ~150 KB | 75% |
| Response JSON | ~20 KB | ~5 KB | 75% |

---

## Cost Monitoring

### Check Token Usage

```bash
# From response
cat llm-response.json | jq '.usage'

# Expected output:
{
  "prompt_tokens": 280000,
  "completion_tokens": 8000,
  "total_tokens": 288000,
  "prompt_tokens_details": {
    "cached_tokens": 200000  # Should be high!
  }
}
```

### Calculate Costs

```bash
# Input tokens (uncached)
echo "scale=4; 80000 / 1000000 * 0.20" | bc  # $0.016

# Input tokens (cached)
echo "scale=4; 200000 / 1000000 * 0.05" | bc  # $0.010

# Output tokens
echo "scale=4; 8000 / 1000000 * 0.50" | bc  # $0.004

# Total: ~$0.03 per run (with caching)
```

---

## Emergency Fixes

### Disable xAI, Use Fallback

```yaml
llm_provider:
  default: "google"  # Instead of "xai"
```

### Skip LLM Analysis

```yaml
- name: 🧠 LLM Analysis
  if: false  # Temporarily disable
```

### Use Smaller Context

```bash
MAX_FILES=10  # Instead of 100
head -c 100000 context-bundle/source-files.txt  # Instead of 500000
```

---

## Support Resources

1. **xAI Documentation:** https://docs.x.ai/
2. **xAI Console:** https://console.x.ai/
3. **Status Page:** Check xAI status for outages
4. **GitHub Actions Logs:** Full curl verbose output

---

## Checklist Before Asking for Help

- [ ] API key is valid and set in secrets
- [ ] Request size is < 2MB
- [ ] Compression is enabled (`-H "Content-Encoding: gzip"`)
- [ ] Timeout is set (`-m 3600`)
- [ ] Endpoint is correct (`/v1/responses`)
- [ ] Request format uses `input` not `messages`
- [ ] Response parsing checks both `output` and `choices`
- [ ] Retry logic is working (check logs)
- [ ] Tested locally with curl
- [ ] Checked xAI console for errors

---

**Last Updated:** January 2025