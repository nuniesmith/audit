# Unified LLM Audit Workflow - Simple Guide

**One workflow. Five modes. Complete coverage.**

---

## 🎯 Quick Start

Just one workflow to remember: **🤖 LLM Audit**

```bash
GitHub → Actions → 🤖 LLM Audit → Run workflow
```

Pick a mode from the dropdown. That's it!

---

## 📊 Mode Selection Guide

### Quick Mode ⚡ (~5 min, $0.05)
**When to use:** Daily checks, fast triage, PR reviews

**What it does:**
- Analyzes ~50 critical files only
- Focuses on security and safety
- Fast results, essential findings

**Perfect for:**
- Morning standup prep
- Quick sanity checks
- Pre-commit validation

---

### Standard Mode ⭐ (~8 min, $0.11) **RECOMMENDED**
**When to use:** Weekly routine audits, regular monitoring

**What it does:**
- Analyzes ~180 critical files
- Comprehensive security, risk, and logic analysis
- Integrates static audit results
- Uses CI failure context

**Perfect for:**
- Weekly team reviews
- Sprint planning prep
- Regular health checks

---

### Deep Mode 🔍 (~15 min, $0.15)
**When to use:** Pre-release audits, major refactoring

**What it does:**
- 3-Phase analysis (Audit → Plan → Act)
- Analyzes ~250 files with full system context
- System-wide pattern detection
- Strategic roadmap generation
- Actionable task generation

**Perfect for:**
- Pre-deployment comprehensive audit
- Architecture reviews
- Major feature releases
- Quarterly deep dives

---

### Critical Mode 🚨 (~5 min, $0.08)
**When to use:** Emergency investigations, safety focus

**What it does:**
- Analyzes ~100 safety-critical files only
- Kill switch, circuit breaker, conscience systems
- Risk management components
- Security and safety focus

**Perfect for:**
- Production incident investigation
- Safety system validation
- Emergency audits
- Compliance checks

---

### CI-Aware Mode 🤖 (~8 min, $0.12)
**When to use:** CI failures, test issues, automated analysis

**What it does:**
- Analyzes files causing CI failures
- Reviews recent workflow run logs
- Correlates static audit issues with CI problems
- Root cause analysis for failures

**Perfect for:**
- Debugging failing CI/CD
- Test failure investigation
- Automated issue detection
- Post-failure analysis

---

## 🎛️ Additional Options

### LLM Provider
- **XAI Grok 4.1** (default) - Best reasoning, 2M context, $0.20/$0.50 per 1M tokens
- **Google Gemini** - Cheapest option, $0.075/$0.30 per 1M tokens

### Focus Areas
Leave empty for auto-detection based on mode, or specify:
- `security,risk-management,logic` (most common)
- `safety,security` (critical systems)
- `performance,optimization` (speed focus)

### Generate Code
- **Enabled:** LLM generates fix recommendations with code
- **Disabled:** Analysis only (default, safer)

⚠️ **Warning:** Generated code is saved for review but NOT auto-committed for safety

### Create Issues
- **Enabled:** Auto-creates GitHub issues for critical findings
- **Disabled:** You review findings first (default, recommended)

---

## 📥 Understanding Results

After the workflow completes, download the artifact:
`llm-audit-[mode]-[run-number]`

### Key Files

#### 1. `AUDIT_REPORT.md` 📄
**For: Humans**
- Executive summary
- Detailed findings
- Recommendations
- Easy to read and share

#### 2. `AI_AGENT_TASKS.json` 🤖
**For: AI Agents & Automation**
```json
{
  "metadata": { ... },
  "context": {
    "static_audit": [ ... ],
    "ci_failures": [ ... ],
    "audit_tags": [ ... ]
  },
  "analysis": {
    "critical_findings": [ ... ],
    "patterns": [ ... ],
    "action_plan": [ ... ]
  }
}
```

**Use for:**
- Automated remediation
- AI agent task assignment
- Integration with other tools
- Machine-readable processing

#### 3. `tasks.csv` 📋
**For: Project Management**
```csv
ID,Priority,Severity,Category,Title,Effort,Files
TASK-001,P0,CRITICAL,security,...
```

**Import to:**
- GitHub Issues
- Jira
- Linear
- Your task tracker

#### 4. `context-bundle/` 📦
**For: Deep Dive**
- `static-analysis.json` - All static audit findings
- `audit-tags.json` - Developer-added audit tags
- `ci-runs.json` - Recent CI/CD run history
- `failed-job-details.json` - Failed run details (CI-aware mode)
- `source-files.txt` - Analyzed source code
- `system-map.md` - Architecture overview

---

## 💰 Cost Reference

| Mode | Files | Time | Cost | Frequency |
|------|-------|------|------|-----------|
| Quick | ~50 | 5 min | $0.05 | Daily |
| Standard | ~180 | 8 min | $0.11 | Weekly |
| Deep | ~250 | 15 min | $0.15 | Monthly/Pre-release |
| Critical | ~100 | 5 min | $0.08 | As needed |
| CI-Aware | ~180 | 8 min | $0.12 | After CI failures |

**Monthly budget examples:**
- Weekly standard audits: 4 × $0.11 = **$0.44/month**
- Weekly + monthly deep: $0.44 + $0.15 = **$0.59/month**
- Weekly + pre-release deep: ~**$0.60-1.00/month**

---

## 🔄 Integration with Static Audit

The LLM audit **complements** your static audit:

### Static Audit (Existing)
```yaml
# Runs on every push/PR
- Fast linting and static analysis
- Immediate feedback in CI
- Catches basic issues
```

### LLM Audit (New)
```yaml
# Runs weekly or on-demand
- Deep semantic analysis
- System-wide pattern detection
- Strategic recommendations
- Uses static audit results as input
```

### Workflow
```
1. Push code → Static audit runs automatically
   ↓
2. Weekly: Run LLM audit (standard mode)
   - Analyzes static audit findings
   - Detects patterns across issues
   - Creates strategic plan
   ↓
3. Pre-release: Run LLM audit (deep mode)
   - Comprehensive system review
   - Validates all safety systems
   - Generates deployment checklist
```

---

## 🎯 Recommended Cadence

### Daily
- **Quick mode** after major changes
- 5 minutes, $0.05
- Sanity check

### Weekly (Monday morning)
- **Standard mode** for regular monitoring
- 8 minutes, $0.11
- Review in standup

### Monthly (Sprint planning)
- **Deep mode** for strategic planning
- 15 minutes, $0.15
- Use for sprint priorities

### Pre-Release
- **Deep mode** comprehensive audit
- 15 minutes, $0.15
- Go/no-go decision

### As Needed
- **Critical mode** for emergencies
- **CI-aware mode** after failures
- 5-8 minutes, $0.08-0.12

**Total monthly cost: $0.60 - $1.50**

---

## 🚀 Example Workflows

### Scenario 1: Weekly Team Review

**Monday 9am:**
```bash
1. Run: Standard mode
2. Wait: 8 minutes
3. Download: llm-audit-standard-[run]
4. Review: AUDIT_REPORT.md in standup
5. Import: tasks.csv to sprint backlog
```

**ROI:** Catches issues before they become bugs

---

### Scenario 2: Pre-Release Audit

**Friday before release:**
```bash
1. Run: Deep mode
2. Wait: 15 minutes
3. Review: Phase 1 (Audit) - any blockers?
4. Review: Phase 2 (Plan) - priority fixes?
5. Review: Phase 3 (Act) - deployment checklist
6. Decision: Go/no-go based on findings
```

**ROI:** Prevents production incidents

---

### Scenario 3: CI Failure Investigation

**CI fails on main branch:**
```bash
1. Run: CI-aware mode
2. Wait: 8 minutes
3. Review: Root cause analysis
4. Check: Correlation with recent changes
5. Fix: Issues identified by LLM
6. Verify: Re-run CI
```

**ROI:** Faster root cause identification

---

### Scenario 4: Emergency Production Issue

**Production incident detected:**
```bash
1. Run: Critical mode
2. Wait: 5 minutes
3. Focus: Safety systems (kill switch, etc)
4. Check: Risk management logic
5. Validate: Circuit breakers working
6. Verify: No other critical issues
```

**ROI:** Comprehensive safety validation

---

## 🤖 AI Agent Integration

The `AI_AGENT_TASKS.json` file is designed for AI agents to consume:

### Example: Automated Fix Agent

```python
import json

# Load audit results
with open('AI_AGENT_TASKS.json') as f:
    audit = json.load(f)

# Get critical findings
critical = [
    f for f in audit['analysis']['critical_findings']
    if f['severity'] == 'CRITICAL'
]

# Process each finding
for finding in critical:
    # Agent analyzes the issue
    # Generates fix
    # Creates PR
    # Requests review
    pass
```

### Example: Issue Tracker Sync

```javascript
// Auto-create issues for high-priority items
const tasks = require('./AI_AGENT_TASKS.json');

tasks.analysis.action_plan
  .filter(item => item.priority === 'P0' || item.priority === 'P1')
  .forEach(item => {
    github.issues.create({
      title: `[LLM Audit] ${item.title}`,
      body: item.description,
      labels: ['llm-audit', item.priority, item.category]
    });
  });
```

---

## 🔧 Advanced Features

### Code Generation (Experimental)

Enable "Generate code" option:
- LLM creates fix recommendations
- Generates code snippets
- Saved in `generated-fixes/` directory
- **Not auto-committed** (requires manual review)

**Use case:** Get AI-assisted fix suggestions

**Warning:** Always review generated code before applying!

---

### GitHub Issues Creation (Optional)

Enable "Create issues" option:
- Auto-creates issues for critical findings
- Adds appropriate labels
- Links to audit run
- **Limited to 10 most critical items**

**Use case:** Quick issue tracking setup

**Warning:** Can create many issues - use sparingly!

---

## 📊 Interpreting Results

### Deep Mode Output Structure

**Phase 1: AUDIT**
```
System-level findings:
- Architecture strengths/weaknesses
- Cross-file patterns
- Systemic risks
- Critical vulnerabilities
```

**Phase 2: PLAN**
```
Strategic roadmap:
- Immediate actions (P0)
- Sprint goals (P1)
- Quick wins
- Deferred items
- Deployment blockers
```

**Phase 3: ACT**
```
Actionable tasks:
- Specific file/line numbers
- Implementation steps
- Fix recommendations
- Test cases
- Effort estimates
```

---

## ⚠️ Common Issues

### "API key not configured"
**Fix:** Add `XAI_API_KEY` or `GOOGLE_API_KEY` in repository secrets

### "Context size too large"
**Fix:** Use Quick or Critical mode (fewer files)

### "No analysis results"
**Fix:** Check API key is valid and has credits

### "Generated code looks wrong"
**Fix:** This is why it's not auto-committed! Review and adjust manually

---

## 💡 Pro Tips

1. **Start with Standard mode** - Best balance of coverage and cost

2. **Run Deep mode monthly** - Great for strategic planning

3. **Use CI-aware after failures** - Speeds up debugging

4. **Leave focus areas empty** - Auto-detection works great

5. **Review before creating issues** - Avoid issue tracker spam

6. **Download artifacts always** - Even if workflow "fails"

7. **Share AUDIT_REPORT.md** - Great for team visibility

8. **Track costs monthly** - Use artifact download counts

9. **Combine with static audit** - They complement each other

10. **Trust the AI, verify the output** - Good findings, but always review

---

## 📚 Related Documentation

- Static audit: `.github/workflows/ci.yml`
- Cost analysis: `UPDATED_COST_COMPARISON.md`
- 3-phase deep dive: `3PHASE_WORKFLOW_GUIDE.md`
- Workflow comparison: `WORKFLOW_COMPARISON.md`

---

## ✅ Quick Decision Matrix

**I want to...**

| Goal | Mode | Why |
|------|------|-----|
| Quick daily check | Quick | Fast, focused |
| Weekly monitoring | Standard | Comprehensive, affordable |
| Pre-release audit | Deep | Full system analysis + roadmap |
| Safety validation | Critical | Focus on safety systems |
| Debug CI failures | CI-Aware | Root cause analysis |
| Understand system | Deep | System-wide insights |
| Get task list | Standard or Deep | Generates actionable tasks |
| Feed AI agents | Any | All produce AI_AGENT_TASKS.json |

---

## 🎉 Summary

**One workflow to rule them all:**

- ⚡ Quick: Fast triage ($0.05)
- ⭐ Standard: Weekly monitoring ($0.11) **← Start here**
- 🔍 Deep: Comprehensive analysis ($0.15)
- 🚨 Critical: Safety focus ($0.08)
- 🤖 CI-Aware: Failure debugging ($0.12)

**All modes produce:**
- Human-readable report
- AI agent-ready JSON
- CSV for task import
- Rich context bundle

**Total cost: <$1/month for comprehensive coverage**

---

**Questions?** Just run Standard mode and see what you get!

**Ready?** GitHub → Actions → 🤖 LLM Audit → Run workflow 🚀