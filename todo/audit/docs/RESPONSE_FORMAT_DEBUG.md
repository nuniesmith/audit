# xAI Response Format Debugging Guide

**Issue:** API calls succeeding but validation failing with "Invalid response format"

---

## Problem Identified

The API is returning a response, but the structure doesn't match what we're checking for.

### What We Expected

```json
{
  "output": [
    {
      "message": {
        "content": "..."
      }
    }
  ]
}
```

### What We're Getting

Unknown - need to check the actual response structure.

---

## Quick Fix Applied

Updated the validation to accept **multiple formats**:

```bash
# Now checks for:
1. .output[0].message.content  # New xAI format
2. .choices[0].message.content # Legacy/OpenAI format
3. .message.content            # Direct message format
4. .content                    # Simple content field
5. .error                      # Error responses
```

---

## How to Debug

### Step 1: Check Actual Response Structure

After next run, download artifacts and examine `llm-response.json`:

```bash
# Download from: Actions → Run → Artifacts → llm-audit-standard-X
cd src/audit
cat llm-response.json | jq '.'
```

### Step 2: Look at CI Logs

The updated workflow now shows:

```
Response received. Checking format...
✅ API call successful (new format: .output)
# OR
✅ API call successful (legacy format: .choices)
# OR
⚠️  Unknown response format
Response structure: ["key1", "key2", "key3"]
Response preview: {...}
```

### Step 3: Common Response Formats

#### Format 1: New xAI (/v1/responses)
```json
{
  "output": [
    {
      "message": {
        "role": "assistant",
        "content": "..."
      }
    }
  ],
  "usage": {...}
}
```

#### Format 2: Legacy OpenAI-compatible
```json
{
  "choices": [
    {
      "message": {
        "role": "assistant",
        "content": "..."
      }
    }
  ],
  "usage": {...}
}
```

#### Format 3: Direct Message
```json
{
  "message": {
    "content": "..."
  }
}
```

#### Format 4: Error Response
```json
{
  "error": {
    "message": "Rate limit exceeded",
    "type": "rate_limit_error"
  }
}
```

---

## Possible Causes

### 1. Endpoint Mismatch
- `/v1/responses` should return `output` format
- `/v1/chat/completions` returns `choices` format

**Check:** Verify endpoint in workflow is `/v1/responses`

### 2. API Version Change
xAI might have updated their response format.

**Check:** Compare against latest xAI docs

### 3. Request Format Issue
Using `input` instead of `messages` might affect response.

**Check:** Request format should use `input` for new endpoint

### 4. Streaming Response
If streaming is enabled, format might differ.

**Check:** Ensure no `stream: true` in request

---

## Debugging Steps

### Local Test

```bash
cd src/audit

# Create minimal test request
cat > test-request.json <<'EOF'
{
  "model": "grok-4-1-fast-reasoning",
  "input": [
    {
      "role": "user",
      "content": "Say 'test successful' in JSON format"
    }
  ],
  "max_tokens": 50
}
EOF

# Compress and send
gzip -c test-request.json > test-request.json.gz

curl -X POST "https://api.x.ai/v1/responses" \
  -H "Content-Type: application/json" \
  -H "Content-Encoding: gzip" \
  -H "Authorization: Bearer $XAI_API_KEY" \
  --compressed \
  --data-binary @test-request.json.gz \
  -o test-response.json \
  -w "\nHTTP Status: %{http_code}\n"

# Check response
echo "Response structure:"
cat test-response.json | jq 'keys'

echo -e "\nFull response:"
cat test-response.json | jq '.'
```

### Compare Endpoints

Test both endpoints to see the difference:

```bash
# Test new endpoint
curl -X POST "https://api.x.ai/v1/responses" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -d '{"model":"grok-4-1-fast-reasoning","input":[{"role":"user","content":"test"}],"max_tokens":10}' \
  | jq 'keys'

# Test legacy endpoint
curl -X POST "https://api.x.ai/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -d '{"model":"grok-4-1-fast-reasoning","messages":[{"role":"user","content":"test"}],"max_tokens":10}' \
  | jq 'keys'
```

---

## Updated Workflow Features

### Better Validation

```bash
# Checks multiple formats
if jq -e '.output[0].message.content' ...; then
  echo "✅ New format"
elif jq -e '.choices[0].message.content' ...; then
  echo "✅ Legacy format"
elif jq -e '.message.content' ...; then
  echo "✅ Direct format"
else
  echo "⚠️ Unknown format"
  cat response | jq 'keys'  # Show structure
fi
```

### Better Error Reporting

```bash
# Shows actual response structure when validation fails
Response structure: ["output", "usage", "id"]
Response preview: {"output":[{"message":...
```

### More Flexible Parsing

```bash
# Tries all possible paths
jq -r '.output[0].message.content // 
       .choices[0].message.content // 
       .message.content // 
       .content // 
       "{\"error\":\"No content found\"}"'
```

---

## Expected Behavior After Fix

### Successful Response
```
Analyzing with grok-4-1-fast-reasoning...
Attempt 1/3...
Response received. Checking format...
✅ API call successful (new format: .output)
Parsed content length: 8330 bytes
✅ Analysis complete
```

### With Retries
```
Attempt 1/3...
Response received. Checking format...
⚠️  Unknown response format
Response structure: ["error"]
Attempt 2/3...
✅ API call successful (legacy format: .choices)
```

### Complete Failure
```
Attempt 3/3...
⚠️  Unknown response format
Response structure: ["error", "message"]
Response preview: {"error":{"message":"Rate limit exceeded"}}
❌ All retries failed
```

---

## Next Steps

1. **Run the workflow again** - It should now show which format it detects
2. **Check the logs** - Look for "API call successful (X format)"
3. **Review artifacts** - Examine `llm-response.json` if still failing
4. **Update docs** - Once we know the format, document it

---

## Questions to Answer

- [ ] What endpoint is actually being used?
- [ ] What response format is xAI returning?
- [ ] Does the format match xAI's latest docs?
- [ ] Are we using the correct request format?
- [ ] Is there an error in the response?

---

**Status:** Debugging in progress  
**Next Run:** Will show detailed format information  
**Last Updated:** January 2025