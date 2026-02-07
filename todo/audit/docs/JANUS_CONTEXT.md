# JANUS Project Context for LLM Audits

> **Reference document for AI-powered code audits of Project JANUS**  
> This file provides comprehensive context about the JANUS neuromorphic trading system to enhance LLM audit quality.

---

## 📋 Table of Contents

- [Overview](#overview)
- [Technical Paper Reference](#technical-paper-reference)
- [Architecture Summary](#architecture-summary)
- [Mathematical Specifications](#mathematical-specifications)
- [Performance Requirements](#performance-requirements)
- [Safety & Compliance](#safety--compliance)
- [Code Conventions](#code-conventions)
- [Audit Checklists](#audit-checklists)

---

## 🎯 Overview

### What is JANUS?

**Project JANUS** is a **neuromorphic trading intelligence system** that synthesizes visual pattern recognition with symbolic logic for autonomous financial trading. Named after the Roman god of beginnings and transitions who looks simultaneously to the future and past, JANUS implements:

- **Dual-Service Architecture**: Forward (wake/execution) and Backward (sleep/learning)
- **Neuro-Symbolic Fusion**: Deep learning outputs gated by logical constraints
- **Multi-Timescale Memory**: Hippocampus → Sharp-Wave Ripple → Neocortex
- **Brain-Inspired Design**: Components mapped to specific brain regions

### Design Philosophy

JANUS addresses the "crisis of complexity" in quantitative trading by:

1. **Biological Plausibility**: Using neuroscience principles for system design
2. **Interpretability**: Symbolic logic makes decisions explainable
3. **Safety**: Built-in circuit breakers and regulatory compliance
4. **Performance**: Rust implementation for production-grade latency
5. **Continual Learning**: Sleep-wake cycle prevents catastrophic forgetting

### Core Technologies

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Vision** | GAF + ViViT | Transform time series to spatiotemporal images |
| **Logic** | LTN (Łukasiewicz) | Enforce regulatory constraints differentiably |
| **Memory** | Qdrant + PER | Vector database + prioritized experience replay |
| **Execution** | Almgren-Chriss | Optimal trade execution with market impact |
| **Language** | Rust + Python | Performance + ML ecosystem |
| **Monitoring** | CNS (Prometheus) | Health checks and observability |

---

## 📚 Technical Paper Reference

### Source

**Repository**: https://github.com/nuniesmith/technical_papers  
**File**: `project_janus/janus.tex`  
**Direct Link**: https://raw.githubusercontent.com/nuniesmith/technical_papers/main/project_janus/janus.tex

### Document Structure

The technical paper is organized into 5 parts:

#### Part 1: Main Architecture
- Introduction and design philosophy
- Crisis of complexity in quantitative trading
- Dual-service design rationale
- Memory hierarchy overview

#### Part 2: Forward Service (Janus Bifrons)
- **Section 1**: Visual Pattern Recognition (DiffGAF, ViViT)
- **Section 2**: Logic Tensor Networks (LTN)
- **Section 3**: Multimodal Fusion
- **Section 4**: Decision Engine (Basal Ganglia)

#### Part 3: Backward Service (Janus Consivius)
- **Section 1**: Three-Timescale Memory Hierarchy
- **Section 2**: Sharp-Wave Ripple Simulation (PER)
- **Section 3**: UMAP Visualization
- **Section 4**: Qdrant Integration

#### Part 4: Neuromorphic Architecture
- Brain region mappings
- Component-to-neuroscience correspondence
- Functional justifications

#### Part 5: Rust Implementation
- ML framework strategy (PyTorch → ONNX → Rust)
- High-performance services
- Deployment architecture

---

## 🏗️ Architecture Summary

### Service Topology

```
┌─────────────────────────────────────────────────────────┐
│                    JANUS System                         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────────┐         ┌──────────────────┐    │
│  │ Forward Service  │  Arrow  │ Backward Service │    │
│  │  (Wake State)    │  ──IPC→ │  (Sleep State)   │    │
│  │                  │         │                  │    │
│  │ • ViViT Vision   │         │ • PER Buffer     │    │
│  │ • LTN Logic      │         │ • UMAP Schemas   │    │
│  │ • BG Decision    │         │ • K-means        │    │
│  │ • Cerebellum Exec│         │ • Qdrant Store   │    │
│  │                  │         │                  │    │
│  │ Port: 8080       │         │ Internal Only    │    │
│  │ Latency: <10ms   │         │ Batch Processing │    │
│  └──────────────────┘         └──────────────────┘    │
│           │                            │               │
│           ├────────────────────────────┤               │
│           ▼                            ▼               │
│  ┌──────────────────┐         ┌──────────────────┐    │
│  │ CNS (Monitoring) │         │ Qdrant Vector DB │    │
│  │ • Health Checks  │         │ • Schema Storage │    │
│  │ • Prometheus     │         │ • Similarity     │    │
│  │ • Grafana        │         │   Search         │    │
│  │ Port: 9090       │         │ Port: 6333       │    │
│  └──────────────────┘         └──────────────────┘    │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Brain-Region Mapping

| Brain Region | Biological Function | JANUS Component | Implementation |
|--------------|---------------------|-----------------|----------------|
| **Visual Cortex** | Pattern recognition | GAF + ViViT | `crates/vision/` |
| **Hippocampus** | Episodic memory | Experience buffer | `crates/memory/src/hippocampus.rs` |
| **Prefrontal Cortex** | Logic & planning | LTN constraints | `crates/logic/` |
| **Basal Ganglia** | Action selection | Dual-pathway decision | `services/forward/src/decision.rs` |
| **Cerebellum** | Motor prediction | Market impact model | `crates/execution/` |
| **Amygdala** | Threat detection | Circuit breakers | `services/forward/src/amygdala.rs` |
| **Neocortex** | Long-term memory | Schema database | Qdrant |

### Service Responsibilities

#### Forward Service (Hot Path)
- Real-time market data processing
- GAF video generation (sliding windows)
- ViViT spatiotemporal attention
- LTN constraint checking
- Basal ganglia action selection
- Order execution
- **Performance**: p99 < 10ms, 10K req/s

#### Backward Service (Cold Path)
- Experience replay prioritization
- UMAP dimensionality reduction
- K-means schema clustering
- Qdrant vector updates
- Model retraining (offline)
- **Timing**: Runs during market close only

#### CNS (Central Nervous System)
- Health monitoring (brain + probes)
- Prometheus metrics collection
- Circuit breaker coordination
- Auto-recovery reflexes
- Grafana dashboards

---

## 🔢 Mathematical Specifications

### 1. Gramian Angular Field (GAF)

**Reference**: Technical Paper Part 2, Section 1.1

#### Input Preprocessing
```
Given: Time series X = {x₁, x₂, ..., xₜ} where xₜ ∈ ℝᴰ
```

#### Step 1: Learnable Normalization
```
x̃ₜ = tanh(γ ⊙ (xₜ - μ)/σ + β)
```
- `γ, β ∈ ℝᶠ` are learned parameters
- `μ, σ` are running statistics
- `tanh` ensures x̃ₜ ∈ (-1, 1)

#### Step 2: Polar Transformation
```
φₜ = arccos(x̃ₜ) ∈ [0, π]
rₜ = t/T (normalized timestamp)
```

#### Step 3: Gramian Matrix Construction
```
GASF: Gᵢⱼ = cos(φᵢ + φⱼ) = x̃ᵢx̃ⱼ - √(1-x̃ᵢ²)√(1-x̃ⱼ²)
GADF: Gᵢⱼ = sin(φᵢ - φⱼ) = √(1-x̃ᵢ²)x̃ⱼ - x̃ᵢ√(1-x̃ⱼ²)
```

**Critical**: Normalization MUST guarantee |x̃ₜ| < 1 or arccos will fail!

### 2. Logic Tensor Networks (LTN)

**Reference**: Technical Paper Part 2, Section 2

#### T-norm Operations

**Łukasiewicz Logic (for inference)**:
```
Conjunction:  a ∧ b = max(0, a + b - 1)
Disjunction:  a ∨ b = min(1, a + b)
Negation:     ¬a = 1 - a
Implication:  a ⇒ b = min(1, 1 - a + b)
```

**Product Logic (for training - ensures gradient flow)**:
```
Conjunction:  a ∧ b = a × b
Implication:  a ⇒ b = 1 - a + a × b
```

#### Constraint Examples

**Wash Sale Rule**:
```
∀t: Sell(t) ∧ Buy(t') ∧ |t-t'| < 30 ⇒ ¬TaxLoss(t)
```

**Risk Constraint**:
```
∀order: Execute(order) ⇒ Slippage(order) < λ × Volatility
```

#### Loss Function
```
SAT(KB) = p-mean(φ₁, φ₂, ..., φₙ)
L_logic = 1 - SAT(KB)
```

### 3. Prioritized Experience Replay (PER)

**Reference**: Technical Paper Part 3, Section 2.2

#### Priority Calculation
```
pᵢ = |δᵢ| + ε
```
where:
- `δᵢ = rᵢ + γ max_a' Q(s_{i+1}, a') - Q(sᵢ, aᵢ)` (TD-error)
- `ε = 10⁻⁶` (numerical stability)

#### Sampling Probability
```
P(i) = pᵢᵅ / Σⱼ pⱼᵅ
```
- `α ∈ [0, 1]` controls prioritization strength (typical: 0.6)

#### Importance Sampling Weights
```
wᵢ = (1 / (N × P(i)))ᵝ
```
- `β ∈ [0, 1]` annealed from 0.4 → 1.0 during training

### 4. Circuit Breaker (Amygdala)

**Reference**: Technical Paper Part 4, Section 2.6

#### Mahalanobis Distance
```
D_M(sₜ) = √((sₜ - μ)ᵀ Σ⁻¹ (sₜ - μ))
```

#### Trigger Condition
```
Trigger = 1 if D_M(sₜ) > τ_danger else 0
```
- `τ_danger` calibrated to false-positive rate (e.g., τ=5 for p<0.001)

#### Additional Threats
- Volatility spike: `σₜ > 3 × σ_baseline`
- Drawdown: `cumulative_loss > L_max`
- Liquidity crisis: `bid_ask_spread > 10 × normal`

### 5. UMAP Dimensionality Reduction

**Reference**: Technical Paper Part 3, Section 3

#### Objective Function
```
L_UMAP = Σᵢ≠ⱼ [wᵢⱼ log(qᵢⱼ) + (1 - wᵢⱼ) log(1 - qᵢⱼ)]
```
where:
- `qᵢⱼ = (1 + ‖yᵢ - yⱼ‖²)⁻¹`
- Repulsion term approximated via negative sampling (k=5)

---

## ⚡ Performance Requirements

### Latency Targets

| Service | Metric | Target | Critical? |
|---------|--------|--------|-----------|
| Forward | p50 latency | < 5ms | ⚠️ Yes |
| Forward | p99 latency | < 10ms | ⚠️ Yes |
| Forward | Throughput | > 10K req/s | ⚠️ Yes |
| Backward | Batch time | < 30 min | No |
| CNS | Health check | < 100ms | Yes |

### Resource Limits

| Service | Memory | CPU | Disk |
|---------|--------|-----|------|
| Forward | < 2GB RSS | 2 cores | Minimal |
| Backward | < 8GB RSS | 4 cores | Ephemeral |
| Qdrant | < 4GB RSS | 2 cores | Persistent |
| CNS | < 512MB | 1 core | Logs only |

### Data Throughput

| Operation | Target | Technology |
|-----------|--------|------------|
| Market ticks | 100K+ writes/sec | QuestDB |
| Vector search | < 10ms @ 1M vectors | Qdrant |
| GAF generation | < 2ms (W=64) | Rust SIMD |
| LTN evaluation | < 1ms (10 constraints) | Rust |

### Benchmarks

On typical hardware (Ryzen 5950X, 64GB RAM):
- **Static Analysis**: ~3s for 50K LOC
- **Tag Scanning**: ~700ms for 50K LOC
- **LLM Analysis**: ~45s for 10 files
- **GAF Video**: ~5ms for 128-frame sequence
- **UMAP Embedding**: ~2s for 10K points

---

## 🛡️ Safety & Compliance

### Regulatory Constraints

#### 1. Wash Sale Rule (IRS)
```rust
// @audit-security: wash-sale
// Verify no buy within 30 days of loss-realizing sell
if sell_order.realizes_loss() {
    assert!(no_buy_within_30_days(symbol, sell_date));
}
```

#### 2. Position Limits
```rust
const MAX_POSITION_SIZE: f64 = 0.10; // 10% of account
const DAILY_LOSS_LIMIT: f64 = 0.02;  // 2% daily drawdown
```

#### 3. Pattern Day Trader (PDT) Rules
```rust
// Must maintain $25K if day trading frequency > threshold
if day_trades_last_5_days >= 4 {
    assert!(account_value >= 25_000.0);
}
```

### Circuit Breakers

#### Level 1: Warning
- Volatility > 2σ baseline
- Drawdown > 1% in 1 hour
- Action: Reduce position sizes by 50%

#### Level 2: Pause
- Volatility > 3σ baseline
- Drawdown > 2% in 1 hour
- Action: Stop new orders, monitor only

#### Level 3: Emergency Stop
- Mahalanobis distance > 5
- Drawdown > 3% (daily limit)
- Liquidity crisis detected
- Action: Close all positions, halt trading

### API Key Security

**NEVER hardcode API keys**:
```rust
// ❌ WRONG
const API_KEY: &str = "xai-123456789";

// ✅ CORRECT
let api_key = env::var("XAI_API_KEY")
    .expect("XAI_API_KEY must be set");
```

### Audit Logging

All trading decisions must be logged:
```rust
info!(
    symbol = %order.symbol,
    direction = %order.side,
    quantity = %order.quantity,
    reason = %decision.rationale,
    "Order submitted"
);
```

---

## 📝 Code Conventions

### Audit Tags

Use these custom tags in JANUS code:

#### @audit-tag: [category]
Categorize code by architectural role:
```rust
// @audit-tag: neuromorphic
// Brain-inspired component

// @audit-tag: forward-path
// Hot path requiring <10ms latency

// @audit-tag: backward-path
// Cold path for training/consolidation

// @audit-tag: mathematical
// Implements formula from technical paper
```

#### @audit-todo: [description]
Track implementation tasks:
```rust
// @audit-todo: Implement learnable GAF normalization (Part 2, Eq. 2.1.1)
```

#### @audit-freeze
Mark constants that must never change:
```rust
// @audit-freeze
const LUKASIEWICZ_EPSILON: f64 = 1e-10;
```

#### @audit-security: [concern]
Flag security-critical code:
```rust
// @audit-security: circuit-breaker
// Emergency stop mechanism - test thoroughly!
```

#### @audit-review: [notes]
Request code review:
```rust
// @audit-review: Verify this matches technical paper formula
```

### Naming Conventions

#### Brain-Region Components
```rust
// Prefix with brain region name
pub struct VisualCortexGAF { ... }
pub struct BasalGangliaDecision { ... }
pub struct HippocampusBuffer { ... }
```

#### Mathematical Functions
```rust
// Use paper equation numbers in comments
/// Implements Łukasiewicz conjunction (Part 2, Eq. 2.3.1)
fn lukasiewicz_and(a: f64, b: f64) -> f64 {
    (a + b - 1.0).max(0.0)
}
```

#### Service Boundaries
```rust
// Forward service: ForwardXxx
pub struct ForwardEngine { ... }

// Backward service: BackwardXxx
pub struct BackwardConsolidator { ... }

// CNS: CnsXxx
pub struct CnsHealthProbe { ... }
```

---

## ✅ Audit Checklists

### Mathematical Correctness

- [ ] GAF normalization uses learnable parameters (γ, β)
- [ ] Polar transformation includes bounds checking
- [ ] Gramian matrix is symmetric (Gᵢⱼ = Gⱼᵢ)
- [ ] LTN uses Łukasiewicz for inference, Product for training
- [ ] PER priority includes epsilon for numerical stability
- [ ] Importance weights are normalized correctly
- [ ] Mahalanobis distance uses inverse covariance
- [ ] UMAP negative sampling ratio is 5:1

### Performance Compliance

- [ ] Forward service has no blocking I/O in hot path
- [ ] Memory allocations use pre-allocated buffers
- [ ] Vector operations use SIMD when possible
- [ ] Database queries are indexed properly
- [ ] Backward service runs only during market close
- [ ] Zero-copy Arrow IPC for forward→backward
- [ ] Prometheus metrics use efficient counters

### Safety Validation

- [ ] All unwrap() calls have justification or are removed
- [ ] Circuit breakers tested with extreme inputs
- [ ] Position limits enforced at multiple layers
- [ ] API keys loaded from environment only
- [ ] Wash sale constraint verified in tests
- [ ] Drawdown limits have hard stops
- [ ] Emergency shutdown halts all services

### Architecture Compliance

- [ ] Brain-region mapping is clear and documented
- [ ] Forward/backward separation is maintained
- [ ] No backward code in forward service
- [ ] CNS can monitor all service health
- [ ] Services communicate via defined interfaces
- [ ] Configuration is externalized
- [ ] Logging uses structured format

### Regulatory Adherence

- [ ] Wash sale rules implemented correctly
- [ ] Position limits match policy documents
- [ ] PDT rules enforced if applicable
- [ ] Audit trail logs all decisions
- [ ] Data retention meets requirements
- [ ] Privacy controls for customer data
- [ ] Compliance tests are automated

---

## 🔍 Common Issues to Check

### High Priority

1. **Incorrect T-norm Implementation**
   - Using Product logic during inference (should be Łukasiewicz)
   - Missing epsilon in division operations

2. **Unbounded Polar Transformation**
   - arccos() called on values outside [-1, 1]
   - Missing normalization validation

3. **Race Conditions in Forward Service**
   - Shared mutable state without locks
   - Concurrent access to experience buffer

4. **Missing Circuit Breakers**
   - No emergency stop mechanism
   - Unbounded position accumulation

5. **Hardcoded Secrets**
   - API keys in source code
   - Credentials in configuration files

### Medium Priority

1. **Inefficient Memory Usage**
   - Large heap allocations in hot path
   - Missing buffer reuse

2. **Blocking Operations**
   - Synchronous I/O in async context
   - Database queries without timeouts

3. **Incorrect Schema Updates**
   - Missing L2 normalization before Qdrant insert
   - Cosine similarity on non-normalized vectors

4. **Missing Error Handling**
   - Network failures not retried
   - Malformed data crashing service

### Low Priority

1. **Logging Verbosity**
   - Debug logs in production
   - Missing structured fields

2. **Configuration Hardcoding**
   - Magic numbers instead of constants
   - Parameters not externalized

3. **Documentation Drift**
   - Comments not matching code
   - Missing equation references

---

## 📖 Additional Resources

### Documentation
- [JANUS README](../../janus/README.md)
- [CNS Architecture](../../janus/docs/CNS_ARCHITECTURE.md)
- [Audit README](./README.md)
- [LLM Audit Guide](./LLM_AUDIT_GUIDE.md)

### External References
- Technical Paper: https://github.com/nuniesmith/technical_papers
- Łukasiewicz Logic: https://en.wikipedia.org/wiki/Łukasiewicz_logic
- Gramian Angular Field: https://arxiv.org/abs/1506.00327
- Logic Tensor Networks: https://arxiv.org/abs/2012.13635
- UMAP: https://arxiv.org/abs/1802.03426
- Almgren-Chriss: https://www.math.nyu.edu/faculty/chriss/optliq_f.pdf

### Contact
- Issues: GitHub Issues
- Discussions: GitHub Discussions
- Documentation: `/docs` directory

---

**Last Updated**: 2025-01-XX  
**Version**: 1.0  
**Maintainer**: Jordan Smith