# LLM-Powered Audit Guide

> **Comprehensive guide for running AI-powered code audits with XAI Grok or Google Gemini**

## 🎯 Overview

The FKS Audit service has been refactored to provide:

1. **API-only server** - No web UI, pure REST API for programmatic access
2. **Static audits in CI** - Fast pattern-based analysis on every push
3. **Manual LLM audits** - Deep AI-powered analysis triggered from GitHub Actions web interface

## 📋 Table of Contents

- [JANUS Project Context](#janus-project-context)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [API-Only Server](#api-only-server)
- [LLM Audit Workflows](#llm-audit-workflows)
- [Supported LLM Providers](#supported-llm-providers)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

---

## 🧠 JANUS Project Context

### About Project JANUS

**JANUS** is a **neuromorphic trading intelligence system** that combines visual pattern recognition with symbolic logic for autonomous financial trading. The system is inspired by biological neural architecture and implements:

- **Dual-Service Architecture**: Forward (wake state) and Backward (sleep state) services
- **Neuro-Symbolic Fusion**: Deep learning with logical constraint satisfaction
- **Multi-Timescale Memory**: Hippocampus → Sharp-Wave Ripple → Neocortex consolidation
- **Brain-Inspired Components**: Visual cortex, prefrontal cortex, basal ganglia, cerebellum, amygdala

### Technical Paper Reference

The complete technical specification is available at:
**https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex**

Key sections include:
1. **Part 1: Main Architecture** - System design and philosophical foundation
2. **Part 2: Forward Service (Janus Bifrons)** - Real-time decision-making
3. **Part 3: Backward Service (Janus Consivius)** - Memory consolidation and learning
4. **Part 4: Neuromorphic Architecture** - Brain-region mapping
5. **Part 5: Rust Implementation** - Production deployment

### FKS Project Structure

The FKS repository contains multiple services:

```
fks/
├── src/
│   ├── janus/          # JANUS neuromorphic trading system
│   │   ├── services/
│   │   │   ├── forward/    # Real-time execution (ViViT, LTN, GAF)
│   │   │   ├── backward/   # Memory consolidation (PER, UMAP, Schemas)
│   │   │   ├── gateway/    # Python FastAPI orchestration
│   │   │   └── cns/        # Central Nervous System monitoring
│   │   ├── crates/
│   │   │   ├── vision/     # GAF & ViViT implementation
│   │   │   ├── logic/      # Logic Tensor Networks
│   │   │   ├── memory/     # Qdrant & experience replay
│   │   │   ├── execution/  # Almgren-Chriss execution
│   │   │   └── cns/        # Health monitoring
│   │   └── neuromorphic/   # Brain-inspired architectures
│   ├── audit/          # THIS SERVICE - Code audit system
│   ├── clients/        # Trading platform clients
│   └── monitor/        # System monitoring
```

### JANUS-Specific Audit Focus Areas

When auditing JANUS code, the LLM should pay special attention to:

#### 1. **Neuromorphic Architecture Compliance**
- Verify brain-region mappings are correctly implemented
- Check that Forward service maintains low-latency paths
- Ensure Backward service runs only during off-market hours
- Validate separation of concerns (hot path vs. cold path)

#### 2. **Mathematical Correctness**
- **GAF Transformations**: Verify polar coordinate conversions and Gramian matrix construction
- **LTN Operations**: Check Łukasiewicz t-norm implementations (conjunction, disjunction, implication)
- **Memory Consolidation**: Validate prioritized experience replay math (TD-error, importance sampling)
- **UMAP Embeddings**: Ensure proper dimensionality reduction and alignment

#### 3. **Financial Safety**
- **Wash Sale Rules**: Verify 30-day constraint enforcement
- **Risk Limits**: Check position sizing, daily loss limits, drawdown protection
- **Circuit Breakers**: Validate Mahalanobis distance calculations for anomaly detection
- **Almgren-Chriss**: Ensure market impact models prevent excessive slippage

#### 4. **Performance Requirements**
- **Forward Service**: p99 latency < 10ms, throughput > 10,000 req/s
- **Static Analysis**: Complete in < 3 seconds for 50k LOC
- **Memory Efficiency**: < 2GB RSS for forward service
- **Zero-Copy Communication**: Arrow IPC for forward→backward data transfer

#### 5. **Integration Points**
- **Qdrant Vector Database**: Schema storage and similarity search
- **Redis**: Job queue for backward service
- **QuestDB**: High-frequency tick data (100K+ writes/sec)
- **Prometheus/Grafana**: CNS health monitoring

#### 6. **Regulatory Compliance**
- Position limits and capital allocation constraints
- Logging requirements for audit trails
- Data retention policies
- API key security (never hardcoded)

### Key Algorithms to Validate

#### Gramian Angular Field (GAF) Generation
```rust
// From technical paper Part 2, Section 1.1
// Input: Time series X = {x_1, ..., x_W}
// Output: Gramian matrix G ∈ ℝ^(W×W)

// 1. Normalize to [-1, 1]
// 2. φ_i = arccos(x̃_i)
// 3. G_ij = cos(φ_i + φ_j)
```

#### Logic Tensor Network Constraints
```rust
// From technical paper Part 2, Section 2
// Łukasiewicz Logic (inference):
// Conjunction: a ∧ b = max(0, a + b - 1)
// Implication: a ⇒ b = min(1, 1 - a + b)
// 
// Product Logic (training):
// Conjunction: a ∧ b = a × b
// Implication: a ⇒ b = 1 - a + a × b
```

#### Prioritized Experience Replay
```rust
// From technical paper Part 3, Section 2.2
// Priority: p_i = |δ_i| + ε, where δ_i is TD-error
// Sampling: P(i) = p_i^α / Σ_j p_j^α
// Importance weight: w_i = (1 / (N × P(i)))^β
```

#### Circuit Breaker (Amygdala)
```rust
// From technical paper Part 4, Section 2.6
// Mahalanobis distance: D_M(s_t) = √((s_t - μ)ᵀ Σ⁻¹ (s_t - μ))
// Trigger if D_M(s_t) > τ_danger (e.g., τ = 5 for p < 0.001)
```

### LLM System Prompt Enhancements

When analyzing JANUS code, the LLM should be provided with this context:

**System Prompt Addition:**
```
You are auditing Project JANUS, a neuromorphic trading system inspired by brain architecture.

ARCHITECTURE OVERVIEW:
- Forward Service: Real-time trading (Visual cortex, Basal ganglia, Cerebellum)
- Backward Service: Memory consolidation (Hippocampus, Neocortex, Sharp-Wave Ripples)
- CNS: Health monitoring and orchestration (Central Nervous System)

KEY TECHNOLOGIES:
- Vision: Gramian Angular Fields (GAF) + Video Vision Transformers (ViViT)
- Logic: Logic Tensor Networks (LTN) with Łukasiewicz t-norms
- Memory: Qdrant vector DB + Prioritized Experience Replay (PER)
- Execution: Almgren-Chriss optimal execution
- Language: Rust for performance, Python for ML training

CRITICAL CHECKS:
1. Verify mathematical formulas match technical paper specifications
2. Ensure forward service maintains <10ms latency requirements
3. Validate LTN constraints enforce regulatory compliance
4. Check memory consolidation only runs during market close
5. Confirm circuit breakers use proper statistical thresholds
6. Validate zero-copy communication between services

TECHNICAL PAPER REFERENCE:
https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex

When you find issues, categorize them as:
- ARCHITECTURE: Deviations from neuromorphic design
- MATHEMATICS: Incorrect formula implementations
- PERFORMANCE: Latency/throughput violations
- SAFETY: Missing circuit breakers or risk controls
- COMPLIANCE: Regulatory constraint violations
```

### Code Tagging Conventions for JANUS

Use these audit tags in JANUS code:

```rust
// @audit-tag: neuromorphic
// Marks brain-inspired architectural components

// @audit-tag: forward-path
// Hot path code requiring <10ms latency

// @audit-tag: backward-path
// Cold path code for training/consolidation

// @audit-tag: mathematical
// Implementations of paper formulas (cite equation numbers)

// @audit-security: circuit-breaker
// Critical safety mechanisms

// @audit-freeze
// Mathematical constants from technical paper (never modify)

// @audit-todo: validate-formula
// Formula implementation needs verification against paper
```

### Example JANUS Audit Issues

**High Priority:**
```
ISSUE: GAF normalization uses fixed min-max instead of learnable affine transform
FILE: crates/vision/src/gaf.rs
LINE: 42
SEVERITY: Critical
CATEGORY: Mathematics
REFERENCE: Technical Paper Part 2, Eq. 2.1.1
RECOMMENDATION: Implement learnable γ, β parameters with tanh activation
```

**Medium Priority:**
```
ISSUE: LTN conjunction uses Product Logic during inference (should be Łukasiewicz)
FILE: crates/logic/src/ltn.rs
LINE: 156
SEVERITY: Medium
CATEGORY: Logic
REFERENCE: Technical Paper Part 2, Section 2.3.1
RECOMMENDATION: Use max(0, a + b - 1) for inference, keep Product for training
```

**Low Priority:**
```
ISSUE: CNS health check timeout not configured
FILE: services/cns/src/health.rs
LINE: 89
SEVERITY: Low
CATEGORY: Architecture
RECOMMENDATION: Add configurable timeout for probe responses
```

---

## 🏗️ Architecture

### Components

```
┌─────────────────────────────────────────────────────────┐
│                   FKS Audit System                      │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────┐    ┌──────────────┐                 │
│  │  audit-cli   │    │ audit-server │                 │
│  │  (Binary)    │    │  (Binary)    │                 │
│  └──────┬───────┘    └──────┬───────┘                 │
│         │                   │                          │
│         │                   │                          │
│  ┌──────▼───────────────────▼──────┐                  │
│  │       Core Audit Library        │                  │
│  │  • Tag Scanner                  │                  │
│  │  • Static Analyzer              │                  │
│  │  • LLM Client (XAI/Google)      │                  │
│  │  • Task Generator               │                  │
│  └─────────────────────────────────┘                  │
│                                                         │
└─────────────────────────────────────────────────────────┘

         │                    │
         ▼                    ▼
    
┌─────────────┐      ┌──────────────┐
│  CI/CD      │      │  REST API    │
│  (GitHub    │      │  Consumers   │
│   Actions)  │      │              │
└─────────────┘      └──────────────┘
```

### Modes of Operation

1. **Static Audit (Automated)** - Runs on every push to `main`/`develop`
   - Fast pattern matching
   - No LLM costs
   - Catches common issues
   - Reports uploaded as artifacts

2. **LLM Audit (Manual)** - Triggered manually from GitHub web interface
   - Deep AI analysis
   - Security and logic validation
   - Compliance checking
   - Task generation
   - Uses API secrets for LLM access

3. **API Server (On-Demand)** - REST API for custom integrations
   - No web UI
   - Pure API endpoints
   - Programmatic access
   - Can be deployed standalone

---

## 🚀 Quick Start

### 1. Static Audit (Already Running)

Every push to `main` or `develop` automatically runs:

```bash
# View in GitHub Actions:
# https://github.com/nuniesmith/fks/actions
```

Artifacts include:
- `tags-report.txt` - All audit tags found
- `static-report.txt` - Pattern-based analysis
- `tasks.json` - Generated action items

### 2. Manual LLM Audit (New!)

**Prerequisites:**
- GitHub repository secrets configured (see [Configuration](#configuration))
- At least one of: `XAI_API_KEY` or `GOOGLE_API_KEY`

**Steps:**

1. Go to **Actions** tab on GitHub
2. Select **🤖 LLM Audit** workflow
3. Click **Run workflow** dropdown
4. Configure options:
   - **LLM Provider**: `xai` (Grok) or `google` (Gemini)
   - **Analysis Depth**: `standard` or `deep`
   - **Focus Areas**: `security,logic,compliance,performance`
   - **Include Tests**: `true`/`false`
   - **Batch Size**: `10` (1-20, lower = more detailed)
5. Click **Run workflow**

**Wait Time:** 10-30 minutes depending on codebase size and depth

**Results:** Download artifacts from workflow run:
- `llm-audit-report.json` - Full structured report
- `llm-audit-report.txt` - Human-readable report
- `llm-tasks.json` - Actionable tasks (priority sorted)
- `llm-tasks.csv` - Import to Jira/Linear/etc.

### 3. API Server (For Custom Integration)

```bash
# Start the API server
cd src/audit
export XAI_API_KEY="your-key-here"
export LLM_ENABLED="true"
cargo run --release --bin audit-server

# Server runs on http://localhost:8080
```

---

## 🌐 API-Only Server

### Why API-Only?

- **Lightweight** - No static assets, minimal dependencies
- **Secure** - No web UI attack surface
- **Cloud-native** - Easy to deploy in containers
- **Programmatic** - Integrate with CI/CD, webhooks, etc.

### Available Endpoints

#### Health Check
```bash
GET /health

Response:
{
  "status": "healthy",
  "version": "0.1.0"
}
```

#### Run Full Audit
```bash
POST /api/audit
Content-Type: application/json

{
  "repository": "/path/to/repo",
  "branch": "main",
  "enable_llm": true,
  "focus": ["security", "logic"],
  "include_tests": false
}

Response:
{
  "id": "uuid",
  "status": "completed",
  "report": { ... }
}
```

#### Scan Tags Only
```bash
POST /api/scan/tags
Content-Type: application/json

{
  "path": "/path/to/repo"
}

Response:
{
  "total": 42,
  "by_type": {
    "Security": 15,
    "Dead": 8,
    ...
  },
  "tags": [...]
}
```

#### Static Analysis
```bash
POST /api/scan/static
Content-Type: application/json

{
  "path": "/path/to/repo"
}

Response:
{
  "total_files": 150,
  "total_issues": 23,
  "critical_files": 5,
  "issues_by_severity": { ... }
}
```

#### Get Audit Results
```bash
GET /api/audit/{id}

Response:
{
  "id": "uuid",
  "summary": { ... },
  "files": [ ... ],
  "tasks": [ ... ]
}
```

#### Get Tasks
```bash
GET /api/audit/{id}/tasks

Response:
{
  "tasks": [
    {
      "id": "task-1",
      "title": "Fix SQL injection",
      "priority": 9,
      ...
    }
  ]
}
```

### Running the Server

**Local Development:**
```bash
cd src/audit

# With XAI Grok
export XAI_API_KEY="xai-..."
export LLM_PROVIDER="xai"
export LLM_ENABLED="true"
cargo run --release --bin audit-server

# With Google Gemini
export GOOGLE_API_KEY="AIza..."
export LLM_PROVIDER="google"
export LLM_MODEL="gemini-2.0-flash-exp"
export LLM_ENABLED="true"
cargo run --release --bin audit-server
```

**Docker:**
```bash
docker build -t fks-audit .
docker run -p 8080:8080 \
  -e XAI_API_KEY="xai-..." \
  -e LLM_ENABLED="true" \
  fks-audit
```

---

## 🤖 LLM Audit Workflows

### GitHub Actions Integration

The LLM audit workflow is located at `.github/workflows/llm-audit.yml`

**Trigger:** Manual only (workflow_dispatch)

**Features:**
- Choose provider (XAI or Google)
- Configurable depth and focus
- Automatic report generation
- Task extraction
- Artifact upload (90-day retention)

### Configuration Options

| Option | Description | Values | Default |
|--------|-------------|--------|---------|
| `llm_provider` | Which AI to use | `xai`, `google` | `xai` |
| `analysis_depth` | How thorough | `standard`, `deep` | `standard` |
| `focus_areas` | What to check | Comma-separated | `security,logic,performance,compliance,architecture` |
| `include_tests` | Scan test files | `true`, `false` | `true` |
| `batch_size` | Files per call | `1-20` | `10` |

### Analysis Depth

**Standard:**
- 4096 max tokens
- Focus on high-priority files
- Faster (10-15 min)
- Lower API costs

**Deep:**
- 8192 max tokens
- Analyzes more context
- Slower (20-30 min)
- Higher quality insights

### Focus Areas

- **security** - Authentication, authorization, SQL injection, XSS, API key management, circuit breakers
- **logic** - Business logic correctness, LTN constraint satisfaction, mathematical formula accuracy
- **compliance** - Wash sale rules, position limits, regulatory constraints, audit logging
- **performance** - Latency requirements (<10ms forward), throughput (100K+ QuestDB writes), memory efficiency
- **architecture** - Neuromorphic design patterns, brain-region mappings, service separation (hot/cold paths)

**Default:** All areas are analyzed for comprehensive coverage

**JANUS-Specific Areas:**
- **neuromorphic** - Brain-inspired architecture compliance, component mappings
- **mathematical** - Formula implementations vs. technical paper specifications
- **trading-safety** - Risk limits, circuit breakers, execution algorithms

---

## 🔌 Supported LLM Providers

### XAI Grok (Default)

**Model:** `grok-4-1-fast-reasoning`  
**API:** https://api.x.ai/v1  
**Secret:** `XAI_API_KEY`

**Pricing:** Check [x.ai pricing](https://x.ai/api)

**Setup:**
```bash
# Get API key from console.x.ai
export XAI_API_KEY="xai-..."
export LLM_PROVIDER="xai"
export LLM_MODEL="grok-4-1-fast-reasoning"
```

**Strengths:**
- Fast responses
- Good code understanding
- Competitive pricing
- OpenAI-compatible API

### Google Gemini

**Model:** `gemini-2.0-flash-exp`  
**API:** https://generativelanguage.googleapis.com/v1beta  
**Secret:** `GOOGLE_API_KEY`

**Pricing:** Check [Google AI pricing](https://ai.google.dev/pricing)

**Setup:**
```bash
# Get API key from aistudio.google.com
export GOOGLE_API_KEY="AIza..."
export LLM_PROVIDER="google"
export LLM_MODEL="gemini-2.0-flash-exp"
```

**Strengths:**
- Generous free tier
- Long context windows
- Multimodal capabilities
- Strong reasoning

### Provider Comparison

| Feature | XAI Grok | Google Gemini |
|---------|----------|---------------|
| Free Tier | Limited | Generous |
| Context Window | 128K | 1M+ |
| Speed | Fast | Very Fast |
| Code Understanding | Excellent | Excellent |
| API Format | OpenAI | Google |
| Authentication | Bearer token | API key in URL |

---

## ⚙️ Configuration

### GitHub Secrets (Required for LLM Audits)

Go to **Settings** → **Secrets and variables** → **Actions** → **New repository secret**

Add at least one:

```
Name: XAI_API_KEY
Value: xai-xxxxxxxxxxxxxxxx

Name: GOOGLE_API_KEY
Value: AIzaxxxxxxxxxxxxxxxx
```

### Environment Variables

**Server Configuration:**
```bash
AUDIT_HOST=0.0.0.0           # Server bind address
AUDIT_PORT=8080              # Server port
```

**LLM Configuration:**
```bash
LLM_PROVIDER=xai             # xai | google
LLM_ENABLED=true             # Enable LLM features
LLM_MODEL=grok-4-1-fast-reasoning          # Model name
LLM_MAX_TOKENS=4096          # Max response length
LLM_TEMPERATURE=0.3          # Creativity (0.0-1.0)
```

**API Keys (provider-specific):**
```bash
XAI_API_KEY=xai-xxx          # For XAI Grok
GOOGLE_API_KEY=AIza-xxx      # For Google Gemini
```

**Scanner Configuration:**
```bash
SCANNER_MAX_FILE_SIZE=1000000    # 1MB
SCANNER_INCLUDE_TESTS=false      # Scan test files
```

**Storage:**
```bash
STORAGE_REPORTS_DIR=./reports    # Audit reports
STORAGE_TASKS_DIR=./tasks        # Generated tasks
```

**Git:**
```bash
GIT_WORKSPACE_DIR=./workspace    # Where to clone repos
GIT_DEFAULT_BRANCH=main          # Default branch
GIT_SHALLOW_CLONE=true           # Shallow clone
```

---

## 🧬 JANUS-Specific Audit Examples

### Example: Audit JANUS Forward Service

```bash
# GitHub Actions workflow trigger
# Focus on neuromorphic architecture and performance

1. Go to Actions → 🤖 LLM Audit
2. Click Run workflow
3. Configure:
   - Provider: xai
   - Depth: deep
   - Focus: architecture,performance,mathematical,trading-safety
   - Include Tests: true
   - Batch Size: 5 (detailed analysis)
```

### Example: Validate Mathematical Implementations

```bash
# Local CLI audit focusing on formula correctness
cd src/audit

# Audit vision crate (GAF, ViViT)
cargo run --release --bin audit-cli -- audit \
  ../janus/crates/vision \
  --llm \
  --focus mathematical,performance \
  --output vision-audit.json

# Audit logic crate (LTN)
cargo run --release --bin audit-cli -- audit \
  ../janus/crates/logic \
  --llm \
  --focus mathematical,logic,compliance \
  --output ltn-audit.json
```

### Example: Check Regulatory Compliance

```bash
# Audit execution and risk management
cargo run --release --bin audit-cli -- audit \
  ../janus/crates/execution \
  --llm \
  --focus compliance,trading-safety,security \
  --output compliance-audit.json
```

### Example: Architecture Review

```bash
# Full JANUS architecture audit
cargo run --release --bin audit-cli -- audit \
  ../janus \
  --llm \
  --focus architecture,neuromorphic \
  --exclude-tests \
  --batch-size 10 \
  --output janus-architecture.json
```

---

## 💡 Examples

### Example 1: Run LLM Audit via GitHub

1. **Navigate to Actions:**
   ```
   https://github.com/nuniesmith/fks/actions/workflows/llm-audit.yml
   ```

2. **Click "Run workflow"**

3. **Configure:**
   - Provider: `xai`
   - Depth: `standard`
   - Focus: `security,logic,performance,compliance,architecture` (all areas)
   - Include Tests: `true` (always include paired test files)

4. **Wait for completion** (~15 min)

5. **Download artifacts** and review:
   ```
   llm-audit-results-xai-123/
   ├── llm-audit-report.json
   ├── llm-audit-report.txt
   ├── llm-tasks.json
   └── llm-tasks.csv
   ```

### Example 2: Local CLI Audit

```bash
cd src/audit
cargo build --release

# Static audit (no LLM)
./target/release/audit-cli static ../.. --format text

# LLM audit with XAI (includes tests by default)
export XAI_API_KEY="xai-..."
./target/release/audit-cli audit ../.. \
  --llm \
  --output report.json \
  --format json

# Exclude tests if needed
./target/release/audit-cli audit ../.. \
  --llm \
  --exclude-tests \
  --output report.json \
  --format json

# Generate tasks
./target/release/audit-cli tasks ../.. \
  --format csv \
  --output tasks.csv
```

### Example 3: API Server Integration

```python
import requests

# Start audit
response = requests.post('http://localhost:8080/api/audit', json={
    'repository': '/path/to/code',
    'enable_llm': True,
    'focus': ['security'],
    'include_tests': False
})

audit_id = response.json()['id']

# Get results
report = requests.get(f'http://localhost:8080/api/audit/{audit_id}')
print(report.json())

# Get tasks
tasks = requests.get(f'http://localhost:8080/api/audit/{audit_id}/tasks')
for task in tasks.json()['tasks']:
    print(f"[{task['priority']}] {task['title']}")
```

### Example 4: Compare Providers

Run the same audit with both providers to compare results:

```bash
# Run with XAI
export LLM_PROVIDER=xai
export XAI_API_KEY="xai-..."
./target/release/audit-cli audit ../.. --llm --output xai-report.json

# Run with Google
export LLM_PROVIDER=google
export GOOGLE_API_KEY="AIza..."
./target/release/audit-cli audit ../.. --llm --output gemini-report.json

# Compare
diff <(jq -S . xai-report.json) <(jq -S . gemini-report.json)
```

---

## 🔧 Troubleshooting

### Issue: "No API key provided"

**Solution:** Set the appropriate environment variable:
```bash
# For XAI
export XAI_API_KEY="xai-..."

# For Google
export GOOGLE_API_KEY="AIza..."
```

### Issue: "API returned 401"

**Cause:** Invalid or expired API key

**Solution:**
1. Verify key is correct
2. Check key hasn't expired
3. Ensure key has proper permissions

### Issue: "Rate limit exceeded"

**Cause:** Too many API calls

**Solution:**
1. Increase `batch_size` in workflow
2. Reduce analysis depth to `standard`
3. Wait before running again
4. Upgrade API tier

### Issue: "Timeout waiting for LLM"

**Cause:** Large files or complex analysis

**Solution:**
1. Exclude large files (check `.dockerignore` patterns)
2. Reduce batch size
3. Skip test files (`include_tests: false`)
4. Use `standard` instead of `deep`

### Issue: Workflow fails with "Model not found"

**Cause:** Invalid model name for provider

**Solution:**
```yaml
# XAI models
LLM_MODEL: "grok-4-1-fast-reasoning"

# Google models
LLM_MODEL: "gemini-2.0-flash-exp"
# or
LLM_MODEL: "gemini-pro"
```

### Issue: High API costs

**Recommendations:**
1. Use Google Gemini free tier for testing
2. Run LLM audits weekly instead of daily
3. Use all focus areas for comprehensive analysis
4. Always include test files (they provide context)
5. Use `standard` depth by default
6. Increase batch size to reduce calls

---

## 📊 Best Practices

### When to Use Static vs LLM Audits

**Static Audits (Every Push):**
- ✅ Fast feedback (< 5 min)
- ✅ No costs
- ✅ Catch common patterns
- ✅ Tag compliance
- ❌ Limited context understanding

**LLM Audits (Weekly/On-Demand):**
- ✅ Deep analysis
- ✅ Logic validation
- ✅ Security insights
- ✅ Compliance checking
- ❌ Slower (15-30 min)
- ❌ API costs

### Recommended Cadence

- **Static:** Every commit (automated)
- **LLM Standard:** Weekly on `main`
- **LLM Deep:** Before major releases
- **LLM Focus:** After security incidents

### Cost Optimization

1. **Use free tiers:**
   - Google Gemini has generous limits
   - Good for testing and small projects

2. **Batch wisely:**
   - Larger batches = fewer API calls
   - Including tests adds context (worth it!)
   - Sweet spot: 10-15 files per batch

3. **Focus areas:**
   - Use all areas by default for complete coverage
   - Narrow focus only for specific investigations
   - Test files help validate logic and security

4. **Incremental audits:**
   - Audit changed files only
   - Full audit monthly

---

## 🎓 Advanced Usage

### Custom LLM Models

Edit `.github/workflows/llm-audit.yml`:

```yaml
env:
  LLM_MODEL: ${{ inputs.llm_provider == 'xai' && 'grok-2' || 'gemini-1.5-pro' }}
```

### Scheduled LLM Audits

Add to `llm-audit.yml`:

```yaml
on:
  schedule:
    - cron: '0 2 * * 0'  # Sundays at 2 AM UTC
  workflow_dispatch:
    # ... existing config
```

### Fail Build on Critical Issues

In `llm-audit.yml`, change:

```yaml
- name: ⚠️ Check Critical Issues
  run: |
    critical_count=$(jq -r '.issues_by_severity.Critical // 0' llm-audit-report.json)
    if [ "$critical_count" -gt "0" ]; then
      echo "::error::Found $critical_count critical issues"
      exit 1  # <-- This will fail the workflow
    fi
```

### Integrate with Issue Tracker

Add step to create GitHub Issues from tasks:

```yaml
- name: Create Issues
  env:
    GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  run: |
    jq -c '.[] | select(.priority >= 7)' llm-tasks.json | while read task; do
      title=$(echo "$task" | jq -r '.title')
      body=$(echo "$task" | jq -r '.description')
      gh issue create --title "$title" --body "$body" --label "audit,security"
    done
```

---

## 📚 Related Documentation

- [Quick Start Guide](./QUICK_START.md)
- [README](./README.md)
- [CI Static Audit](../../docs/CI_STATIC_AUDIT.md)
- [GitHub Actions Workflows](../../.github/workflows/)

---

## 🆘 Support

**Issues:** [github.com/nuniesmith/fks/issues](https://github.com/nuniesmith/fks/issues)

**Questions:** Open a discussion or issue

**Contributing:** PRs welcome!

---

**Last Updated:** 2024
**Version:** 0.1.0