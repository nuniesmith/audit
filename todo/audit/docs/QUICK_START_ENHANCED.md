# Quick Start: Enhanced Audit Features

This guide helps you get started with the new TODO scanning and LLM analysis features.

## Prerequisites

```bash
# Set your API key (choose one)
export XAI_API_KEY=your_xai_key_here      # For Grok (recommended)
export GOOGLE_API_KEY=your_google_key     # For Gemini
```

## Quick Commands

### 1. Find All TODOs in Your Codebase

```bash
cd src/audit
cargo run --release --bin audit-cli -- todo ../../src/janus
```

**Output:**
```
📝 TODO Items Found: 45
═══════════════════════════════════════
🔴 High Priority:   12
🟡 Medium Priority: 28
🟢 Low Priority:    5
📁 Files with TODOs: 23

🔴 High Priority TODOs:
  1. [src/brain.rs:278] Integrate with alerting system
  2. [gateway/janus_client.py:7] Generate Python gRPC stubs
  ...
```

### 2. Filter High-Priority TODOs Only

```bash
cargo run --release --bin audit-cli -- todo ../../src/janus \
    --priority high \
    --output high-priority.json
```

### 3. Rate Files with AI

```bash
# Rate a single critical file
cargo run --release --bin audit-cli -- rate \
    ../../src/janus/crates/cns/src/brain.rs \
    --provider xai

# Batch rate multiple files (faster)
cargo run --release --bin audit-cli -- rate \
    ../../src/janus/services/gateway \
    --provider xai \
    --batch \
    --output ratings.json
```

### 4. Run LLM Questionnaire (Full Audit)

```bash
cargo run --release --bin audit-cli -- question \
    ../../src/janus \
    --provider xai \
    --output audit-results.json
```

**This will:**
- Analyze each file for reachability
- Check for compliance issues
- Identify incomplete code
- Suggest appropriate audit tags
- Provide improvement recommendations

## CI/CD Usage

### Manual Workflow Trigger

1. Go to GitHub Actions → "🤖 LLM Audit"
2. Click "Run workflow"
3. Configure options:
   - **Mode**: `full` for comprehensive audit
   - **LLM Provider**: `xai` (Grok) or `google` (Gemini)
   - **Scan TODOs**: ✅ (enabled by default)
   - **Run Questionnaire**: ✅ (for full analysis)
   - **Max Files**: `0` (analyze all)
   - **Focus Area**: Leave empty for all areas

### Standard vs Full Mode

**Standard Mode** (Quick, ~5 min):
- Static analysis
- TODO scanning
- Basic LLM analysis
- Best for: PR reviews, daily checks

**Full Mode** (Comprehensive, ~15 min):
- Everything in Standard
- File-by-file questionnaire
- Dead code detection
- Architecture review
- Best for: Weekly audits, releases

## Output Formats

All commands support three formats:

```bash
# Human-readable (default)
cargo run --bin audit-cli -- todo . --format text

# JSON for tooling
cargo run --bin audit-cli -- todo . --format json --output todos.json

# CSV for spreadsheets
cargo run --bin audit-cli -- todo . --format csv --output todos.csv
```

## Real-World Examples

### Example 1: Pre-Release Checklist

```bash
# 1. Find all high-priority TODOs
cargo run --release --bin audit-cli -- todo ./src \
    --priority high --output release-todos.json

# 2. Rate security-critical files
cargo run --release --bin audit-cli -- rate ./src/auth \
    --provider xai --output security-ratings.json

# 3. Run full questionnaire
cargo run --release --bin audit-cli -- question ./src \
    --provider xai --output full-audit.json

# 4. Review results
echo "TODOs: $(cat release-todos.json | jq 'length')"
echo "Files rated: $(cat security-ratings.json | jq 'length')"
echo "Unreachable files: $(cat full-audit.json | jq '[.[] | select(.reachable == false)] | length')"
```

### Example 2: Daily PR Check

```bash
# Get changed files
git diff --name-only main...HEAD > changed_files.txt

# Scan only changed files for TODOs
while read file; do
    if [ -f "$file" ]; then
        cargo run --release --bin audit-cli -- todo "$file"
    fi
done < changed_files.txt
```

### Example 3: Weekly Audit Report

```bash
#!/bin/bash
# weekly-audit.sh

echo "Running weekly audit..."

# 1. Scan TODOs
cargo run --release --bin audit-cli -- todo ./src \
    --output weekly-todos.json --format json

# 2. Run questionnaire
cargo run --release --bin audit-cli -- question ./src \
    --provider xai --output weekly-audit.json

# 3. Generate summary
TODO_HIGH=$(cat weekly-todos.json | jq '[.[] | select(.priority == "High")] | length')
UNREACHABLE=$(cat weekly-audit.json | jq '[.[] | select(.reachable == false)] | length')
INCOMPLETE=$(cat weekly-audit.json | jq '[.[] | select(.incomplete == true)] | length')

echo "Weekly Audit Summary"
echo "===================="
echo "High-priority TODOs: $TODO_HIGH"
echo "Unreachable files: $UNREACHABLE"
echo "Incomplete files: $INCOMPLETE"

# 4. Upload to your tracking system
# curl -X POST https://your-tracker.com/api/audit ...
```

## Interpreting Results

### TODO Priority Levels

- **🔴 High**: FIXME, XXX, security/urgent keywords → Address ASAP
- **🟡 Medium**: Standard TODOs → Schedule for next sprint
- **🟢 Low**: NOTE, "maybe", "consider" → Backlog items

### LLM Security Ratings

- **Critical**: Immediate action required
- **High**: Address before next release
- **Medium**: Fix in upcoming sprints
- **Low**: Technical debt, no immediate risk
- **Info**: General observations

### Questionnaire Flags

- **Unreachable = false**: Dead code candidate → Consider removal
- **Incomplete = true**: Missing implementation → Add to backlog
- **Compliance Issues > 0**: Regulatory concern → Priority fix

## Cost Optimization

### Token Usage (approximate)

- TODO scan: FREE (no API calls)
- Static analysis: FREE (no API calls)
- File rating (individual): ~200 tokens/file
- File rating (batch): ~50 tokens/file
- Questionnaire: ~100 tokens/file
- Full CI audit: ~12,000-16,000 tokens (~$0.15-0.20)

### Tips to Save Costs

1. Use static analysis and TODO scans first (free)
2. Use batch mode for file rating
3. Limit questionnaire to critical directories
4. Run full audits weekly, not daily
5. Use `max_files` parameter to limit scope

## Troubleshooting

### "No API key found"

```bash
# Make sure you've exported the key
export XAI_API_KEY=your_key_here

# Verify it's set
echo $XAI_API_KEY
```

### "No TODOs found"

Check:
- File extensions are supported (.rs, .py, .kt, .ts, .js)
- TODO comments use correct format: `// TODO:` not `//TODO`
- You're not in an excluded directory (target/, node_modules/)

### "Rate limit exceeded"

- Wait a few minutes
- Switch providers: `--provider google` instead of `--provider xai`
- Reduce scope with `max_files` or focus on specific directories

### Build errors

```bash
# Clean and rebuild
cd src/audit
cargo clean
cargo build --release
```

## Next Steps

1. **Read the full documentation**: [ENHANCED_AUDIT_FEATURES.md](./ENHANCED_AUDIT_FEATURES.md)
2. **Check examples**: See [examples/](./examples/) directory
3. **Integrate with your CI**: Add TODO checks to your pipeline
4. **Schedule regular audits**: Weekly full mode, daily standard mode

## Tips for Better Results

### Writing Good TODOs

```rust
// ✅ Good - Clear, actionable, with context
// TODO: Implement rate limiting for API endpoints (see issue #123)
// FIXME: Security vulnerability in JWT validation - urgent!
// NOTE: Consider caching this result in future optimization pass

// ❌ Avoid - Vague or incomplete
// TODO: fix this
// todo
```

### Getting Better AI Ratings

1. Add clear comments explaining complex logic
2. Include docstrings for functions
3. Use descriptive variable names
4. Organize code into logical modules

### Effective Questionnaire Use

- Run on directories, not individual files (better context)
- Review "unreachable" files - candidates for removal
- Track "incomplete" files - work not yet done
- Use suggested tags to improve organization

## Support

- **Issues**: Open a GitHub issue
- **Docs**: Check the main [README.md](./README.md)
- **CI Logs**: Review workflow runs in `.github/workflows/llm-audit.yml`

---

**Happy auditing!** 🚀