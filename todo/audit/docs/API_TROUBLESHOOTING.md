# API Troubleshooting Guide

**Last Updated:** December 2024
**Status:** Active

## Current Issue: xAI API Error

### Problem
The CI audit workflow is failing when calling the xAI API with the following error:
```
⚠️  API returned error:
jq: error (at <stdin>:0): Cannot index string with string "message"
```

### Root Cause
The xAI API is returning a plain error string instead of JSON, or the response format doesn't match what we're expecting.

## Quick Fixes

### 1. Check API Key
Ensure `XAI_API_KEY` secret is set in GitHub:
- Go to Settings → Secrets and variables → Actions
- Verify `XAI_API_KEY` exists and is valid
- Test key with curl:
```bash
curl -X POST "https://api.x.ai/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_KEY" \
  -d '{
    "model": "grok-beta",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 100
  }'
```

### 2. Verify Model Name
xAI models available:
- `grok-beta` - Main model
- `grok-2-1212` - Dated version
- Check [xAI docs](https://docs.x.ai/) for latest model names

### 3. Check Endpoint URL
Current endpoint: `https://api.x.ai/v1/chat/completions`

Alternative endpoints to try:
- `https://api.x.ai/v1/completions`
- `https://api.x.ai/v1/responses` (old format)

### 4. Response Format Issues
The xAI API might return:
- Error as plain string (not JSON)
- Different JSON structure than OpenAI
- Rate limit errors
- Authentication errors

## Debugging Steps

### Step 1: Check Raw Response
Add this to workflow to see actual response:
```bash
echo "=== RAW RESPONSE ===" >> $GITHUB_STEP_SUMMARY
cat llm-response.json >> $GITHUB_STEP_SUMMARY
echo "" >> $GITHUB_STEP_SUMMARY
```

### Step 2: Test Locally
```bash
cd src/audit

# Set API key
export XAI_API_KEY="your-key"

# Test with curl
curl -X POST "https://api.x.ai/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -d '{
    "model": "grok-beta",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Say hello"}
    ],
    "max_tokens": 50
  }' | jq '.'
```

### Step 3: Check API Status
- Visit https://status.x.ai/ (if available)
- Check xAI Discord/Twitter for outages
- Review xAI changelog for breaking changes

## Common Errors

### Authentication Failed
**Error:** `401 Unauthorized` or `Invalid API key`
**Fix:** 
- Regenerate API key at https://console.x.ai/
- Update GitHub secret
- Ensure no extra spaces in key

### Rate Limit
**Error:** `429 Too Many Requests`
**Fix:**
- Add exponential backoff (already implemented)
- Reduce request frequency
- Upgrade API plan

### Invalid Model
**Error:** `Model not found` or `Invalid model`
**Fix:**
- Check xAI docs for current model names
- Update MODEL variable in workflow
- Try `grok-2-1212` or latest version

### Request Too Large
**Error:** `Request entity too large`
**Fix:**
- Reduce MAX_TOKENS from 16000
- Limit source code size (currently 500KB)
- Split into smaller requests

### Timeout
**Error:** `Operation timed out`
**Fix:**
- Increase timeout from 300s to 600s
- Check network connectivity
- Try during off-peak hours

## Workarounds

### Use Google Gemini Instead
If xAI is down, use Google:
```bash
# In workflow UI, select:
llm_provider: google
```

### Reduce Request Size
Temporarily limit files analyzed:
```yaml
# In workflow, change:
MAX_FILES=150  # to
MAX_FILES=50   # smaller set
```

### Skip LLM Analysis
Comment out LLM steps temporarily:
```yaml
# - name: 🧠 LLM Analysis
#   id: llm-analysis
#   ...
```

## Testing New API Versions

### Test Script
```bash
#!/bin/bash
# test-xai-api.sh

API_KEY="${XAI_API_KEY}"
MODEL="grok-beta"
URL="https://api.x.ai/v1/chat/completions"

# Test basic request
curl -X POST "$URL" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d "{
    \"model\": \"$MODEL\",
    \"messages\": [{\"role\": \"user\", \"content\": \"Test\"}],
    \"max_tokens\": 10
  }" \
  -w "\nHTTP Status: %{http_code}\n" \
  -o response.json

echo ""
echo "Response:"
cat response.json | jq '.' || cat response.json
```

### Expected Response Format
```json
{
  "id": "chatcmpl-...",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "grok-beta",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Your response here"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 20,
    "total_tokens": 30
  }
}
```

## Alternative Solutions

### 1. Use Rust CLI Directly
The `audit-cli` can handle LLM calls:
```bash
cargo run --release --bin audit-cli -- question \
  ../../src/janus \
  --provider xai \
  --output results.json
```

### 2. Local LLM
Set up local LLM for testing:
```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Run local model
ollama run llama2

# Update audit CLI to use local endpoint
```

### 3. Different Provider
Try alternative providers:
- OpenAI (GPT-4)
- Anthropic (Claude)
- Google (Gemini)
- Local (Ollama)

## Getting Help

### Resources
- xAI Docs: https://docs.x.ai/
- xAI Console: https://console.x.ai/
- API Status: Check Twitter @xai
- GitHub Issues: Check this repo's issues

### Reporting Bugs
When reporting API issues, include:
1. Full error message
2. Request payload (sanitized)
3. Response (sanitized)
4. HTTP status code
5. Timestamp
6. Model name used

### Contact
- GitHub Issues: nuniesmith/fks
- Check workflow logs for detailed errors
- Review `llm-response.json` artifact

## Resolution Checklist

When API fails:
- [ ] Check API key is valid
- [ ] Verify model name is current
- [ ] Test endpoint with curl
- [ ] Check for rate limits
- [ ] Review xAI status/changelog
- [ ] Try with smaller request
- [ ] Test with different model
- [ ] Switch to Google Gemini
- [ ] Check GitHub Actions logs
- [ ] Download artifacts for details

## Prevention

### Best Practices
1. **Always validate API key** before workflow runs
2. **Monitor API costs** to avoid surprises
3. **Set up alerts** for workflow failures
4. **Keep model names updated** as xAI evolves
5. **Test locally first** before pushing changes
6. **Use fallback provider** (Google) as backup
7. **Cache static context** to reduce costs
8. **Implement retry logic** with backoff (✅ done)

### Monitoring
- Track workflow success rate
- Monitor API response times
- Watch for cost increases
- Review error patterns

---

**Note:** This is a living document. Update as API evolves and issues are resolved.