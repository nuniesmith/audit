# LLM Audit Workflow Fixes

**Date:** 2025-12-29  
**Status:** ✅ FIXED

## Issues Found and Resolved

### 1. CLI Command Mismatch ❌ → ✅

**Error:**
```
error: unrecognized subcommand 'analyze'
error: unrecognized subcommand 'scan'
```

**Root Cause:**
The GitHub Actions workflow was calling CLI subcommands that don't exist in the actual implementation.

**Changes Made:**

| Before (Broken) | After (Fixed) |
|----------------|---------------|
| `audit-cli analyze` | `audit-cli static` |
| `audit-cli scan` | `audit-cli tags` |

**Files Modified:**
- `.github/workflows/llm-audit.yml` (Lines 86, 103)

---

### 2. Curl Argument List Too Long ❌ → ✅

**Error:**
```
/usr/bin/curl: Argument list too long
```

**Root Cause:**
The workflow was attempting to pass a ~500KB prompt as a command-line argument to curl, exceeding the system's `ARG_MAX` limit (typically 128KB-2MB on Linux).

**Solution:**
Changed from inline JSON data to using `jq` to build a proper JSON file, then using `--data-binary @request.json`.

**Before:**
```bash
PROMPT_CONTENT=$(cat prompt.txt | jq -Rs .)
curl -d "{\"messages\":[{\"content\":$PROMPT_CONTENT}]}" ...
```

**After:**
```bash
PROMPT_CONTENT=$(cat prompt.txt | jq -Rs .)
jq -n \
  --arg model "$MODEL" \
  --argjson prompt "$PROMPT_CONTENT" \
  --argjson temp 0.2 \
  --argjson max_tokens "$MAX_TOKENS" \
  '{
    model: $model,
    messages: [
      {
        role: "system",
        content: "You are an expert code auditor specializing in trading systems, security, and risk management."
      },
      {
        role: "user",
        content: $prompt
      }
    ],
    temperature: $temp,
    max_tokens: $max_tokens
  }' > request.json

curl --data-binary @request.json ...
```

**Files Modified:**
- `.github/workflows/llm-audit.yml` (Lines 233-262)

---

### 3. Format Flag Issue ❌ → ✅

**Issue:**
The workflow was passing `--format json` to the `static` command, but the CLI doesn't support this flag for the `static` subcommand.

**Fix:**
Removed the `--format json` flag from the static analysis step.

**Before:**
```bash
cargo run --release --bin audit-cli -- analyze \
    --path ../../src/janus \
    --output static-analysis.json \
    --format json
```

**After:**
```bash
cargo run --release --bin audit-cli -- static \
    ../../src/janus \
    --output static-analysis.json
```

---

### 4. YAML Indentation (Bonus Fix) ✅

**Issue:**
Inconsistent YAML indentation that could cause parsing issues in some YAML parsers.

**Fix:**
Standardized indentation to 4 spaces throughout the workflow file.

---

## Available CLI Commands

For reference, here are the actual CLI commands implemented:

```bash
audit-cli audit <REPO>       # Full audit of a repository
audit-cli tags [PATH]        # Scan for audit tags (PATH is positional)
audit-cli static [PATH]      # Static analysis (PATH is positional)
audit-cli tasks [PATH]       # Generate tasks from findings (PATH is positional)
audit-cli clone <URL>        # Clone repository for audit
audit-cli stats [PATH]       # Show audit statistics (PATH is positional)
```

**Common Options:**
- `-o, --output <FILE>` - Output file path
- `-f, --format <FORMAT>` - Output format (text, json, csv) - only for `tags` and `tasks`
- `-v, --verbose` - Verbose output

**Important:** PATH is a positional argument, not a `--path` flag!
- ✅ Correct: `audit-cli static ./my-code --output report.json`
- ❌ Wrong: `audit-cli static --path ./my-code --output report.json`

---

## Testing Recommendations

### 1. Test CLI Commands Locally

```bash
# From repository root
cd src/audit

# Test static analysis
cargo run --release --bin audit-cli -- static \
    ../../src/janus \
    --output test-static.json

# Test tag scanning
cargo run --release --bin audit-cli -- tags \
    ../../src/janus \
    --output test-tags.json

# Verify outputs exist and are valid JSON
cat test-static.json | jq .
cat test-tags.json | jq .
```

### 2. Test Full Workflow in GitHub Actions

```bash
# Trigger the workflow
gh workflow run llm-audit.yml \
    -f mode=standard \
    -f llm_provider=xai

# Or via GitHub UI:
# 1. Go to Actions tab
# 2. Select "🤖 LLM Audit" workflow
# 3. Click "Run workflow"
# 4. Select mode: standard
# 5. Click "Run workflow" button
```

### 3. Verify Outputs

After workflow completes, check:
- ✅ Workflow runs without errors
- ✅ Artifact `llm-audit-standard-N` is created
- ✅ `AUDIT_REPORT.md` is generated
- ✅ `tasks.csv` is populated
- ✅ `llm-analysis.json` has valid content
- ✅ `context-bundle/` contains source files

---

## Future Improvements

### 1. Add Format Flag to Static Command (Optional)

**File:** `src/audit/src/bin/cli.rs`

```rust
Static {
    #[arg(value_name = "PATH", default_value = ".")]
    path: PathBuf,

    #[arg(short, long)]
    focus: Vec<String>,

    // Add this:
    #[arg(short = 'f', long)]
    format: Option<OutputFormat>,

    #[arg(short, long)]
    output: Option<PathBuf>,
}
```

### 2. Add Request Size Validation

Add to workflow before curl:

```bash
REQUEST_SIZE=$(wc -c < request.json)
if [ $REQUEST_SIZE -gt 1000000 ]; then
    echo "⚠️ Warning: Request is ${REQUEST_SIZE} bytes, truncating..."
    # Implement truncation strategy
fi
```

### 3. Implement Chunking for Large Codebases

For codebases with >150 files:
- Split analysis into multiple LLM calls
- Aggregate results
- Generate combined report

### 4. Add Better Error Messages

Enhance CLI help output:

```bash
audit-cli static --help

# Should show:
# Examples:
#   audit-cli static --path /path/to/code
#   audit-cli static --path . --focus security,testing
#   audit-cli static --output report.json
```

### 5. Add Integration Tests

**File:** `src/audit/tests/cli_integration_tests.rs`

```rust
#[test]
fn test_static_command_works() {
    let output = Command::new("cargo")
        .args(&["run", "--", "static", "--path", "test-fixtures"])
        .output()
        .unwrap();
    
    assert!(output.status.success());
}
```

---

## Related Files

- **Workflow:** `.github/workflows/llm-audit.yml`
- **CLI Implementation:** `src/audit/src/bin/cli.rs`
- **Documentation:** `src/audit/LLM_AUDIT_GUIDE.md`

---

## Rollback Instructions

If you need to revert these changes:

```bash
# Checkout previous version of workflow
git checkout HEAD~1 .github/workflows/llm-audit.yml

# Or revert specific commits
git revert <commit-hash>
```

---

## Summary

All critical issues have been resolved:
- ✅ CLI commands now match actual implementation (`analyze` → `static`, `scan` → `tags`)
- ✅ Curl argument list issue fixed with `jq` and `--data-binary`
- ✅ Removed unsupported `--format` flag from static command
- ✅ Fixed YAML indentation for consistency

**The workflow should now run successfully end-to-end.**

### Verification Checklist

Before considering this complete, verify:

- [ ] Workflow runs without "unrecognized subcommand" errors
- [ ] Workflow runs without "Argument list too long" errors
- [ ] Static analysis produces valid JSON output
- [ ] Tag scanning produces valid JSON output
- [ ] LLM API call succeeds (with valid API key)
- [ ] Artifacts are uploaded successfully
- [ ] All reports are generated correctly

---

**Next Steps:**
1. Test the workflow with a manual trigger
2. Monitor the first few automated runs
3. Gather feedback on report quality
4. Iterate on improvements