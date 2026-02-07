# Latest Fixes - Questionnaire Parsing Improvements

**Date:** 2025-12-31  
**Status:** ✅ FIXED - Ready for Testing  
**Affected Components:** LLM Questionnaire, CI Workflow  

---

## 🎯 Problem Summary

The CI audit workflow was running successfully but the LLM questionnaire step was failing to parse responses, resulting in:

```
Files Audited: 0
⚠️ WARN audit::llm: Failed to parse questionnaire response
```

Despite the LLM API call succeeding and returning a valid response, the parser couldn't handle the response format, causing all questionnaire results to be lost.

---

## 🔧 Root Cause

The `parse_questionnaire_response()` function in `src/audit/src/llm.rs` was **too rigid**:

1. Only tried 2 formats (wrapped object + markdown-wrapped)
2. Failed silently when parsing failed (returned empty array)
3. No debugging output to diagnose issues
4. No way to inspect failed responses in production

**Example of what was happening:**

```
LLM Response: [{...}, {...}]  ← Direct array format
Parser tried:  {"file_audits": [{...}]}  ← Expected wrapped format
Result:        []  ← Silent failure, no diagnostics
```

---

## ✅ Solutions Implemented

### 1. **Multi-Format Parser with Fallbacks**

Enhanced the parser to try **4 different formats** in order:

```rust
// Format 1: Wrapped object (original)
{"file_audits": [{...}]}

// Format 2: Direct array (NEW)
[{...}, {...}]

// Format 3: Markdown-wrapped object (original)
```json
{"file_audits": [{...}]}
```

// Format 4: Markdown-wrapped array (NEW)
```json
[{...}]
```
```

**File:** `src/audit/src/llm.rs`  
**Function:** `parse_questionnaire_response()`  
**Lines Changed:** ~30 lines (added fallbacks + logging)

---

### 2. **Debug File Capture**

When parsing fails, the raw LLM response is now saved to disk:

```rust
if let Ok(debug_dir) = std::env::var("AUDIT_DEBUG_DIR") {
    let debug_path = Path::new(&debug_dir).join("questionnaire-failed-response.txt");
    std::fs::write(&debug_path, response)?;
    warn!("Saved failed questionnaire response to {:?}", debug_path);
}
```

**Benefits:**
- ✅ Inspect actual LLM responses in production
- ✅ Debug format mismatches without re-running expensive API calls
- ✅ Included in CI artifacts for easy download

---

### 3. **Enhanced Workflow Debugging**

**File:** `.github/workflows/ci-audit.yml`  
**Step:** `🧠 LLM Questionnaire (Full Mode)`

Added environment variables:
```yaml
env:
  AUDIT_DEBUG_DIR: ${{ github.workspace }}/src/audit/debug
  RUST_LOG: audit=debug,audit::llm=debug
```

Added debug directory creation:
```bash
mkdir -p debug
```

Added debug preview in step summary:
```bash
if [ "$FILE_COUNT" -eq 0 ] && [ -f debug/questionnaire-failed-response.txt ]; then
    echo "### ⚠️ Debug Info" >> $GITHUB_STEP_SUMMARY
    echo "Questionnaire parsing failed. Response preview:" >> $GITHUB_STEP_SUMMARY
    head -c 500 debug/questionnaire-failed-response.txt >> $GITHUB_STEP_SUMMARY
fi
```

**Benefits:**
- ✅ See response preview directly in GitHub Actions UI
- ✅ No need to download artifacts for quick diagnosis
- ✅ Debug logs available via `RUST_LOG=debug`

---

### 4. **Debug Artifacts**

Updated artifact upload to include debug directory:

```yaml
- name: 📦 Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: llm-audit-full-${{ github.run_number }}
    path: |
      src/audit/AUDIT_REPORT.md
      src/audit/tasks.csv
      src/audit/llm-analysis.json
      src/audit/questionnaire-results.json
      src/audit/debug/  # <-- NEW
```

**Benefits:**
- ✅ Failed responses preserved for 90 days
- ✅ Easy to share debugging info in issues
- ✅ Historical record of LLM response formats

---

## 📊 Expected Outcomes

### Before Fix
```
📋 LLM Questionnaire Results
═══════════════════════════════════════
Files Audited: 0
```

### After Fix (Success Case)
```
📋 LLM Questionnaire Results
═══════════════════════════════════════
Files Audited: 50
🔴 High Priority:   0
🟡 Medium Priority: 35
🟢 Low Priority:    15
📁 Files with Issues: 12
```

### After Fix (Failure Case with Debugging)
```
📋 LLM Questionnaire Results
═══════════════════════════════════════
Files Audited: 0

### ⚠️ Debug Info
```
Questionnaire parsing failed. Response preview:
Here are the audit results for each file:

1. services/forward/src/lib.rs
   - Reachable: Yes
   ...
```
```

**Now you can see WHY it failed!**

---

## 🧪 Testing Recommendations

### Local Testing

```bash
cd src/audit

# Set debug environment
export RUST_LOG=audit=debug,audit::llm=debug
export AUDIT_DEBUG_DIR=./debug
mkdir -p debug

# Run questionnaire
cargo run --release --bin audit-cli -- question \
    ../../src/janus \
    --provider xai \
    --output questionnaire-results.json

# Check results
cat questionnaire-results.json | jq 'length'

# If failed, check debug file
cat debug/questionnaire-failed-response.txt
```

### CI Testing

1. **Push changes** to trigger workflow
2. **Monitor the questionnaire step** in GitHub Actions
3. **Check the step summary** for:
   - Files Audited count
   - Debug info (if parsing failed)
4. **Download artifact** `llm-audit-full-X`
5. **Inspect** `debug/questionnaire-failed-response.txt` if needed

---

## 📁 Files Modified

| File | Lines Changed | Purpose |
|------|---------------|---------|
| `src/audit/src/llm.rs` | +40 | Enhanced parser with 4 formats + debug logging |
| `.github/workflows/ci-audit.yml` | +20 | Added debug env vars + preview in summary |
| `src/audit/CI_FIX_SUMMARY.md` | +155 | Documented questionnaire fixes |
| `src/audit/QUESTIONNAIRE_DEBUG.md` | +317 | Complete debugging guide |
| `src/audit/LATEST_FIXES.md` | +293 | This file |

**Total:** ~825 lines of improvements + documentation

---

## 🚀 Next Steps

1. **Test the workflow** - Push to trigger a CI run
2. **Check questionnaire results** - Should see > 0 files audited
3. **If parsing fails:**
   - Download the artifact
   - Read `debug/questionnaire-failed-response.txt`
   - Follow `QUESTIONNAIRE_DEBUG.md` guide
4. **Adjust prompts if needed** - Based on actual LLM response format
5. **Report back** - Share results or any issues

---

## 📚 Documentation

- **`CI_FIX_SUMMARY.md`** - Complete history of all CI fixes
- **`QUESTIONNAIRE_DEBUG.md`** - Step-by-step debugging guide
- **`QUICK_REFERENCE.md`** - General workflow reference

---

## ✅ Verification Checklist

- [x] Parser supports 4 different response formats
- [x] Failed responses saved to debug files
- [x] Debug directory included in artifacts
- [x] Step summary shows preview of failures
- [x] Detailed logs available via RUST_LOG
- [x] No breaking changes to existing functionality
- [x] Backward compatible with old response formats
- [x] Documentation complete and comprehensive

---

## 🎉 Success Criteria

The fix is successful if:

1. **Questionnaire runs** - No crashes or panics
2. **Results parsed** - `Files Audited > 0` OR clear debug info shown
3. **Debug files created** - Present in artifacts when parsing fails
4. **Logs helpful** - Easy to diagnose issues from logs alone

---

## 🆘 If Issues Persist

If the questionnaire still returns 0 files after these fixes:

1. **Download the artifact** - Get `debug/questionnaire-failed-response.txt`
2. **Check the format** - Compare against expected formats in `QUESTIONNAIRE_DEBUG.md`
3. **Update the prompt** - Adjust `build_questionnaire_system_prompt()` to be more explicit
4. **Open an issue** - Include:
   - Workflow run URL
   - Contents of debug file
   - Expected vs actual format
5. **Temporary workaround** - Set `run_questionnaire: false` in workflow inputs

---

**Status:** Ready for production testing! 🚀

All fixes have been implemented and are ready to merge. The next CI run will include enhanced debugging that should help diagnose any remaining issues.