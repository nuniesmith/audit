# Technical Paper Integration Guide

> **How to leverage the JANUS technical paper for comprehensive code audits**

---

## 📋 Overview

This guide explains how to use the JANUS technical paper as a reference during code audits to verify mathematical correctness, architectural compliance, and implementation fidelity.

**Technical Paper Location**: https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex

---

## 🎯 Quick Start

### For LLM Audits

When triggering an LLM audit via GitHub Actions, the system automatically provides context from this guide. However, you can enhance results by:

1. **Focus on Mathematical Components**:
   ```yaml
   Focus Areas: mathematical,architecture,neuromorphic
   ```

2. **Reference Specific Sections**:
   - Add comments in code referencing paper sections
   - Tag implementations with equation numbers

3. **Use Deep Analysis Mode**:
   ```yaml
   Analysis Depth: deep
   Batch Size: 5  # More detailed per-file analysis
   ```

### For Manual Audits

When reviewing code manually:

1. **Open the technical paper** alongside your IDE
2. **Find the relevant section** using the mapping table below
3. **Compare implementation** against mathematical specifications
4. **Verify edge cases** are handled per paper requirements

---

## 📖 Paper Structure Reference

### Part 1: Main Architecture

**Focus**: High-level design philosophy

**Key Sections**:
- Crisis of complexity in quantitative trading
- Dual-service design rationale
- Memory hierarchy overview
- Rust-first implementation strategy

**Audit Usage**:
- Verify architectural decisions align with philosophy
- Check service boundaries are maintained
- Ensure hot/cold path separation

---

### Part 2: Forward Service (Janus Bifrons)

**Focus**: Real-time execution and decision-making

#### Section 1: Visual Pattern Recognition

**Location**: Part 2, Section 1

**Key Equations**:
- **Eq. 1.1.1**: Learnable normalization with tanh
  ```
  x̃ₜ = tanh(γ ⊙ (xₜ - μ)/σ + β)
  ```
  
- **Eq. 1.1.2**: Polar coordinate transformation
  ```
  φₜ = arccos(x̃ₜ) ∈ [0, π]
  ```
  
- **Eq. 1.1.3**: GASF matrix construction
  ```
  Gᵢⱼ = cos(φᵢ + φⱼ)
  ```

**Implementation File**: `crates/vision/src/gaf.rs`

**Audit Checklist**:
- [ ] Normalization ensures |x̃ₜ| < 1 (critical for arccos)
- [ ] γ, β parameters are learnable (not hardcoded)
- [ ] Running statistics (μ, σ) are updated correctly
- [ ] Gramian matrix is symmetric
- [ ] Edge case: Handle all-constant time series

**Common Issues**:
```rust
// ❌ WRONG: Fixed min-max normalization
x_norm = (x - min) / (max - min) * 2.0 - 1.0;

// ✅ CORRECT: Learnable affine transform + tanh
x_norm = (gamma * (x - mu) / sigma + beta).tanh();
```

---

#### Section 2: Logic Tensor Networks

**Location**: Part 2, Section 2

**Key Equations**:
- **Eq. 2.3.1**: Łukasiewicz conjunction (inference)
  ```
  a ∧ b = max(0, a + b - 1)
  ```
  
- **Eq. 2.3.2**: Łukasiewicz implication (inference)
  ```
  a ⇒ b = min(1, 1 - a + b)
  ```
  
- **Eq. 2.3.3**: Product conjunction (training)
  ```
  a ∧ b = a × b
  ```

**Implementation File**: `crates/logic/src/ltn.rs`

**Audit Checklist**:
- [ ] Inference uses Łukasiewicz t-norms
- [ ] Training uses Product t-norms (for gradient flow)
- [ ] Mode switching is explicit and documented
- [ ] Numerical stability (add epsilon where needed)
- [ ] Wash sale constraint is correctly encoded

**Common Issues**:
```rust
// ❌ WRONG: Using Product logic during inference
fn and_inference(a: f64, b: f64) -> f64 {
    a * b  // No gradient flow needed during inference!
}

// ✅ CORRECT: Łukasiewicz for inference
fn and_inference(a: f64, b: f64) -> f64 {
    (a + b - 1.0).max(0.0)
}
```

---

#### Section 3: Multimodal Fusion

**Location**: Part 2, Section 3

**Key Equations**:
- **Eq. 3.1**: Cross-attention mechanism
- **Eq. 3.2**: Gating function for modality weighting

**Implementation File**: `services/forward/src/fusion.rs`

**Audit Checklist**:
- [ ] Visual, temporal, and textual inputs are properly aligned
- [ ] Attention weights sum to 1.0
- [ ] Gating mechanism prevents mode collapse
- [ ] Gradient flow to all modalities

---

#### Section 4: Basal Ganglia Decision Engine

**Location**: Part 2, Section 4

**Key Equations**:
- **Eq. 4.1**: Direct pathway (Go signal)
- **Eq. 4.2**: Indirect pathway (No-Go signal)
- **Eq. 4.3**: Action selection with inhibition

**Implementation File**: `services/forward/src/decision.rs`

**Audit Checklist**:
- [ ] Dual pathways compete via softmax
- [ ] Inhibition weight λ is configurable
- [ ] No single pathway dominates (check gradients)
- [ ] Cerebellar forward model integrated

---

### Part 3: Backward Service (Janus Consivius)

**Focus**: Memory consolidation and learning

#### Section 1: Three-Timescale Memory

**Location**: Part 3, Section 1

**Components**:
- Hippocampus (short-term buffer)
- Sharp-Wave Ripple (medium-term replay)
- Neocortex (long-term schemas)

**Implementation Files**:
- `crates/memory/src/hippocampus.rs`
- `crates/memory/src/swr.rs`
- `services/backward/src/consolidation.rs`

**Audit Checklist**:
- [ ] Hippocampus buffer has fixed capacity
- [ ] Pattern separation uses random projections
- [ ] SWR replay is prioritized (not random)
- [ ] Schema updates are recall-gated

---

#### Section 2: Prioritized Experience Replay

**Location**: Part 3, Section 2.2

**Key Equations**:
- **Eq. 2.2.1**: Priority calculation
  ```
  pᵢ = |δᵢ| + ε
  ```
  
- **Eq. 2.2.2**: Sampling probability
  ```
  P(i) = pᵢᵅ / Σⱼ pⱼᵅ
  ```
  
- **Eq. 2.2.3**: Importance weights
  ```
  wᵢ = (1 / (N × P(i)))ᵝ
  ```

**Implementation File**: `crates/memory/src/per.rs`

**Audit Checklist**:
- [ ] TD-error includes epsilon (ε = 10⁻⁶)
- [ ] Alpha (α) is configurable, typically 0.6
- [ ] Beta (β) anneals from 0.4 to 1.0
- [ ] Priority updates after gradient step
- [ ] Sum-tree for O(log N) sampling

**Common Issues**:
```rust
// ❌ WRONG: Missing epsilon causes divide-by-zero
priority = td_error.abs();

// ✅ CORRECT: Epsilon ensures numerical stability
priority = td_error.abs() + 1e-6;
```

---

#### Section 3: UMAP Visualization

**Location**: Part 3, Section 3

**Key Equations**:
- **Eq. 3.1**: UMAP loss function
- **Eq. 3.2**: Attraction term
- **Eq. 3.3**: Repulsion term (negative sampling)

**Implementation File**: `services/backward/src/umap.rs`

**Audit Checklist**:
- [ ] AlignedUMAP maintains consistency across sleep cycles
- [ ] Negative sampling ratio is 5:1
- [ ] Parametric UMAP for new points
- [ ] Dimensionality matches Qdrant expectations

---

#### Section 4: Qdrant Integration

**Location**: Part 3, Section 4

**Key Requirements**:
- L2 normalization before insertion
- Cosine similarity search
- Schema metadata (count, reward, volatility)

**Implementation File**: `crates/memory/src/qdrant.rs`

**Audit Checklist**:
- [ ] Vectors are L2-normalized
- [ ] Metadata includes n, r̄, σ
- [ ] Search uses cosine distance
- [ ] Collection creation is idempotent

**Common Issues**:
```rust
// ❌ WRONG: Inserting non-normalized vectors
qdrant.upsert(id, vector, metadata).await?;

// ✅ CORRECT: L2 normalize first
let normalized = vector / vector.norm_l2();
qdrant.upsert(id, normalized, metadata).await?;
```

---

### Part 4: Neuromorphic Architecture

**Focus**: Brain-region mapping justifications

**Key Sections**:
- Visual Cortex → GAF/ViViT
- Hippocampus → Experience buffer
- Prefrontal Cortex → LTN
- Basal Ganglia → Decision engine
- Cerebellum → Market impact model
- Amygdala → Circuit breakers

**Audit Usage**:
- Verify component names match brain regions
- Check functional responsibilities align
- Ensure biological plausibility

---

### Part 5: Rust Implementation

**Focus**: Production deployment specifications

**Key Sections**:
- ML framework strategy (PyTorch → ONNX → Rust)
- Performance requirements
- Service architecture
- Docker deployment

**Audit Checklist**:
- [ ] ONNX models are loaded correctly
- [ ] Async runtime uses Tokio
- [ ] Forward service: p99 < 10ms
- [ ] Zero-copy Arrow IPC for service communication

---

## 🔍 Equation-to-Code Mapping

### Quick Reference Table

| Paper Equation | File Location | Function/Struct |
|----------------|---------------|-----------------|
| Part 2, Eq. 1.1.1 | `crates/vision/src/gaf.rs` | `normalize_learnable()` |
| Part 2, Eq. 1.1.3 | `crates/vision/src/gaf.rs` | `compute_gasf()` |
| Part 2, Eq. 2.3.1 | `crates/logic/src/ltn.rs` | `lukasiewicz_and()` |
| Part 2, Eq. 2.3.2 | `crates/logic/src/ltn.rs` | `lukasiewicz_implies()` |
| Part 3, Eq. 2.2.1 | `crates/memory/src/per.rs` | `compute_priority()` |
| Part 3, Eq. 2.2.2 | `crates/memory/src/per.rs` | `sample_batch()` |
| Part 4, Eq. 2.6.1 | `services/forward/src/amygdala.rs` | `mahalanobis_distance()` |

---

## 🎓 Common Verification Patterns

### Pattern 1: Mathematical Formula Audit

**Steps**:
1. Find paper equation (e.g., Part 2, Eq. 2.3.1)
2. Locate implementation file
3. Compare code line-by-line with equation
4. Check edge cases (division by zero, domain violations)
5. Verify numerical stability (epsilon values)

**Example**:
```rust
// Paper: Part 2, Eq. 2.3.1
// a ∧ b = max(0, a + b - 1)

// Code audit:
fn lukasiewicz_and(a: f64, b: f64) -> f64 {
    // ✓ Correct implementation
    // ✓ Handles a,b ∈ [0,1]
    // ✓ Returns value ∈ [0,1]
    (a + b - 1.0).max(0.0)
}
```

---

### Pattern 2: Architecture Compliance Audit

**Steps**:
1. Identify component in code
2. Find corresponding brain region in Part 4
3. Verify responsibilities match paper description
4. Check service boundary placement (Forward vs. Backward)

**Example**:
```rust
// Component: BasalGangliaDecision
// Paper: Part 4, Section 2.3 (Basal Ganglia)
// Expected: Action selection via dual pathways

// Audit:
// ✓ Direct pathway exists
// ✓ Indirect pathway exists
// ✓ Competition via softmax
// ✓ Located in Forward service (correct boundary)
```

---

### Pattern 3: Performance Requirement Audit

**Steps**:
1. Find performance spec in Part 5
2. Locate code implementation
3. Check for blocking operations
4. Verify allocations are minimized
5. Run benchmarks if needed

**Example**:
```rust
// Paper: Part 5 - Forward service p99 < 10ms

// Audit checklist:
// ✓ No blocking I/O in hot path
// ✓ Pre-allocated buffers used
// ✓ SIMD operations for vectors
// ✗ Missing benchmark test! (add TODO)
```

---

## 🚨 Critical Verification Points

### Priority 1: Safety

- [ ] Circuit breaker thresholds match Part 4, Section 2.6
- [ ] Position limits are enforced
- [ ] Wash sale constraints are correct
- [ ] API keys are never hardcoded

### Priority 2: Correctness

- [ ] GAF normalization prevents arccos domain errors
- [ ] LTN uses correct t-norms for mode (train/infer)
- [ ] PER priorities include epsilon
- [ ] Qdrant vectors are L2-normalized

### Priority 3: Performance

- [ ] Forward service has no blocking I/O
- [ ] Backward service runs only during market close
- [ ] Zero-copy IPC for service communication
- [ ] Memory usage within limits

---

## 📝 Audit Report Format

When reporting issues found via paper comparison:

```markdown
**Issue**: [Brief description]
**File**: [Path to file]
**Line**: [Line number]
**Severity**: [Critical/High/Medium/Low]
**Category**: [Mathematics/Architecture/Performance/Safety]
**Paper Reference**: [Part X, Section Y, Equation Z]
**Current Implementation**:
```rust
// Current code
```
**Expected Implementation** (from paper):
```rust
// Corrected code matching paper
```
**Impact**: [What could go wrong]
**Recommendation**: [How to fix]
```

---

## 🔗 Additional Resources

### External Papers

- **Gramian Angular Field**: [Wang & Oates, 2015](https://arxiv.org/abs/1506.00327)
- **Logic Tensor Networks**: [Badreddine et al., 2020](https://arxiv.org/abs/2012.13635)
- **Prioritized Experience Replay**: [Schaul et al., 2015](https://arxiv.org/abs/1511.05952)
- **UMAP**: [McInnes et al., 2018](https://arxiv.org/abs/1802.03426)
- **Almgren-Chriss**: [Almgren & Chriss, 2000](https://www.math.nyu.edu/faculty/chriss/optliq_f.pdf)

### Internal Documentation

- [JANUS Context](./JANUS_CONTEXT.md) - Quick reference for audits
- [LLM Audit Guide](./LLM_AUDIT_GUIDE.md) - How to run AI-powered audits
- [Audit README](./README.md) - Service overview

---

## 🤖 LLM Integration

### Prompt Template

When using LLMs for technical paper verification:

```
You are auditing code against the JANUS technical paper specifications.

PAPER REFERENCE:
https://raw.githubusercontent.com/nuniesmith/technical_papers/main/project_janus/janus.tex

TASK:
Review the following code and verify it matches the mathematical specifications in:
- Part [X], Section [Y], Equation [Z]

CODE:
[paste code here]

VERIFICATION CHECKLIST:
1. Does the implementation match the paper equation exactly?
2. Are edge cases handled (domain violations, numerical stability)?
3. Are there any deviations from the specification?
4. If deviations exist, are they justified and documented?

OUTPUT FORMAT:
- Compliance: [Pass/Fail/Partial]
- Issues Found: [List any problems]
- Recommendations: [Suggested fixes]
```

---

**Last Updated**: 2025-01-XX  
**Version**: 1.0  
**Maintainer**: Jordan Smith