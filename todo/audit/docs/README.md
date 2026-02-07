# Audit Service Janus Framework Documentation

This directory contains comprehensive documentation for the Janus theoretical framework integration with the audit service.

## Overview

The audit service implements a **neuromorphic dual-service architecture** based on the [Project JANUS paper](https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex). This brain-inspired design enables sophisticated code analysis through biological principles.

## Documentation Files

### [JANUS_MAPPING.md](./JANUS_MAPPING.md)
**Complete theoretical framework mapping**

This is the primary reference document that maps every component of the audit service to its theoretical counterpart in the Janus paper.

**Contents**:
- Dual-service architecture (Forward/Backward)
- Neuromorphic brain region mapping
- Mathematical foundations
- Implementation guide with code examples
- Integration patterns

**Use this when**:
- Understanding the theoretical foundation
- Implementing new features that align with the framework
- Explaining the system architecture to stakeholders
- Contributing to the codebase

## Quick Reference

### Core Concepts

| Concept | Paper Reference | Implementation | Purpose |
|---------|----------------|----------------|---------|
| Forward Service | Part 2 | `scanner.rs`, `tags.rs`, `tasks.rs` | Real-time static analysis |
| Backward Service | Part 3 | `llm.rs`, `context.rs` | Offline consolidation & learning |
| Visual Cortex | Section 4.1 | `scanner.rs`, `parser.rs` | Pattern detection |
| Hippocampus | Section 4.2 | `HippocampalMemory` | Recent audit history |
| Prefrontal Cortex | Section 4.4 | `LTNConstraint`, `tags.rs` | Constraint validation |
| Basal Ganglia | Section 4.3 | `DualPathwayDecision`, `tasks.rs` | Task generation |
| Amygdala | Section 4.5 | Critical file detection | Threat/issue detection |
| Neocortex | Section 4.1 | `NeocorticalSchemas` | Pattern learning |

### Brain Region → Code Analysis Mapping

```
Visual Cortex       → Static code pattern detection (AST, regex, metrics)
Hippocampus         → Recent audit findings buffer (FIFO, capacity: 1000)
Prefrontal Cortex   → Audit tag constraint checking (LTN, Łukasiewicz logic)
Basal Ganglia       → Task generation decision engine (dual-pathway)
Cerebellum          → Change impact prediction (future)
Amygdala            → Critical issue alerts (frozen code, security)
Neocortex           → Learned schemas from LLM analysis
```

### Mathematical Foundations

#### Łukasiewicz T-Norm Logic
Used for fuzzy constraint satisfaction:

```rust
// Conjunction (AND): u ∧ v = max(0, u + v - 1)
LukasiewiczLogic::and(0.8, 0.7) // = 0.5

// Implication (IF-THEN): u ⇒ v = min(1, 1 - u + v)
LukasiewiczLogic::implies(is_critical, security_ok)
```

#### State Representation
```
S_t = (τ_t, f_t, O_t, c_t)
where:
  τ_t = timestamp
  f_t = feature vector (LOC, complexity, coverage, etc.)
  O_t = observation (file content, AST)
  c_t = context (repo, branch, CI info)
```

#### Memory Hierarchy
```
Hippocampus (short-term)
    ↓ prioritization
SWR Buffer (medium-term)
    ↓ clustering
Neocortex (long-term schemas)
```

## Code Examples

### Example 1: Running the Integration Demo

```bash
cd fks/src/audit
cargo run --example janus_integration
```

This demonstrates:
- Constraint definition (LTN)
- Forward pass (real-time analysis)
- Dual-pathway decision making
- Backward pass (consolidation)
- Łukasiewicz logic operations

### Example 2: Using Janus Structures in Code

```rust
use fks_audit::janus::*;

// 1. Create orchestrator
let config = JanusConfig::default();
let mut orchestrator = JanusOrchestrator::new(config);

// 2. Add constraints
orchestrator.add_constraint(LTNConstraint {
    id: "frozen_code".to_string(),
    predicate: "frozen_code".to_string(),
    weight: 10.0,
    satisfaction: None,
    variables: vec!["file_path".to_string()],
});

// 3. Analyze code
let state = ForwardState {
    timestamp: Utc::now(),
    features: extract_features(&file),
    observation: parse_file(&file),
    context: build_context(&repo),
};

let decision = orchestrator.forward_pass(state);

if decision.should_act() {
    generate_task(&state, &decision);
}

// 4. Consolidate (offline)
orchestrator.backward_pass();
```

### Example 3: Constraint Validation

```rust
// Define constraint: "Critical files must have high security"
// ∀f: IsCritical(f) ⇒ SecurityScore(f) ≥ 0.9

let is_critical = if is_critical_file(path) { 1.0 } else { 0.0 };
let security_ok = if security_score >= 0.9 { 1.0 } else { 0.0 };

let satisfaction = LukasiewiczLogic::implies(is_critical, security_ok);

if satisfaction < 1.0 {
    // Constraint violated - generate critical task
    create_security_task(path, security_score);
}
```

## Architecture Diagrams

### Dual-Service Architecture

```
┌─────────────────────────────────────────────────┐
│            JANUS AUDIT ARCHITECTURE             │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌──────────────────┐  ┌─────────────────────┐ │
│  │ Forward Service  │  │  Backward Service   │ │
│  │ (Wake/Real-time) │  │  (Sleep/Offline)    │ │
│  ├──────────────────┤  ├─────────────────────┤ │
│  │ • Static Analysis│  │ • LLM Analysis      │ │
│  │ • Pattern Detect │  │ • Schema Formation  │ │
│  │ • Constraint Val │  │ • Pattern Learning  │ │
│  │ • Task Generate  │  │ • Consolidation     │ │
│  └────────┬─────────┘  └──────────┬──────────┘ │
│           │                       │             │
│           └───────────┬───────────┘             │
│                       ▼                         │
│           ┌───────────────────────┐             │
│           │  Memory Hierarchy     │             │
│           │ ───────────────────── │             │
│           │ • Hippocampus (1000)  │             │
│           │ • SWR Buffer (PER)    │             │
│           │ • Neocortex (Schemas) │             │
│           └───────────────────────┘             │
└─────────────────────────────────────────────────┘
```

### Forward Pass Flow

```
Code File
   │
   ▼
[Visual Cortex]────► Pattern Detection
   │                  (AST, metrics)
   ▼
[Feature Extraction]─► FeatureVector
   │                   (LOC, complexity, security, etc.)
   ▼
[Prefrontal Cortex]──► Constraint Validation
   │                   (LTN, Łukasiewicz logic)
   ▼
[Basal Ganglia]──────► Decision Engine
   │                   (Dual-pathway: direct vs indirect)
   ▼
[Amygdala Check]─────► Critical Issue Detection
   │
   ▼
Task Generation (if decision.should_act())
   │
   ▼
[Hippocampus]────────► Store Episode
```

### Backward Pass Flow

```
Hippocampus Episodes
   │
   ▼
[SWR Prioritization]─► TD-error based sampling
   │                    P(i) = p_i^α / Σ p_j^α
   ▼
[Batch Sampling]─────► Select high-priority experiences
   │
   ▼
[LLM Analysis]───────► Pattern extraction
   │                    (Deep reasoning)
   ▼
[Clustering]─────────► K-means on embeddings
   │                    min Σ ||h_i - z_k||²
   ▼
[Schema Formation]───► Compute centroids & metadata
   │
   ▼
[Neocortex Update]───► Store learned patterns
```

## Integration Guide

### Step 1: Understand the Theory
1. Read the [Janus paper](https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex)
2. Review [JANUS_MAPPING.md](./JANUS_MAPPING.md) for implementation details
3. Run the integration example: `cargo run --example janus_integration`

### Step 2: Identify Brain Region
When adding a feature, determine which brain region it maps to:
- **Pattern detection** → Visual Cortex
- **Constraint checking** → Prefrontal Cortex
- **Decision making** → Basal Ganglia
- **Critical alerts** → Amygdala
- **Learning patterns** → Neocortex

### Step 3: Implement with Framework
Use the Janus structures from `janus.rs`:

```rust
use fks_audit::janus::*;

// For pattern detection (Visual Cortex)
let state = ForwardState { /* ... */ };

// For constraints (Prefrontal Cortex)
let constraint = LTNConstraint { /* ... */ };

// For decisions (Basal Ganglia)
let decision = DualPathwayDecision::from_features(&features);

// For memory (Hippocampus)
orchestrator.memory.hippocampus.store(episode);
```

### Step 4: Document the Mapping
Update [JANUS_MAPPING.md](./JANUS_MAPPING.md) with:
- Brain region reference
- Paper section citation
- Mathematical formulation
- Implementation code
- Tests validating theoretical properties

## Testing

### Unit Tests
```bash
cd fks/src/audit
cargo test janus::
```

Tests validate:
- Łukasiewicz logic operations
- Dual-pathway decision thresholds
- Hippocampal buffer eviction (FIFO)
- Cosine distance for schema retrieval

### Integration Tests
```bash
cargo run --example janus_integration
```

Demonstrates end-to-end workflow.

## Future Enhancements

### Planned Features
1. **Vector Database Integration** (Qdrant)
   - Store schemas in vector database
   - Similarity search for pattern matching
   - Persistent long-term memory

2. **UMAP Visualization**
   - Embed experiences in 2D space
   - Visualize schema clusters
   - Real-time monitoring dashboard

3. **Full Backward Pipeline**
   - Automated LLM pattern extraction
   - Recall-gated consolidation
   - Parametric UMAP for new experiences

4. **Cerebellum Forward Model**
   - Predict change impact
   - Estimate downstream effects
   - Risk scoring for modifications

## References

### Primary Sources
1. [Project JANUS Paper](https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex)
   - Part 2: Forward Service (Janus Bifrons)
   - Part 3: Backward Service (Janus Consivius)
   - Part 4: Neuromorphic Architecture
   - Part 5: Rust Implementation

### Implementation
- `src/janus.rs` - Core framework structures
- `examples/janus_integration.rs` - Usage examples
- `docs/JANUS_MAPPING.md` - Complete mapping documentation

### Related Documentation
- `../JANUS_AUDIT_INDEX.md` - High-level overview
- `../JANUS_CONTEXT.md` - System context
- `../JANUS_LLM_SYSTEM_PROMPT.md` - LLM integration

## Contributing

When contributing, ensure alignment with the theoretical framework:

1. **Identify**: Which brain region does this feature belong to?
2. **Reference**: Cite the relevant paper section
3. **Implement**: Use mathematical formulations from the paper
4. **Document**: Update JANUS_MAPPING.md with the mapping
5. **Test**: Validate theoretical properties with tests

### Example Contribution Workflow

```rust
/// Implements Amygdala threat detection (Section 4.5)
/// 
/// Mahalanobis distance: D_M(s_t) = sqrt((s_t - μ)^T Σ^-1 (s_t - μ))
/// Circuit breaker triggers when D_M(s_t) > τ_danger
/// 
/// # Arguments
/// * `state` - Current system state
/// * `threshold` - Danger threshold τ
/// 
/// # Returns
/// `true` if threat detected, `false` otherwise
#[brain_region(Amygdala)]
pub fn detect_threat(state: &ForwardState, threshold: f64) -> bool {
    let distance = mahalanobis_distance(&state.features);
    distance > threshold
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_threat_detection_threshold() {
        // Validates Section 4.5 equations
        // ...
    }
}
```

## Questions?

- **Theoretical questions**: See [JANUS_MAPPING.md](./JANUS_MAPPING.md)
- **Implementation questions**: See code in `src/janus.rs`
- **Usage questions**: Run `cargo run --example janus_integration`
- **Paper questions**: Read [Project JANUS](https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex)

---

**Last Updated**: 2025-12-28  
**Framework Version**: 1.0  
**Paper Version**: Complete Technical Specification