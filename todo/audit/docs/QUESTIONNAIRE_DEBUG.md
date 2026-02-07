# Questionnaire Debugging Guide

This guide helps you diagnose and fix LLM questionnaire parsing failures in the CI Audit workflow.

## Quick Diagnosis Checklist

When you see `Files Audited: 0` in the questionnaire results:

1. ✅ **Check if the LLM API call succeeded**
   - Look for "API call successful" in the logs
   - Check for HTTP errors or timeouts

2. ✅ **Download the debug artifact**
   - Go to the failed workflow run
   - Download `llm-audit-full-X` artifact
   - Extract and examine `debug/questionnaire-failed-response.txt`

3. ✅ **Review the response format**
   - Is it valid JSON?
   - What structure does it use?
   - Is it wrapped in markdown code blocks?

4. ✅ **Check the step summary**
   - Failed responses show a preview in the GitHub Actions UI
   - Look for the "⚠️ Debug Info" section

## Expected Response Formats

The parser supports these formats (in order of precedence):

### Format 1: Wrapped Object (Preferred)
```json
{
  "file_audits": [
    {
      "file": "path/to/file.rs",
      "reachable": true,
      "compliance_issues": ["Issue 1", "Issue 2"],
      "incomplete": false,
      "suggested_tags": ["tag1", "tag2"],
      "improvement": "Suggestion text"
    }
  ]
}
```

### Format 2: Direct Array
```json
[
  {
    "file": "path/to/file.rs",
    "reachable": true,
    "compliance_issues": ["Issue 1"],
    "incomplete": false,
    "suggested_tags": ["tag1"],
    "improvement": "Suggestion text"
  }
]
```

### Format 3: Markdown-Wrapped
````markdown
```json
{
  "file_audits": [...]
}
```
````

### Format 4: Markdown-Wrapped Direct Array
````markdown
```json
[
  {...}
]
```
````

## Common Issues and Solutions

### Issue 1: Response is Not Valid JSON

**Symptoms:**
- Debug file shows natural language instead of JSON
- Preview shows "Here are the results..." or similar

**Solution:**
Update the prompt in `src/audit/src/llm.rs` → `build_questionnaire_system_prompt()` to be more explicit:

```rust
## OUTPUT FORMAT

CRITICAL: Respond with ONLY valid JSON. No preamble, no explanation, JUST the JSON object.

Use this EXACT format:
{
  "file_audits": [
    {
      "file": "services/forward/src/lib.rs",
      "reachable": true,
      "compliance_issues": [],
      "incomplete": false,
      "suggested_tags": ["forward-path", "real-time"],
      "improvement": "None needed"
    }
  ]
}
```

### Issue 2: Response is Truncated

**Symptoms:**
- Debug file ends mid-JSON
- Ends with `...` or incomplete array

**Solution:**
Increase `max_tokens` for questionnaire calls:

In `.github/workflows/ci-audit.yml`, update the questionnaire CLI call:
```bash
cargo run --release --bin audit-cli -- question \
    ../../src/janus \
    --provider ${{ inputs.llm_provider }} \
    --max-tokens 32000 \  # <-- Add this flag
    --output questionnaire-results.json
```

### Issue 3: Missing Required Fields

**Symptoms:**
- JSON parses but has missing fields
- Error: "missing field `reachable`"

**Solution:**
All `FileAuditResult` objects MUST have these fields:
- `file` (string)
- `reachable` (boolean)
- `compliance_issues` (array of strings)
- `incomplete` (boolean)
- `suggested_tags` (array of strings)
- `improvement` (string)

Update the prompt to explicitly list required fields with types.

### Issue 4: LLM Returns Unexpected Structure

**Symptoms:**
- Valid JSON but wrong top-level keys
- Uses `results` instead of `file_audits`

**Solution:**
Add parser support for the new format in `src/audit/src/llm.rs`:

```rust
// Add a new deserialize struct
#[derive(Deserialize)]
struct AlternativeResponse {
    results: Vec<FileAuditResult>,
}

// Add to parse_questionnaire_response():
if let Ok(result) = serde_json::from_str::<AlternativeResponse>(response) {
    return Ok(result.results);
}
```

## Debug Environment Variables

Set these in your workflow or locally:

```bash
# Enable debug logging
export RUST_LOG=audit=debug,audit::llm=debug

# Set debug output directory
export AUDIT_DEBUG_DIR=/path/to/debug/dir

# Run the questionnaire
cargo run --release --bin audit-cli -- question \
    /path/to/src \
    --provider xai \
    --output results.json
```

## Viewing Debug Logs

### In GitHub Actions

1. Go to the failed workflow run
2. Click on the "🧠 LLM Questionnaire" step
3. Expand the logs
4. Search for "WARN" or "DEBUG" to see detailed parsing attempts
5. Check the step summary for the preview of failed response

### Locally

```bash
cd src/audit

# Set debug variables
export RUST_LOG=audit=debug,audit::llm=debug
export AUDIT_DEBUG_DIR=./debug
mkdir -p debug

# Run questionnaire
cargo run --release --bin audit-cli -- question \
    ../../src/janus \
    --provider xai \
    --output questionnaire-results.json

# Check debug files
cat debug/questionnaire-failed-response.txt
```

## Advanced Debugging: Test the Parser

You can test the parser directly with a sample response:

```rust
// Add to src/audit/src/llm.rs tests
#[test]
fn test_parse_questionnaire_custom_format() {
    let client = LlmClient::new(
        "test".to_string(),
        "grok-4-1-fast-reasoning".to_string(),
        1000,
        0.2,
    ).unwrap();

    // Test your actual LLM response here
    let response = r#"{
      "file_audits": [
        {
          "file": "test.rs",
          "reachable": true,
          "compliance_issues": [],
          "incomplete": false,
          "suggested_tags": ["test"],
          "improvement": "None"
        }
      ]
    }"#;

    let result = client.parse_questionnaire_response(response).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].file, "test.rs");
}
```

Run the test:
```bash
cargo test test_parse_questionnaire_custom_format -- --nocapture
```

## Prompt Optimization Tips

If the LLM consistently fails to return the right format:

1. **Be more explicit about JSON-only output:**
   ```
   RESPOND WITH ONLY VALID JSON. 
   DO NOT include any text before or after the JSON.
   DO NOT wrap in markdown code blocks.
   START your response with { and END with }
   ```

2. **Provide a concrete example:**
   Show exactly what you want in the prompt.

3. **Request structured output:**
   Some LLMs support schema-based output (e.g., Google's function calling, OpenAI's structured outputs).

4. **Reduce batch size:**
   Instead of 50 files at once, try 10-20 files per request.

## File Checklist

When debugging, verify these files exist and have correct content:

- [ ] `src/audit/debug/questionnaire-failed-response.txt` - Raw LLM response
- [ ] `src/audit/questionnaire-results.json` - Parsed results (may be empty)
- [ ] Workflow step summary shows debug preview
- [ ] Artifact includes `debug/` directory

## Contact & Support

If you've tried all the above and still can't fix it:

1. **Create an issue** with:
   - The failed workflow run URL
   - Contents of `debug/questionnaire-failed-response.txt`
   - Expected vs actual format

2. **Check xAI/Google API status:**
   - xAI: https://status.x.ai
   - Google: https://status.cloud.google.com

3. **Try a different provider:**
   - Switch from `xai` to `google` or vice versa
   - Different models may have better JSON adherence

## Quick Fix: Disable Questionnaire

If questionnaire is blocking your CI and you need a quick workaround:

```yaml
# In .github/workflows/ci-audit.yml
- name: 🧠 LLM Questionnaire (Full Mode)
  if: false  # <-- Temporarily disable
```

Or set workflow input:
```yaml
run_questionnaire: false
```

This lets the rest of the audit run while you debug the questionnaire separately.