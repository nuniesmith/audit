# Quick Start: Simplified CI Audit

## Run an Audit in 3 Steps

### 1. Go to GitHub Actions
Navigate to: **Actions** → **🤖 CI Audit**

### 2. Click "Run workflow"
Choose your LLM provider:
- **xai** (default) - Grok Beta, fast and cost-effective
- **google** - Gemini 2.0 Flash, experimental

### 3. Wait ~5-8 minutes
That's it! The workflow handles everything automatically.

---

## What Gets Analyzed

### ✅ Always Included (No Options Needed)

- **Static Analysis** - All Rust code scanned for issues
- **Audit Tags** - AUDIT, TODO, FIXME markers found
- **TODO Comments** - Categorized by priority
- **Comprehensive LLM Analysis** - 16K token deep dive
- **File Questionnaire** - Every file evaluated
- **Up to 150 files** - Maximum coverage

### 📊 Analysis Categories

1. **Security** - Vulnerabilities, auth, crypto
2. **Safety** - Memory safety, panics, errors
3. **Reliability** - Error handling, resilience
4. **Performance** - Async, caching, optimization
5. **Code Quality** - Structure, patterns, maintainability

---

## View Results

### Option 1: GitHub Actions Summary
**Fastest** - Click on the workflow run to see:
- Critical findings count
- TODO statistics
- Cost estimation
- Quick overview

### Option 2: Download Artifact
**Complete** - Download `llm-audit-{run_number}` containing:
- `audit-report.md` - Full analysis
- `llm-analysis.json` - Structured findings
- `static-analysis.json` - Static scan results
- `todos.json` - All TODOs found
- `questionnaire-results.json` - File-by-file audit

### Option 3: Repository
**Permanent** - Check `audit-results/{timestamp}/` for:
- All JSON results
- Comprehensive markdown report
- Historical reference

---

## Cost Per Run

**xAI Grok Beta (typical):**
- Input: ~150K tokens × $2.50/1M = **$0.375**
- Output: ~4K tokens × $10.00/1M = **$0.040**
- **Total: ~$0.42 USD**

**Google Gemini (varies):**
- Check Google AI pricing page
- Generally competitive with xAI

---

## What Changed?

### Before (Complex)
- Choose mode: Standard or Full
- Select focus area: Security, Performance, etc.
- Enable/disable TODO scanning
- Enable/disable questionnaire
- Set max files to analyze

### After (Simple)
- Choose provider: xAI or Google
- **Done!**

All features are always enabled. No decisions needed.

---

## Typical Workflow

```bash
# 1. Make code changes
git add src/janus/...
git commit -m "feat: implement new feature"
git push

# 2. Run audit via GitHub UI
# Actions → CI Audit → Run workflow → Select provider

# 3. Review results
# - Check Actions summary for overview
# - Download artifact for details
# - Address critical findings
```

---

## Understanding Results

### Critical Findings (JSON)
```json
{
  "critical_findings": [
    {
      "id": "SEC-001",
      "title": "Unsafe memory access in trading loop",
      "severity": "critical",
      "category": "security",
      "description": "Direct pointer dereference without bounds check...",
      "recommendation": "Add bounds checking before pointer access..."
    }
  ],
  "summary": "Overall assessment of codebase quality..."
}
```

### Severity Levels
- **Critical** (P0) - Fix immediately, blocks production
- **High** (P1) - Fix soon, significant risk
- **Medium** (P2) - Plan to fix, moderate impact
- **Low** (P3) - Nice to have, minor improvement

---

## Troubleshooting

### Workflow Failed?
1. Check GitHub Actions logs for error message
2. Verify API keys are set: `XAI_API_KEY` or `GOOGLE_API_KEY`
3. Check if audit CLI built successfully
4. Review step-by-step summary

### No Results?
1. Ensure `src/janus` directory exists
2. Check that Rust files are present
3. Verify workflow permissions (contents: write)

### Parsing Errors?
1. Download artifact and check raw JSON
2. Look for malformed LLM responses
3. Try different provider (xAI ↔ Google)

---

## Best Practices

### When to Run
- ✅ Before merging PRs
- ✅ After major refactors
- ✅ Weekly security checks
- ✅ Before releases
- ⚠️ Not on every commit (cost adds up)

### How to Use Results
1. **Triage** - Review critical/high findings first
2. **Validate** - LLMs can be wrong, verify issues
3. **Track** - Create issues for real problems
4. **Fix** - Address critical items before release
5. **Learn** - Use recommendations to improve code

### Cost Management
- Run on-demand, not on every push
- Use for significant changes, not typo fixes
- One audit per PR is usually sufficient
- Monitor monthly spend in GitHub billing

---

## Pro Tips

### 🎯 Focus Your Review
The audit analyzes everything, but you should:
1. Start with critical/high severity
2. Check files you modified
3. Review security findings carefully
4. Use TODO list for future work

### 💡 Interpret LLM Output
Remember:
- LLMs suggest, you decide
- False positives happen
- Context matters (they don't always have full picture)
- Use as guidance, not gospel

### 🔄 Iterate and Improve
- Track recurring issues
- Update code patterns based on feedback
- Share findings with team
- Incorporate into code review process

---

## Need Help?

**Documentation:**
- `WORKFLOW_SIMPLIFICATION.md` - What changed and why
- `AUDIT_REVIEW_SUMMARY.md` - Complete review details
- `README.md` - Full audit service documentation

**Quick Checks:**
```bash
# Verify audit CLI works locally
cd src/audit
cargo build --release
cargo run --release --bin audit-cli -- --help

# Test static analysis
cargo run --release --bin audit-cli -- static ../../src/janus

# Test TODO scanning
cargo run --release --bin audit-cli -- todo ../../src/janus
```

---

## Summary

**Old Way:**
- 5 inputs to configure
- Mode selection confusion
- Inconsistent results
- Complex to use

**New Way:**
- 1 input (LLM provider)
- Always comprehensive
- Consistent results
- Simple to use

**Cost:** ~$0.42/run (only $0.02 more than old "standard" mode)

**Time:** ~5-8 minutes

**Value:** Complete code audit with security, safety, performance, and quality analysis

---

**Ready?** Go to Actions → CI Audit → Run workflow! 🚀