# LLM Audit Quick Reference Card

**Version:** 2.0 | **Updated:** 2024

---

## 🚀 Quick Start

```bash
# Run via GitHub Actions:
# Actions → 🤖 Enhanced LLM Audit → Run workflow

# Recommended first run:
Provider: google
Depth: critical
Focus: security,risk-management
Batch: 3
```

---

## 💰 Cost Cheat Sheet (Updated Dec 2024)

| Configuration | Cost/Run | Use Case |
|---------------|----------|----------|
| **google + critical** | $0.09 | ✅ Weekly audits (CHEAPEST) |
| **xai + critical** | $0.16 | ✅ Weekly with reasoning (RECOMMENDED) |
| **google + standard** | $0.15 | Monthly comprehensive |
| **google + deep** | $0.25 | Pre-release review |
| **xai + deep** | $0.50 | Pre-release with reasoning |

**💡 Major Update:** XAI Grok 4.1 Fast is now 93% cheaper! Now competitive for regular use!
**💡 Savings:** `critical` vs `deep` = **60-80% cost reduction**

---

## 🎯 Analysis Depths Explained

### `critical` ⭐ RECOMMENDED DEFAULT
- **Analyzes:** ~180 critical files (kill_switch, conscience, circuit_breaker, risk management)
- **Cost:** 💰 (cheapest)
- **Time:** ⏱️ 3-5 min
- **Use for:** Weekly routine audits, continuous monitoring
- **Savings:** 60-80% vs deep

### `quick`
- **Analyzes:** ~100 essential files only
- **Cost:** 💰 (very cheap)
- **Time:** ⏱️ 2-3 min
- **Use for:** Fast triage, emergency scans

### `standard`
- **Analyzes:** All ~550 files, standard detail
- **Cost:** 💰💰 (moderate)
- **Time:** ⏱️ 5-8 min
- **Use for:** Bi-weekly comprehensive review

### `deep`
- **Analyzes:** All ~550 files, maximum detail
- **Cost:** 💰💰💰 (expensive)
- **Time:** ⏱️ 8-12 min
- **Use for:** Pre-release, major refactoring

---

## 🏢 Provider Selection

### Google Gemini ⭐ CHEAPEST
- **Model:** `gemini-2.0-flash-exp`
- **Cost:** $0.075/1M input, $0.30/1M output
- **Context:** 1M tokens
- **Pros:** Cheapest option, fast, good quality
- **Cons:** Smaller context than XAI
- **Best for:** Cost-optimized regular audits

### XAI Grok 4.1 Fast ⭐ RECOMMENDED FOR QUALITY
- **Model:** `grok-4-1-fast-reasoning`
- **Cost:** $0.20/1M input, $0.50/1M output (93% cheaper than old grok-4-1-fast-reasoning!)
- **Context:** 2M tokens (2x larger than Gemini!)
- **Pros:** Reasoning capability, huge context, better logic analysis
- **Cons:** 75% more expensive than Gemini (but still very affordable)
- **Best for:** Complex logic analysis, pre-release audits, investigations

---

## 📊 Batch Size Guide

| Batch Size | Detail | Speed | Cost | Use Case |
|------------|--------|-------|------|----------|
| **2** | ⭐⭐⭐⭐⭐ | Slow | 💰💰💰 | Maximum detail |
| **3** ⭐ | ⭐⭐⭐⭐ | Fast | 💰💰 | **OPTIMAL** |
| **5** | ⭐⭐⭐ | Faster | 💰 | Quick scan |
| **10** | ⭐⭐ | Fastest | 💰 | Surface scan |

**Recommendation:** Batch size `3` = best balance of detail and cost

---

## 🎯 Usage Patterns

### Weekly Routine - Cost Optimized
```yaml
Provider: google
Depth: critical
Focus: security,risk-management,logic
Batch: 3
Deep context: false
Include tests: false
```
**Cost:** ~$0.09/run | **Frequency:** Weekly

### Weekly Routine - Quality Optimized ⭐ NEW RECOMMENDATION
```yaml
Provider: xai
Depth: critical
Focus: security,risk-management,logic
Batch: 3
Deep context: false
Include tests: false
```
**Cost:** ~$0.16/run | **Frequency:** Weekly
**Why:** Reasoning capability worth the 75% premium

---

### Pre-Release Comprehensive
```yaml
Provider: xai
Depth: deep
Focus: security,risk-management,logic,performance
Batch: 3
Deep context: true
Include tests: true
```
**Cost:** ~$0.50/run | **Frequency:** Before releases
**Why:** Reasoning + 2M context for critical decisions

---

### Emergency Investigation
```yaml
Provider: xai
Depth: deep
Focus: security,risk-management
Target: [specific components]
Batch: 2
Deep context: true
```
**Cost:** ~$0.50/run | **Frequency:** As needed
**Why:** Best reasoning for root cause analysis

---

## 📈 Cost Tracking

### View costs after each run:
```bash
# Download cost-tracking artifact from GitHub Actions
# Extract to src/audit/cost-tracking/

cd src/audit
./scripts/analyze-costs.sh ./cost-tracking
cat COST_ANALYSIS_REPORT.md
```

### Key metrics in report:
- Total cost to date
- Average cost per run
- Cost by provider
- Cost by depth
- Monthly projections
- Optimization recommendations

---

## 💡 Optimization Tips

### Reduce Costs by 60-80%
1. ✅ Use `critical` depth for regular runs
2. ✅ Use Google Gemini for absolute cheapest ($0.09/run)
3. ✅ Keep batch size at 3
4. ✅ Focus on 2-3 areas at a time

### Get Best Quality (Still Affordable!)
1. ✅ Use XAI Grok 4.1 Fast Reasoning ($0.16/run)
2. ✅ Leverage 2M context window with larger batches
3. ✅ Use reasoning mode for complex logic
4. ✅ Still 93% cheaper than old XAI pricing!

### Improve Quality
1. ✅ Reduce batch size to 2-3
2. ✅ Enable deep context for complex analysis
3. ✅ Use specific focus areas
4. ✅ Include tests for comprehensive review

### Speed Up Analysis
1. ✅ Use `quick` or `critical` depth
2. ✅ Increase batch size to 5-10
3. ✅ Limit target components
4. ✅ Disable deep context

---

## 🔍 Focus Areas

**Common combinations:**

| Focus Areas | Use Case |
|-------------|----------|
| `security,risk-management` | Security audit |
| `security,risk,logic` | Comprehensive safety |
| `logic,performance` | Code quality review |
| `security,risk,logic,performance` | Full audit (expensive) |

**Available areas:**
- `security` - Security vulnerabilities, auth, crypto
- `risk-management` - Risk calculation, position management
- `logic` - Business logic, algorithms, edge cases
- `performance` - Optimization, bottlenecks, efficiency
- `error-handling` - Error paths, recovery, resilience
- `testing` - Test coverage, test quality

---

## 📊 Interpreting Results

### Artifacts Downloaded
1. **llm-audit-full.json** - Complete analysis results
2. **llm-audit-report.md** - Summary report
3. **llm-tasks.json** - Generated tasks (JSON)
4. **llm-tasks.csv** - Generated tasks (CSV for import)
5. **COMPREHENSIVE_AUDIT_REPORT.md** - Executive summary
6. **cost-tracking/** - Cost data

### Priority Levels
- **CRITICAL** - Address immediately (security, safety)
- **HIGH** - Fix within 1 week (risk, logic errors)
- **MEDIUM** - Plan for next sprint (improvements)
- **LOW** - Backlog (nice-to-haves)

---

## 🚨 Common Issues

### "API key not configured"
→ Add `XAI_API_KEY` or `GOOGLE_API_KEY` in GitHub repo secrets

### Cost higher than expected
→ Switch to `google` provider + `critical` depth

### Too many false positives
→ Reduce batch size to 2, enable deep context

### Analysis too slow
→ Use `critical` depth, increase batch size to 5

### Not finding real issues
→ Reduce batch size to 2, use `deep` depth, enable deep context

---

## 📅 Recommended Schedule

| Frequency | Configuration | Monthly Cost |
|-----------|---------------|--------------|
| **Weekly (cheap)** | google + critical | ~$0.36 (4 runs) |
| **Weekly (quality)** | xai + critical | ~$0.64 (4 runs) |
| **Pre-release** | xai + deep | ~$0.50 (1 run) |
| **Total recommended** | Mix both | ~$0.86-1.14/month |

**Recommended budget:** $1-2/month for comprehensive coverage (both providers!)

---

## ✅ Best Practices Checklist

- [ ] Start with `google` + `critical` for first run
- [ ] Review cost analysis after each run
- [ ] Use `critical` for weekly routine audits
- [ ] Reserve `deep` for pre-release only
- [ ] Track costs monthly with analyze-costs.sh
- [ ] Focus on 2-3 areas per run
- [ ] Keep batch size at 3 (optimal)
- [ ] Download and review all artifacts
- [ ] Generate tasks from findings
- [ ] Re-run after fixes to verify

---

## 🔗 Related Documentation

- **LLM_AUDIT_DEPLOYMENT_GUIDE.md** - Full deployment steps
- **LLM_AUDIT_OPTIMIZATION_GUIDE.md** - Detailed optimization strategies
- **CI_LLM_AUDIT_REVIEW.md** - CI integration patterns
- **COST_ANALYSIS_REPORT.md** - Generated by analyze-costs.sh

---

## 📞 Quick Commands

```bash
# Run cost analysis
cd src/audit && ./scripts/analyze-costs.sh ./cost-tracking

# View latest report
cat src/audit/COST_ANALYSIS_REPORT.md

# Build audit CLI locally
cd src/audit && cargo build --release

# Run local audit
cd src/audit && ./run.sh
```

---

**🎯 TL;DR:** 
- **Cheapest:** `google` + `critical` + batch `3` = $0.09/run
- **Best quality:** `xai` + `critical` + batch `3` = $0.16/run (reasoning + 2M context!)
- **Major news:** XAI is now 93% cheaper - viable for regular use! 🚀