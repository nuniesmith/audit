# Audit Service Quick Reference Card

## 🚀 Quick Commands

### TODO Scanning (FREE)
```bash
# Find all TODOs
audit-cli todo .

# High priority only
audit-cli todo . --priority high

# Export to JSON
audit-cli todo . --output todos.json
```

### LLM File Rating (Paid)
```bash
# Rate single file
audit-cli rate file.rs --provider xai

# Rate directory (batch mode - faster)
audit-cli rate ./src --batch --provider xai

# Export results
audit-cli rate ./src --output ratings.json
```

### LLM Questionnaire (Paid)
```bash
# Full audit with standardized questions
audit-cli question ./src --provider xai --output audit.json
```

### Traditional Commands
```bash
audit-cli audit .           # Full audit
audit-cli tags .            # Scan audit tags
audit-cli static .          # Static analysis only
audit-cli tasks .           # Generate tasks
audit-cli stats .           # Show statistics
```

## 📊 Output Formats

Add `--format` flag to any command:
- `--format text` - Console output (default)
- `--format json` - Machine-readable
- `--format csv` - Spreadsheet-ready

## 🔑 API Keys

```bash
# For xAI Grok (recommended)
export XAI_API_KEY=your_key_here

# For Google Gemini
export GOOGLE_API_KEY=your_key_here
```

## 🤖 CI/CD Modes

| Mode | Use Case | Duration | Cost |
|------|----------|----------|------|
| **Standard** | PR reviews, daily checks | ~5 min | ~$0.15 |
| **Full** | Weekly audits, releases | ~10 min | ~$0.20 |
| **Full + Quest** | Deep dives, major releases | ~20 min | ~$0.30 |

## 📝 TODO Priority Levels

- 🔴 **High** - FIXME, XXX, security, urgent
- 🟡 **Medium** - Standard TODO items
- 🟢 **Low** - NOTE, maybe, consider

## 🎯 LLM Providers

```bash
--provider xai      # Grok (faster, cheaper)
--provider google   # Gemini (alternative)
```

## 📦 CI Workflow Inputs

Trigger manually with:
- **mode**: `standard` or `full`
- **llm_provider**: `xai` or `google`
- **scan_todos**: `true` (recommended)
- **run_questionnaire**: `true` (full mode only)
- **max_files**: `0` (all) or limit
- **focus_area**: `security`, `performance`, etc.

## 💰 Cost Optimization

1. ✅ Use TODO scan first (FREE)
2. ✅ Use batch mode for multiple files
3. ✅ Limit scope with max_files
4. ✅ Run full audits weekly, not daily
5. ✅ Use static analysis before LLM

## 🔍 Common Patterns

### Pre-Release Check
```bash
audit-cli todo ./src --priority high
audit-cli rate ./src/critical --provider xai
audit-cli question ./src --provider xai
```

### Daily PR Check
```bash
git diff --name-only main...HEAD | \
  xargs audit-cli todo
```

### Security Audit
```bash
audit-cli static ./src --focus security
audit-cli rate ./src --provider xai --batch
```

## 📈 Interpreting Results

### TODO Scanner
- Check `high_priority` count
- Review files_with_todos
- Address urgent items first

### File Ratings
- **Critical/High** → Fix before release
- **Medium** → Schedule for sprint
- **Low/Info** → Technical debt

### Questionnaire
- `reachable: false` → Dead code
- `incomplete: true` → Unfinished work
- `compliance_issues > 0` → Priority fix

## 🐛 Troubleshooting

| Issue | Solution |
|-------|----------|
| "No API key found" | `export XAI_API_KEY=...` |
| "No TODOs found" | Check format: `// TODO:` |
| "Rate limit exceeded" | Switch provider or wait |
| Build errors | `cargo clean && cargo build` |

## 📚 Documentation

- **Full Docs**: `ENHANCED_AUDIT_FEATURES.md`
- **Quick Start**: `QUICK_START_ENHANCED.md`
- **Summary**: `AUDIT_ENHANCEMENTS_SUMMARY.md`

## ⚡ Pro Tips

1. 🎯 TODO scan is instant and free - use it often
2. 🚀 Batch mode is 4x faster for multiple files
3. 💡 Run questionnaire on directories, not files
4. 📊 Export to JSON/CSV for tracking trends
5. 🔄 Automate with cron or CI/CD

---

**Quick Help**: `audit-cli --help` or `audit-cli <command> --help`
