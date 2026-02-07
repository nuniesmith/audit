# LLM Audit Optimization - Deployment Checklist

**Version:** 2.0  
**Status:** Ready for deployment  
**Estimated Time:** 40 minutes total

---

## ✅ Pre-Deployment Verification

- [ ] Repository has GitHub Actions enabled
- [ ] You have admin access to repository settings
- [ ] Audit CLI builds successfully (`cd src/audit && cargo build`)
- [ ] You have obtained API key(s) for at least one provider:
  - [ ] Google Gemini API key (recommended) - OR -
  - [ ] XAI Grok API key

---

## 📋 Step-by-Step Deployment

### Step 1: Verify Workflow Configuration (2 min)

- [ ] Check workflow file exists: `.github/workflows/llm-audit-enhanced.yml`
- [ ] Verify default settings:
  - [ ] `analysis_depth: "critical"` (line 33)
  - [ ] `max_files_per_batch: "3"` (line 57)
  - [ ] Cost tracking section present (line 645)

**Verification command:**
```bash
grep "default: \"critical\"" .github/workflows/llm-audit-enhanced.yml
grep "default: \"3\"" .github/workflows/llm-audit-enhanced.yml
grep "Track API costs" .github/workflows/llm-audit-enhanced.yml
```

Expected: All three should match ✅

---

### Step 2: Configure API Secrets (5 min)

- [ ] Navigate to: `https://github.com/[YOUR_ORG]/[YOUR_REPO]/settings/secrets/actions`
- [ ] Click "New repository secret"
- [ ] Add Google Gemini key:
  - [ ] Name: `GOOGLE_API_KEY`
  - [ ] Value: [your API key]
  - [ ] Click "Add secret"
- [ ] (Optional) Add XAI key:
  - [ ] Name: `XAI_API_KEY`
  - [ ] Value: [your API key]
  - [ ] Click "Add secret"
- [ ] Verify both secrets appear in the list

---

### Step 3: Run First Test Audit (10 min)

- [ ] Navigate to: `https://github.com/[YOUR_ORG]/[YOUR_REPO]/actions`
- [ ] Click on "🤖 Enhanced LLM Audit" workflow
- [ ] Click "Run workflow" button (top right)
- [ ] Configure test run:
  - [ ] LLM Provider: `google`
  - [ ] Analysis depth: `critical` (default)
  - [ ] Focus areas: `security,risk-management` (or use default)
  - [ ] Target components: [leave empty]
  - [ ] Include test files: `false`
  - [ ] Files per batch: `3` (default)
  - [ ] Enable deep context: `false`
- [ ] Click green "Run workflow" button
- [ ] Wait for workflow to complete (5-8 minutes)
- [ ] Monitor both jobs:
  - [ ] ✅ `prepare-context` job completed
  - [ ] ✅ `llm-deep-analysis` job completed

---

### Step 4: Review First Results (5 min)

- [ ] Click on completed workflow run
- [ ] Review job summary sections:
  - [ ] 📋 Context Bundle Created
  - [ ] 🎯 Analysis Configuration
  - [ ] 🔍 Findings Overview
  - [ ] 💰 API Cost Analysis
- [ ] Note the estimated cost (should be $0.01-0.10)
- [ ] Check artifacts section at bottom of page:
  - [ ] `llm-audit-results-[...]` artifact exists
  - [ ] `cost-tracking-[...]` artifact exists
- [ ] Download both artifacts
- [ ] Extract artifacts to local machine

---

### Step 5: Analyze First Run Costs (5 min)

- [ ] Extract `cost-tracking-[RUN_NUMBER]` artifact
- [ ] Copy extracted files to `src/audit/cost-tracking/`
- [ ] Make analysis script executable:
  ```bash
  chmod +x src/audit/scripts/analyze-costs.sh
  ```
- [ ] Run cost analysis:
  ```bash
  cd src/audit
  ./scripts/analyze-costs.sh ./cost-tracking
  ```
- [ ] Review generated report:
  ```bash
  cat COST_ANALYSIS_REPORT.md
  ```
- [ ] Verify key metrics:
  - [ ] Total runs: 1
  - [ ] Estimated cost: $0.02-0.10
  - [ ] Tokens used: [reasonable number]
  - [ ] Files analyzed: ~180 (for critical depth)

---

### Step 6: Review Audit Results (5 min)

- [ ] Extract `llm-audit-results-[...]` artifact
- [ ] Review key files:
  - [ ] `COMPREHENSIVE_AUDIT_REPORT.md` - Executive summary
  - [ ] `llm-audit-report.md` - Detailed findings
  - [ ] `llm-tasks.csv` - Actionable tasks (import to issue tracker)
  - [ ] `llm-tasks.json` - Tasks in JSON format
  - [ ] `llm-audit-full.json` - Complete raw results
- [ ] Count findings by severity:
  - [ ] CRITICAL: [count]
  - [ ] HIGH: [count]
  - [ ] MEDIUM: [count]
  - [ ] LOW: [count]
- [ ] Spot-check 5 random findings for quality
- [ ] Estimate false positive rate (should be <15%)

---

### Step 7: Share with Team (3 min)

- [ ] Share quick reference with developers:
  ```bash
  cat src/audit/LLM_AUDIT_QUICK_REF.md
  ```
- [ ] Send summary email/message to team:
  - [ ] LLM audit is now available
  - [ ] Cost: ~$0.02 per run (weekly audits = $0.08/month)
  - [ ] Results available in GitHub Actions artifacts
  - [ ] Link to quick reference guide
- [ ] Schedule first team review session

---

### Step 8: Set Up Regular Audits (5 min)

**Option A: Manual Triggers (Recommended to start)**
- [ ] Document weekly process:
  - [ ] Every Monday at 10am: Run workflow via GitHub Actions
  - [ ] Review results same day
  - [ ] Generate tasks from critical/high findings
- [ ] Add to team calendar

**Option B: Scheduled Runs (Advanced)**
- [ ] Edit `.github/workflows/llm-audit-enhanced.yml`
- [ ] Add schedule trigger:
  ```yaml
  on:
    workflow_dispatch:
      # ... existing inputs ...
    
    schedule:
      # Weekly audit every Monday at 2 AM UTC
      - cron: '0 2 * * 1'
  ```
- [ ] Commit and push changes
- [ ] Verify schedule appears in Actions tab

**Option C: CI Integration (Advanced)**
- [ ] Review `CI_LLM_AUDIT_REVIEW.md` for integration patterns
- [ ] Add conditional trigger to `ci.yml`
- [ ] Test with PR that has critical issues

---

## 📊 Success Validation

### After First Week
- [ ] Run at least 2 test audits (different depths/providers)
- [ ] Cost analysis report generated successfully
- [ ] Average cost per run: $0.02-0.10 (for critical/standard)
- [ ] Team familiar with downloading and reviewing artifacts
- [ ] At least 5 issues identified and triaged

### After First Month
- [ ] 4+ routine audits completed
- [ ] Cost tracking data for all runs available
- [ ] Monthly cost analysis run and reviewed
- [ ] Total monthly cost: <$5
- [ ] 10+ critical issues identified and addressed
- [ ] Team comfortable with workflow

### After First Quarter
- [ ] Regular audit cadence established
- [ ] Pre-release deep audits integrated
- [ ] Cost trends analyzed and optimized
- [ ] ROI demonstrated (bugs prevented)
- [ ] Documentation updated with learnings

---

## 📈 Ongoing Operations

### Weekly Tasks (5 min)
- [ ] Run routine audit (or verify scheduled run completed)
- [ ] Download cost-tracking artifact
- [ ] Run `analyze-costs.sh`
- [ ] Review findings and generate tasks
- [ ] Address critical/high priority items

### Monthly Tasks (15 min)
- [ ] Aggregate all cost-tracking data
- [ ] Generate comprehensive cost report
- [ ] Review trends:
  - [ ] Are costs increasing? (codebase growth)
  - [ ] Are we using optimal settings?
  - [ ] Should we adjust frequency?
- [ ] Update team on findings and ROI

### Quarterly Tasks (30 min)
- [ ] Deep dive cost/quality analysis
- [ ] Test alternative configurations:
  - [ ] Try different provider (compare XAI vs Google)
  - [ ] Try different depths
  - [ ] Test different batch sizes
- [ ] Review and update documentation
- [ ] Present ROI to stakeholders
- [ ] Plan next quarter optimizations

---

## 🎯 Configuration Quick Reference

### Recommended Configurations

**Weekly Routine Audit:**
```yaml
Provider: google
Depth: critical
Focus: security,risk-management,logic
Batch: 3
Cost: ~$0.02/run
```

**Pre-Release Deep Audit:**
```yaml
Provider: google
Depth: deep
Focus: security,risk-management,logic,performance
Batch: 3
Deep context: true
Include tests: true
Cost: ~$0.30/run
```

**Emergency Investigation:**
```yaml
Provider: xai
Depth: deep
Target: [specific components]
Batch: 2
Deep context: true
Cost: ~$1.00/run
```

---

## 🚨 Troubleshooting

### Issue: Workflow fails with "API key not configured"
- [ ] Verify secret exists in repo settings
- [ ] Check secret name is exact: `GOOGLE_API_KEY` or `XAI_API_KEY`
- [ ] Try re-creating the secret
- [ ] Re-run workflow

### Issue: Cost tracking data not found
- [ ] Check workflow completed successfully
- [ ] Verify artifact was uploaded (bottom of workflow page)
- [ ] Download artifact manually
- [ ] Extract to correct directory: `src/audit/cost-tracking/`
- [ ] Ensure JSON files are present

### Issue: Analysis takes too long (>15 min)
- [ ] Switch to `critical` or `quick` depth
- [ ] Increase batch size to 5
- [ ] Disable deep context
- [ ] Add target components to limit scope

### Issue: Too many false positives
- [ ] Reduce batch size to 2
- [ ] Enable deep context
- [ ] Use more specific focus areas
- [ ] Try XAI provider for better quality

### Issue: Costs higher than expected
- [ ] Verify you're using Google Gemini (not XAI)
- [ ] Check depth is set to `critical` (not `deep`)
- [ ] Review cost analysis report for trends
- [ ] Consider reducing audit frequency

---

## 📚 Documentation Reference

- **LLM_AUDIT_DEPLOYMENT_GUIDE.md** - Full 40-minute deployment guide
- **LLM_AUDIT_QUICK_REF.md** - Quick reference for team
- **OPTIMIZATION_COMPLETE.md** - Implementation details
- **OPTIMIZATION_EXECUTIVE_SUMMARY.md** - Executive summary
- **LLM_AUDIT_OPTIMIZATION_GUIDE.md** - Advanced optimization
- **CI_LLM_AUDIT_REVIEW.md** - CI integration patterns

---

## ✅ Final Sign-Off

Deployment is complete when ALL of the following are true:

- [ ] API secrets configured
- [ ] First test audit successful
- [ ] Cost tracking working
- [ ] analyze-costs.sh generates report
- [ ] Team has access to quick reference
- [ ] Regular audit schedule established
- [ ] Results review process documented
- [ ] Monthly cost review scheduled

**Completed by:** ________________  
**Date:** ________________  
**Initial monthly cost estimate:** $________________  

---

## 🎉 You're Live!

Congratulations! Your LLM audit system is now operational with:

✅ 60-80% cost savings (critical depth)  
✅ Full cost visibility (tracking + analysis)  
✅ 2x better insights (batch size 3)  
✅ Production-ready workflow

**Next audit:** Run weekly routine audit next Monday

**Monthly review:** First Friday of each month

**Questions?** See documentation in `src/audit/`

---

**Happy Auditing! 🚀**