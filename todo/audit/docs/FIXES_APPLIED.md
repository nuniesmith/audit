# Audit Workflow Fixes - Quick Summary

**Date:** December 29, 2024  
**Status:** ✅ ALL ISSUES FIXED

---

## 🐛 Issues Found

### 1. Wrong CLI Commands
- ❌ Workflow called `analyze` → ✅ Fixed to `static`
- ❌ Workflow called `scan` → ✅ Fixed to `tags`

### 2. Curl Argument Too Large
- ❌ Trying to pass 500KB+ in command line
- ✅ Fixed by using `jq` + `--data-binary @file`

### 3. Invalid Format Flag
- ❌ Using `--format json` with `static` command (unsupported)
- ✅ Removed the flag

---

## 📝 Changes Made

**File:** `.github/workflows/llm-audit.yml`

### Line 86 (Static Analysis)
```yaml
# BEFORE
cargo run --release --bin audit-cli -- analyze \
    --path ../../src/janus \
    --output static-analysis.json \
    --format json

# AFTER
cargo run --release --bin audit-cli -- static \
    ../../src/janus \
    --output static-analysis.json
```

### Line 103 (Tag Scanning)
```yaml
# BEFORE
cargo run --release --bin audit-cli -- scan \
    --path ../../src/janus \
    --output audit-tags.json

# AFTER
cargo run --release --bin audit-cli -- tags \
    ../../src/janus \
    --output audit-tags.json
```

### Lines 233-262 (LLM API Call)
```yaml
# BEFORE
PROMPT_CONTENT=$(cat prompt.txt | jq -Rs .)
curl -d "{\"messages\":[{\"content\":$PROMPT_CONTENT}]}" ...

# AFTER
PROMPT_CONTENT=$(cat prompt.txt | jq -Rs .)
jq -n \
  --arg model "$MODEL" \
  --argjson prompt "$PROMPT_CONTENT" \
  --argjson temp 0.2 \
  --argjson max_tokens "$MAX_TOKENS" \
  '{
    model: $model,
    messages: [...],
    temperature: $temp,
    max_tokens: $max_tokens
  }' > request.json

curl --data-binary @request.json ...
```

---

## ✅ Testing

### Quick Test
```bash
cd src/audit

# Test static analysis
cargo run --release --bin audit-cli -- static ../../src/janus

# Test tag scanning
cargo run --release --bin audit-cli -- tags ../../src/janus
```

### Full Workflow Test
```bash
gh workflow run llm-audit.yml -f mode=standard -f llm_provider=xai
```

---

## 📚 Available CLI Commands

```bash
audit-cli audit <REPO>        # Full audit with optional LLM
audit-cli static [PATH]       # Static analysis only (fast)
audit-cli tags [PATH]         # Scan for audit tags
audit-cli tasks [PATH]        # Generate task list
audit-cli clone <URL>         # Clone repo for audit
audit-cli stats [PATH]        # Show statistics

# Note: PATH is a positional argument, not --path flag
# Example: audit-cli static ./my-code --output report.json
```

---

## 🎯 Result

The workflow will now:
1. ✅ Run static analysis successfully
2. ✅ Scan for audit tags successfully
3. ✅ Send LLM requests without "argument too long" errors
4. ✅ Generate complete audit reports
5. ✅ Upload artifacts with all outputs

---

## 📖 More Info

- **Detailed fixes:** See `WORKFLOW_FIXES.md`
- **Workflow file:** `.github/workflows/llm-audit.yml`
- **CLI source:** `src/bin/cli.rs`
- **Usage guide:** `LLM_AUDIT_GUIDE.md`

---

**Ready to run!** 🚀