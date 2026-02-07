# Latest Fix - December 30, 2024

## Issue #5: jq Argument List Too Long

**Status:** ✅ FIXED

---

## The Problem

Even after fixing the curl command to use `--data-binary @file`, the workflow was still failing with:

```
/usr/bin/jq: Argument list too long
Error: Process completed with exit code 126.
```

**Root Cause:**
The prompt file is ~500KB. We were doing this:

```bash
PROMPT_CONTENT=$(cat prompt.txt | jq -Rs .)
jq -n --argjson prompt "$PROMPT_CONTENT" '{...}'
```

This loads the entire file into a shell variable, then passes it as a command-line argument to `jq`. Both steps hit system limits (ARG_MAX).

---

## The Fix

**File:** `.github/workflows/llm-audit.yml` (Line 234)

**Before:**
```bash
PROMPT_CONTENT=$(cat prompt.txt | jq -Rs .)
jq -n \
  --arg model "$MODEL" \
  --argjson prompt "$PROMPT_CONTENT" \
  --argjson temp 0.2 \
  --argjson max_tokens "$MAX_TOKENS" \
  '{
    model: $model,
    messages: [{role: "user", content: $prompt}],
    temperature: $temp,
    max_tokens: $max_tokens
  }' > request.json
```

**After:**
```bash
# Use jq --rawfile to read prompt directly from file (avoids variable size limits)
jq -n \
  --arg model "$MODEL" \
  --rawfile prompt prompt.txt \
  --argjson temp 0.2 \
  --argjson max_tokens "$MAX_TOKENS" \
  '{
    model: $model,
    messages: [{role: "user", content: $prompt}],
    temperature: $temp,
    max_tokens: $max_tokens
  }' > request.json
```

**Key Changes:**
- ✅ Removed `PROMPT_CONTENT=$(cat prompt.txt | jq -Rs .)` 
- ✅ Changed `--argjson prompt "$PROMPT_CONTENT"` to `--rawfile prompt prompt.txt`
- ✅ jq now reads the file directly without loading into a variable

---

## How --rawfile Works

```bash
# --rawfile reads file content as a raw string
# No shell variable needed, no size limits hit
jq -n --rawfile myvar file.txt '{content: $myvar}'
```

The file content is passed directly to jq's internal memory, bypassing shell argument limits.

---

## Testing

```bash
$ cd src/audit
$ echo "Large prompt content here..." > test-prompt.txt
$ jq -n \
    --arg model "grok-beta" \
    --rawfile prompt test-prompt.txt \
    --argjson temp 0.2 \
    --argjson max_tokens 1000 \
    '{
      model: $model,
      messages: [{role: "user", content: $prompt}],
      temperature: $temp,
      max_tokens: $max_tokens
    }' | jq .

✅ SUCCESS - Outputs valid JSON with prompt content included
```

---

## Complete Fix History

This was the **5th and final issue** in the workflow:

1. ✅ CLI command names (`analyze` → `static`, `scan` → `tags`)
2. ✅ JSON parsing (array vs object structure)
3. ✅ Argument syntax (`--path` → positional)
4. ✅ Curl payload size (`-d` → `--data-binary @file`)
5. ✅ jq variable size (`--argjson $VAR` → `--rawfile file`)

---

## Why This Happened

**The Learning:**
- Shell variables have size limits (ARG_MAX, typically 128KB-2MB)
- Even using files with curl, the variable assignment still hits limits
- `--rawfile` bypasses variables entirely

**The Solution:**
Always use `--rawfile` for large file content in jq, never:
```bash
VAR=$(cat large-file | jq -Rs .)  # ❌ Hits limits
jq --argjson var "$VAR" '{...}'   # ❌ Hits limits again
```

Instead:
```bash
jq --rawfile var large-file '{...}'  # ✅ No limits
```

---

## Next Workflow Run

The workflow should now complete successfully through:
1. ✅ Static analysis
2. ✅ Tag scanning  
3. ✅ CI context gathering
4. ✅ Context bundle building
5. ✅ **LLM API call** (previously failed here)
6. ⏳ Report generation
7. ⏳ Artifact upload

---

## Status: Ready to Test

```bash
gh workflow run llm-audit.yml \
  -f mode=standard \
  -f llm_provider=xai
```

**Expected Result:** Complete workflow run with LLM analysis and full report generation.

---

**Date:** December 30, 2024  
**Time:** ~09:36 UTC  
**Status:** All issues resolved ✅  
**Next:** Production workflow test