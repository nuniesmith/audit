# Changelog - Audit Workflow Fixes

## [2025-12-30] - Critical Fixes

### Fixed
- **CLI Command Names**: Changed `analyze` → `static` and `scan` → `tags` to match actual implementation
- **Argument Syntax**: Changed from `--path <PATH>` to positional argument `<PATH>`
- **JSON Parsing**: Fixed workflow to match actual CLI output structure (tags as array, issues_by_severity object)
- **Curl Payload**: Fixed "Argument list too long" by using `jq` to build JSON file and `--data-binary @file`
- **jq Variable Size**: Fixed "Argument list too long" for jq by using `--rawfile` to read prompt directly from file instead of shell variable
- **YAML Syntax**: Removed unsupported `--format json` flag from `static` command
- **Indentation**: Standardized YAML indentation to 4 spaces

### Changed Files
- `.github/workflows/llm-audit.yml` - All workflow fixes applied

### Testing
- ✅ `audit-cli static` tested successfully (423 files, 137 issues found)
- ✅ `audit-cli tags` tested successfully (0 tags found - expected)
- ✅ JSON output validated and parsing fixed
- ✅ Workflow jq commands tested locally
- ⏳ GitHub Actions workflow pending (requires API keys)

### Documentation Added
- `ISSUE_RESOLUTION.md` - Complete technical summary
- `WORKFLOW_FIXES.md` - Detailed fix documentation
- `FIXES_APPLIED.md` - Quick reference guide
- `CHANGELOG_FIXES.md` - This file

## Commands Now Working

```bash
# Static analysis (was 'analyze')
audit-cli static ../../src/janus --output report.json

# Tag scanning (was 'scan')
audit-cli tags ../../src/janus --output tags.json
```

## Before → After

| Before (Broken) | After (Fixed) |
|----------------|---------------|
| `analyze --path X` | `static X` |
| `scan --path X` | `tags X` |
| `.tags \| length` | `length` (direct array) |
| `.findings[].severity` | `.issues_by_severity.critical` |
| `--argjson prompt "$VAR"` | `--rawfile prompt file.txt` |
| `curl -d "{big json}"` | `jq ... > file && curl --data-binary @file` |

## CLI Output Structures

### Tags Command
```json
[
  {"tag_type": "TODO", "file": "path/to/file.rs", ...},
  ...
]
```
Access: `jq 'length'` for count

### Static Command
```json
{
  "issues_by_severity": {
    "critical": 44,
    "high": 5,
    "medium": 77,
    "low": 11
  },
  ...
}
```
Access: `jq '.issues_by_severity.critical'`

## Key Learning

**Shell Variable Size Limits:**
Even after fixing curl to use `--data-binary @file`, the shell variable itself (`PROMPT_CONTENT=$(cat big-file | jq -Rs .)`) exceeded argument limits when passed to jq. 

**Solution:** Use `jq --rawfile` to read files directly without storing in variables.

## Next Steps
1. Test workflow in GitHub Actions
2. Add audit tags to codebase
3. Verify LLM responses with real API keys