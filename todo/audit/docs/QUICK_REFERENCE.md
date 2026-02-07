# Audit Service Quick Reference Card

> **TL;DR:** API-only audit service with automated static analysis + manual LLM-powered deep audits

## 🎯 Three Ways to Use Audit

### 1. Automated (Already Running) ✅
**Every push to `main`/`develop` = automatic static audit**
- Includes test files for comprehensive analysis
- Check **Actions** tab for results
- Download artifacts: `tags-report.txt`, `static-report.txt`, `tasks.json`
- No setup needed, already configured

### 2. Manual LLM Audit (NEW!) 🤖
**Deep AI analysis triggered from GitHub**
1. Go to **Actions** → **🤖 LLM Audit**
2. Click **Run workflow**
3. Pick provider: `xai` (Grok) or `google` (Gemini)
4. Uses all focus areas and includes tests by default
5. Wait ~15-30 min
6. Download artifacts

**Requirements:** GitHub secrets `XAI_API_KEY` or `GOOGLE_API_KEY`

### 3. API Server (Custom Integration) 🌐
```bash
cd src/audit
export XAI_API_KEY="xai-..."
cargo run --release --bin audit-server
# Server: http://localhost:8080
```

---

## 🔑 Setup (One-Time)

### Add GitHub Secrets
**Settings** → **Secrets and variables** → **Actions** → **New repository secret**

```
Name: XAI_API_KEY
Value: xai-xxxxxxxxxxxxxxxx

Name: GOOGLE_API_KEY  
Value: AIzaxxxxxxxxxxxxxxxx
```

*You only need one, depending on which LLM you prefer*

### Local Environment
```bash
# .env file in src/audit/
LLM_PROVIDER=xai              # or: google
XAI_API_KEY=xai-xxx           # or GOOGLE_API_KEY=AIza-xxx
LLM_ENABLED=true
```

---

## ⚡ Quick Commands

### CLI (Local)
```bash
cd src/audit
cargo build --release

# Scan tags
./target/release/audit-cli tags ../..

# Static analysis (fast, no LLM, includes tests)
./target/release/audit-cli static ../..

# Full audit with LLM (includes tests by default)
export XAI_API_KEY="xai-..."
./target/release/audit-cli audit ../.. --llm

# Exclude tests if needed
./target/release/audit-cli audit ../.. --llm --exclude-tests

# Generate tasks
./target/release/audit-cli tasks ../.. --format csv --output tasks.csv
```

### API (curl)
```bash
# Health check
curl http://localhost:8080/health

# Scan tags
curl -X POST http://localhost:8080/api/scan/tags \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/repo"}'

# Static analysis
curl -X POST http://localhost:8080/api/scan/static \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/repo"}'

# Full audit with LLM (includes tests by default)
curl -X POST http://localhost:8080/api/audit \
  -H "Content-Type: application/json" \
  -d '{
    "repository": "/path/to/repo",
    "enable_llm": true,
    "focus": ["security", "logic", "performance", "compliance", "architecture"],
    "include_tests": true
  }'
```

---

## 🤖 LLM Providers

### XAI Grok
```bash
LLM_PROVIDER=xai
XAI_API_KEY=xai-xxx
LLM_MODEL=grok-4-1-fast-reasoning
```
- **Best for:** Production
- **Free tier:** Limited
- **API:** console.x.ai

### Google Gemini
```bash
LLM_PROVIDER=google
GOOGLE_API_KEY=AIza-xxx
LLM_MODEL=gemini-2.0-flash-exp
```
- **Best for:** Testing/Development
- **Free tier:** 1,500 requests/day 🎉
- **API:** aistudio.google.com

---

## 📊 What Each Audit Type Does

| Type | Speed | Cost | What It Checks |
|------|-------|------|----------------|
| **Static** | ~3 min | Free | Patterns, tags, common issues (+ tests) |
| **LLM Standard** | ~15 min | $ | All areas: security, logic, performance, compliance, architecture (+ tests) |
| **LLM Deep** | ~30 min | $$ | Everything + deep context (+ tests) |

---

## 🎯 When to Use What

### Daily: Automated Static
- ✅ Every commit
- ✅ Fast feedback
- ✅ No cost
- ✅ Catches common issues

### Weekly: LLM Standard
- 🤖 On `main` branch
- 🤖 All focus areas
- 🤖 Includes test files
- 🤖 After sprint
- 🤖 Before deployment

### Release: LLM Deep
- 🚀 Before major release
- 🚀 Complete security audit
- 🚀 All areas + tests
- 🚀 After incidents

---

## 🔧 Troubleshooting

### "API key not found"
```bash
# Check provider matches key
LLM_PROVIDER=xai → needs XAI_API_KEY
LLM_PROVIDER=google → needs GOOGLE_API_KEY
```

### "Workflow failed"
1. Check **Actions** tab for logs
2. Verify GitHub secrets are set
3. Check provider/model combination

### "Wrong API format"
```bash
# Provider and model must match!
xai → grok-4-1-fast-reasoning
google → gemini-2.0-flash-exp
```

### "Can't access web UI"
**There is no web UI!** This is API-only.
- Use API endpoints
- Use CLI tool
- Use GitHub Actions

---

## 📚 Full Docs

- **[LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md)** - Complete guide (738 lines)
- **[MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)** - Migration from web UI
- **[README.md](./README.md)** - Full documentation
- **[QUICK_START.md](./QUICK_START.md)** - Getting started

---

## ⚡ Super Quick Start

```bash
# 1. Add secrets to GitHub (one-time)
# Settings → Secrets → Add XAI_API_KEY or GOOGLE_API_KEY

# 2. Run LLM audit
# Actions → 🤖 LLM Audit → Run workflow → Choose provider

# 3. Download results
# Artifacts: llm-audit-report.txt, llm-tasks.csv

# Done! 🎉
```

---

## 💡 Pro Tips

1. **Free tier testing:** Use Google Gemini (1,500/day free)
2. **Cost control:** LLM audits are manual-only (no accidental runs)
3. **Batch size:** 10 is optimal (fewer calls, good context)
4. **Focus areas:** Use all areas by default for comprehensive coverage
5. **Include tests:** Always enabled - tests provide crucial context for logic validation

---

## 🆘 Quick Help

**Static audit not running?**  
→ Check `.github/workflows/ci.yml` job

**LLM audit button missing?**  
→ Check `.github/workflows/llm-audit.yml` exists

**API server won't start?**  
→ Rebuild: `cd src/audit && cargo build --release`

**Questions?**  
→ Open issue or see [LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md)

---

**Version:** 0.1.0 (API-only)  
**Updated:** 2024