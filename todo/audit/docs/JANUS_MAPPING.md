# Janus Theoretical Framework Mapping

This document maps the audit service implementation to the theoretical structures defined in the [Project JANUS paper](https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex).

## Table of Contents

- [Overview](#overview)
- [Dual-Service Architecture](#dual-service-architecture)
- [Neuromorphic Brain Region Mapping](#neuromorphic-brain-region-mapping)
- [Forward Service (Real-Time Analysis)](#forward-service-real-time-analysis)
- [Backward Service (Consolidation)](#backward-service-consolidation)
- [Mathematical Foundations](#mathematical-foundations)
- [Implementation Guide](#implementation-guide)

---

## Overview

The audit service implements a **neuromorphic dual-service architecture** inspired by biological neural systems:

```
┌─────────────────────────────────────────────────┐
│         JANUS AUDIT SYSTEM ARCHITECTURE         │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌──────────────────┐  ┌─────────────────────┐ │
│  │ Forward Service  │  │  Backward Service   │ │
│  │ (Wake State)     │  │  (Sleep State)      │ │
│  ├──────────────────┤  ├─────────────────────┤ │
│  │ Static Analysis  │  │ LLM Consolidation   │ │
│  │ Pattern Detection│  │ Schema Formation    │ │
│  │ Real-time Tags   │  │ Pattern Learning    │ │
│  │ Task Generation  │  │ Memory Consolidation│ │
│  └────────┬─────────┘  └──────────┬──────────┘ │
│           │                       │             │
│           └───────────┬───────────┘             │
│                       │                         │
│           ┌───────────▼───────────┐             │
│           │  Memory Hierarchy     │             │
│           │ ─────────────────────│             │
│           │ Hippocampus (Recent)  │             │
│           │ SWR Buffer (Priority) │             │
│           │ Neocortex (Schemas)   │             │
│           └───────────────────────┘             │
└─────────────────────────────────────────────────┘
```

## Dual-Service Architecture

### Forward Service (Janus Bifrons)
**Paper Reference**: Part 2 - Forward Service

**Purpose**: Real-time code analysis during CI/CD execution (market hours analogy)

**Implementation Components**:
- `scanner.rs` - Pattern detection (Visual Cortex)
- `tags.rs` - Constraint checking (Prefrontal Cortex)
- `tasks.rs` - Decision engine (Basal Ganglia)
- `enhanced_scanner.rs` - Multi-modal fusion

**Key Operations**:
1. Extract code features (GAF transformation analogy)
2. Evaluate constraints (LTN constraint satisfaction)
3. Generate tasks (Dual-pathway decision)
4. Store episodes (Hippocampal memory)

### Backward Service (Janus Consivius)
**Paper Reference**: Part 3 - Backward Service

**Purpose**: Offline consolidation and learning (sleep state analogy)

**Implementation Components**:
- `llm.rs` - Deep analysis and pattern extraction
- `context.rs` - Experience bundling
- Future: Schema storage in vector database

**Key Operations**:
1. Prioritize experiences (SWR replay)
2. Cluster patterns (UMAP-like embedding)
3. Form schemas (Neocortical consolidation)
4. Update long-term knowledge

---

## Neuromorphic Brain Region Mapping

### Visual Cortex → Static Pattern Detection
**Paper Section**: 4.1 - Visual Cortex

**Biological Function**: Pattern recognition in visual data

**Audit Implementation**:
- **Component**: `scanner.rs`, `parser.rs`
- **Input**: Raw code files (time series analogy)
- **Processing**: AST parsing, regex patterns, structural analysis
- **Output**: Detected patterns (issues, tags, metrics)

**Mathematical Mapping**:
```rust
// Gramian Angular Field (GAF) analogy:
// Transform code structure into feature space
pub struct FeatureVector {
    pub loc: usize,              // Time series length
    pub complexity: f64,          // Signal complexity
    pub doc_coverage: f64,        // Structural coherence
    pub security_score: f64,      // Pattern stability
}
```

**Paper Equations**:
- GAF Generation: `G_ij = cos(φ_i + φ_j)` (Section 2.1.3)
- Maps to: Code structure correlation matrix

### Hippocampus → Recent Audit History
**Paper Section**: 4.2 - Hippocampus

**Biological Function**: Episodic memory and rapid encoding

**Audit Implementation**:
- **Component**: `HippocampalMemory` in `janus.rs`
- **Storage**: Recent audit findings (last N runs)
- **Capacity**: Configurable buffer size (default: 1000 episodes)
- **Eviction**: FIFO when capacity reached

**Mathematical Mapping**:
```rust
// Episodic buffer: D_hippo = {(s_t, a_t, r_t, s_{t+1}, c_t)}
pub struct Episode {
    pub state: ForwardState,      // s_t: code state
    pub action: String,            // a_t: task generated
    pub reward: f64,               // r_t: outcome quality
    pub next_state: Option<ForwardState>, // s_{t+1}
    pub context: ContextMetadata,  // c_t: environmental data
}
```

**Paper Equations**:
- Pattern Separation: `h_t = tanh(W_rand · [s_t; a_t; c_t])` (Section 3.1.2)

### Prefrontal Cortex → Constraint Validation
**Paper Section**: 4.4 - Prefrontal Cortex

**Biological Function**: Executive function and logical reasoning

**Audit Implementation**:
- **Component**: `tags.rs`, `LTNConstraint` in `janus.rs`
- **Constraints**: Audit tag rules (@audit-freeze, @audit-security, etc.)
- **Validation**: Lukasiewicz T-norm fuzzy logic
- **Output**: Constraint satisfaction scores [0, 1]

**Mathematical Mapping**:
```rust
// Logic Tensor Network (LTN) constraint: C_k = (P_k, w_k)
pub struct LTNConstraint {
    pub predicate: String,        // P_k: constraint name
    pub weight: f64,              // w_k: importance
    pub satisfaction: Option<f64>, // eval result ∈ [0,1]
}

// Łukasiewicz logic operations
impl LukasiewiczLogic {
    pub fn and(u: f64, v: f64) -> f64 {
        (u + v - 1.0).max(0.0)  // u ∧ v = max(0, u + v - 1)
    }
    
    pub fn implies(u: f64, v: f64) -> f64 {
        (1.0 - u + v).min(1.0)  // u ⇒ v = min(1, 1 - u + v)
    }
}
```

**Paper Equations**:
- Conjunction: `u ∧ v = max(0, u + v - 1)` (Section 2.3.1)
- Implication: `u ⇒ v = min(1, 1 - u + v)` (Section 2.3.4)

### Basal Ganglia → Task Generation
**Paper Section**: 4.3 - Basal Ganglia

**Biological Function**: Action selection and reinforcement learning

**Audit Implementation**:
- **Component**: `tasks.rs`, `DualPathwayDecision` in `janus.rs`
- **Direct Pathway**: Encourage task creation for high-priority issues
- **Indirect Pathway**: Inhibit tasks for low-priority/noisy issues
- **Output**: Binary decision (create task or skip)

**Mathematical Mapping**:
```rust
// Dual-pathway decision engine
pub struct DualPathwayDecision {
    pub direct_pathway: f64,      // d_direct: encouragement
    pub indirect_pathway: f64,    // d_indirect: inhibition
    pub inhibition_weight: f64,   // λ: balance parameter
    pub action_score: f64,        // final decision score
}

impl DualPathwayDecision {
    // Action selection: a_t = softmax(d_direct - λ * d_indirect)
    pub fn should_act(&self) -> bool {
        self.action_score > 0.5  // Threshold decision
    }
}
```

**Paper Equations**:
- Direct Pathway: `d_direct = ReLU(W_d * h + b_d)` (Section 4.3)
- Indirect Pathway: `d_indirect = ReLU(W_i * h + b_i)` (Section 4.3)
- Action: `a_t = softmax(d_direct - λ * d_indirect)` (Section 4.3)

### Amygdala → Critical Issue Detection
**Paper Section**: 4.5 - Amygdala

**Biological Function**: Threat detection and fear response

**Audit Implementation**:
- **Component**: Critical file detection in `types.rs`
- **Triggers**: Frozen code violations, security issues, kill_switch modifications
- **Response**: High-priority tasks, circuit breaker alerts

**Mathematical Mapping**:
```rust
// Mahalanobis distance for anomaly detection
// D_M(s_t) = sqrt((s_t - μ)^T Σ^-1 (s_t - μ))

pub fn is_critical_file(path: &str) -> bool {
    path.contains("kill_switch") ||
    path.contains("circuit_breaker") ||
    path.contains("conscience") ||
    // ... threat patterns
}
```

**Paper Equations**:
- Mahalanobis Distance: `D_M(s_t) = sqrt((s_t - μ)^T Σ^-1 (s_t - μ))` (Section 4.5)
- Circuit Breaker: `Trigger = 1 if D_M(s_t) > τ_danger` (Section 4.5)

### Cerebellum → Impact Prediction
**Paper Section**: 4.6 - Cerebellum

**Biological Function**: Predictive modeling and fine motor control

**Audit Implementation**:
- **Future**: Change impact analysis
- **Prediction**: Estimate downstream effects of code changes
- **Model**: Forward model of codebase dependencies

**Mathematical Mapping**:
```rust
// Forward model for impact prediction
// Δp = f_cerebellum(change_size, dependencies, volatility)

pub struct ImpactPrediction {
    pub affected_files: Vec<PathBuf>,
    pub risk_score: f64,
    pub confidence: f64,
}
```

**Paper Equations**:
- Forward Model: `Δp = f_cerebellum(order_size, liquidity, volatility)` (Section 4.6)

### Neocortex → Schema Learning
**Paper Section**: 4.1 - Neocortex

**Biological Function**: Abstract reasoning and long-term knowledge

**Audit Implementation**:
- **Component**: `NeocorticalSchemas` in `janus.rs`
- **Storage**: Learned code patterns (via LLM analysis)
- **Future**: Vector database integration (Qdrant)
- **Retrieval**: Similarity search for pattern matching

**Mathematical Mapping**:
```rust
// Schema representation: S_k = (id_k, z_k, M_k)
pub struct Schema {
    pub id: String,                 // id_k: unique identifier
    pub centroid: Vec<f64>,         // z_k: embedding vector
    pub member_count: usize,        // n_k: cluster size
    pub avg_reward: f64,            // r̄_k: average quality
    pub volatility: f64,            // σ_k: std deviation
    pub description: String,        // human-readable pattern
}
```

**Paper Equations**:
- Schema Centroid: `z_k = (1/|C_k|) Σ_{i∈C_k} h_i` (Section 3.3.1)
- Recall-Gated Update: `z_k ← z_k + η · 𝟙[recall_success] · (h_new - z_k)` (Section 3.3.2)

---

## Forward Service (Real-Time Analysis)

### State Representation
**Paper Reference**: Section 5.2 - Core Data Structures

**Mathematical Definition**:
```
S_t = (τ_t, f_t, O_t, c_t)
```

**Rust Implementation**:
```rust
pub struct ForwardState {
    pub timestamp: DateTime<Utc>,   // τ_t
    pub features: FeatureVector,    // f_t ∈ ℝ^d
    pub observation: Observation,   // O_t: file content/AST
    pub context: ContextMetadata,   // c_t: environment
}
```

### Feature Extraction Pipeline

**Step 1: Code Parsing**
- Parse source files into AST
- Extract structural metrics (LOC, complexity, etc.)
- Identify import/dependency graph

**Step 2: Feature Vector Construction**
```rust
pub struct FeatureVector {
    pub loc: usize,                    // Lines of code
    pub complexity: f64,                // Cyclomatic complexity
    pub doc_coverage: f64,              // Documentation ratio
    pub test_coverage: f64,             // Test coverage
    pub security_score: f64,            // Security rating [0,1]
    pub type_safety: f64,               // Type safety score [0,1]
    pub async_safety: f64,              // Async safety score [0,1]
    pub custom_metrics: HashMap<String, f64>, // Extensible
}
```

**Step 3: Constraint Evaluation**
```rust
// Evaluate all LTN constraints
let satisfaction: f64 = constraints
    .iter()
    .map(|c| c.evaluate(&state) * c.weight)
    .sum::<f64>() / total_weight;
```

**Step 4: Decision Engine**
```rust
// Dual-pathway decision
let decision = DualPathwayDecision::from_features(&state.features);

// Modulate by constraints
decision.action_score *= satisfaction;

// Generate task if threshold exceeded
if decision.should_act() {
    TaskGenerator::create_task(&state, &decision);
}
```

### Constraint Examples

#### Frozen Code Constraint
**Rule**: Never modify files marked with `@audit-freeze`

**Implementation**:
```rust
LTNConstraint {
    id: "frozen_code".to_string(),
    predicate: "frozen_code".to_string(),
    weight: 10.0,  // High importance
    satisfaction: None,
    variables: vec!["file_path", "tags"],
}

// Evaluation:
// ∀t: Modified(file) ∧ HasTag(file, "freeze") ⇒ Violation(Critical)
fn evaluate_frozen_code(state: &ForwardState) -> f64 {
    if has_freeze_tag(&state.observation.path) {
        0.0  // Violation
    } else {
        1.0  // Satisfied
    }
}
```

#### Security Constraint
**Rule**: All security-critical files must have high security scores

**Implementation**:
```rust
LTNConstraint {
    id: "security_critical".to_string(),
    predicate: "security".to_string(),
    weight: 5.0,
    satisfaction: None,
    variables: vec!["file_category", "security_score"],
}

// Evaluation:
// ∀f: IsCritical(f) ⇒ SecurityScore(f) ≥ 0.9
fn evaluate_security(state: &ForwardState) -> f64 {
    if is_critical_file(&state.observation.path) {
        state.features.security_score  // Direct score
    } else {
        1.0  // Non-critical files always pass
    }
}
```

---

## Backward Service (Consolidation)

### Three-Timescale Memory Hierarchy
**Paper Reference**: Section 3.1 - Memory Hierarchy

```
┌─────────────────────────────────────────────┐
│         Memory Consolidation Flow           │
├─────────────────────────────────────────────┤
│                                             │
│  ┌─────────────────────┐                   │
│  │ Hippocampus         │                   │
│  │ (Short-term)        │                   │
│  │ ─────────────────── │                   │
│  │ Recent episodes     │                   │
│  │ FIFO buffer         │                   │
│  │ Capacity: 1000      │                   │
│  └──────────┬──────────┘                   │
│             │                               │
│             │ Prioritization                │
│             ▼                               │
│  ┌─────────────────────┐                   │
│  │ SWR Buffer          │                   │
│  │ (Medium-term)       │                   │
│  │ ─────────────────── │                   │
│  │ Prioritized replay  │                   │
│  │ TD-error weighting  │                   │
│  │ Batch sampling      │                   │
│  └──────────┬──────────┘                   │
│             │                               │
│             │ Clustering                    │
│             ▼                               │
│  ┌─────────────────────┐                   │
│  │ Neocortex           │                   │
│  │ (Long-term)         │                   │
│  │ ─────────────────── │                   │
│  │ Learned schemas     │                   │
│  │ Pattern prototypes  │                   │
│  │ Vector DB storage   │                   │
│  └─────────────────────┘                   │
└─────────────────────────────────────────────┘
```

### Prioritized Experience Replay (SWR)
**Paper Reference**: Section 3.2 - SWR Simulator

**Algorithm**:
```rust
impl SWRBuffer {
    /// Add experience with TD-error priority
    /// Priority: p_i = |δ_i| + ε
    pub fn store(&mut self, episode: Episode, td_error: f64) {
        let priority = td_error.abs() + self.epsilon;
        self.experiences.push((episode, priority));
    }
    
    /// Sample batch with importance sampling
    /// P(i) = p_i^α / Σ_j p_j^α
    pub fn sample(&self, batch_size: usize) -> Vec<&Episode> {
        let total: f64 = self.experiences
            .iter()
            .map(|(_, p)| p.powf(self.alpha))
            .sum();
        
        // Weighted sampling (implementation simplified)
        let mut sampled = Vec::new();
        for _ in 0..batch_size {
            // Sample according to P(i)
            let sample = weighted_sample(&self.experiences, total);
            sampled.push(sample);
        }
        sampled
    }
}
```

**Paper Equations**:
- Priority: `p_i = |δ_i| + ε` where `δ_i = r_i + γ max_a' Q(s_{i+1}, a') - Q(s_i, a_i)` (Section 3.2.1)
- Sampling: `P(i) = p_i^α / Σ_j p_j^α` (Section 3.2.2)
- Importance Weight: `w_i = (1 / (N · P(i)))^β` (Section 3.2.3)

### Schema Formation
**Paper Reference**: Section 3.3 - UMAP Visualization

**Clustering Algorithm**:
```rust
impl NeocorticalSchemas {
    /// Form schemas from experience batch
    /// Algorithm: K-means clustering on embeddings
    pub fn consolidate(&mut self, experiences: Vec<Episode>, k: usize) {
        // Step 1: Extract embeddings
        let embeddings: Vec<Vec<f64>> = experiences
            .iter()
            .map(|e| self.embed(&e.state))
            .collect();
        
        // Step 2: K-means clustering
        let clusters = kmeans(&embeddings, k);
        
        // Step 3: Form schemas
        for (cluster_id, cluster) in clusters.iter().enumerate() {
            let centroid = compute_centroid(cluster);
            let members = cluster.len();
            let avg_reward = cluster.iter()
                .map(|i| experiences[*i].reward)
                .sum::<f64>() / members as f64;
            
            let schema = Schema {
                id: format!("schema_{}", cluster_id),
                centroid,
                member_count: members,
                avg_reward,
                volatility: compute_std_dev(cluster, experiences),
                description: self.generate_description(&centroid),
            };
            
            self.upsert(schema);
        }
    }
}
```

**Paper Equations**:
- K-means Objective: `min_C Σ_{k=1}^K Σ_{i∈C_k} ||h_i - z_k||^2` (Section 5.4)
- Schema Centroid: `z_k = (1/|C_k|) Σ_{i∈C_k} h_i` (Section 3.3.1)

### LLM Integration for Pattern Extraction
**Paper Reference**: Section 3 - Backward Service

**Workflow**:
```rust
// Backward consolidation workflow
pub async fn backward_consolidation(
    orchestrator: &mut JanusOrchestrator,
    llm_client: &LLMClient,
) -> Result<()> {
    // Step 1: Sample prioritized experiences
    let batch = orchestrator.memory.swr_buffer
        .sample(orchestrator.config.swr_batch_size);
    
    // Step 2: Bundle context for LLM
    let context = bundle_experiences(batch);
    
    // Step 3: LLM analysis (extract patterns)
    let patterns = llm_client
        .analyze_patterns(&context)
        .await?;
    
    // Step 4: Form schemas
    for pattern in patterns {
        let schema = Schema {
            id: pattern.id,
            centroid: pattern.embedding,
            member_count: pattern.support,
            avg_reward: pattern.quality_score,
            volatility: pattern.variance,
            description: pattern.description,
        };
        
        orchestrator.memory.neocortex.upsert(schema);
    }
    
    Ok(())
}
```

---

## Mathematical Foundations

### Łukasiewicz T-Norm Logic
**Paper Reference**: Section 2.3 - Lukasiewicz T-Norm Operations

**Truth Values**: `[0, 1]` continuous fuzzy logic

**Operations**:
```rust
impl LukasiewiczLogic {
    /// Conjunction (AND): u ∧ v = max(0, u + v - 1)
    pub fn and(u: f64, v: f64) -> f64 {
        (u + v - 1.0).max(0.0)
    }
    
    /// Disjunction (OR): u ∨ v = min(1, u + v)
    pub fn or(u: f64, v: f64) -> f64 {
        (u + v).min(1.0)
    }
    
    /// Negation (NOT): ¬u = 1 - u
    pub fn not(u: f64) -> f64 {
        1.0 - u
    }
    
    /// Implication (IF-THEN): u ⇒ v = min(1, 1 - u + v)
    pub fn implies(u: f64, v: f64) -> f64 {
        (1.0 - u + v).min(1.0)
    }
}
```

**Example Constraint**:
```rust
// Rule: "If file is critical, then security_score must be high"
// ∀f: IsCritical(f) ⇒ SecurityScore(f) ≥ 0.9

let is_critical = if is_critical_file(path) { 1.0 } else { 0.0 };
let security_ok = if security_score >= 0.9 { 1.0 } else { 0.0 };

let satisfaction = LukasiewiczLogic::implies(is_critical, security_ok);
// If critical and security low: implies(1.0, 0.0) = 0.0 (violation)
// If critical and security high: implies(1.0, 1.0) = 1.0 (satisfied)
// If not critical: implies(0.0, x) = 1.0 (always satisfied)
```

### Cosine Similarity for Schema Retrieval
**Paper Reference**: Section 3.4 - Similarity Search

**Definition**:
```
cosine(a, b) = (a · b) / (||a|| ||b||)
distance(a, b) = 1 - cosine(a, b)
```

**Implementation**:
```rust
fn cosine_distance(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    
    1.0 - (dot / (norm_a * norm_b))
}

// Retrieve nearest schema
pub fn find_nearest(&self, embedding: &[f64]) -> Option<&Schema> {
    self.schemas.iter().min_by(|a, b| {
        let dist_a = cosine_distance(&a.centroid, embedding);
        let dist_b = cosine_distance(&b.centroid, embedding);
        dist_a.partial_cmp(&dist_b).unwrap()
    })
}
```

---

## Implementation Guide

### Quick Start: Using Janus Structures

```rust
use fks_audit::janus::*;

// 1. Create orchestrator
let config = JanusConfig::default();
let mut orchestrator = JanusOrchestrator::new(config);

// 2. Define constraints
orchestrator.add_constraint(LTNConstraint {
    id: "frozen_code".to_string(),
    predicate: "frozen_code".to_string(),
    weight: 10.0,
    satisfaction: None,
    variables: vec!["file_path".to_string()],
});

// 3. Analyze code (forward pass)
let state = ForwardState {
    timestamp: Utc::now(),
    features: extract_features(&file),
    observation: parse_file(&file),
    context: build_context(&repo),
};

let decision = orchestrator.forward_pass(state);

if decision.should_act() {
    println!("Generating task for file: {}", file.path);
}

// 4. Consolidate patterns (backward pass)
orchestrator.backward_pass();
```

### Integration with Existing Audit Service

**Step 1: Augment Scanner with Feature Extraction**
```rust
// In scanner.rs
impl Scanner {
    pub fn analyze_with_features(&self, path: &Path) -> Result<ForwardState> {
        let features = self.extract_features(path)?;
        let observation = self.parse_file(path)?;
        let context = self.build_context()?;
        
        Ok(ForwardState {
            timestamp: Utc::now(),
            features,
            observation,
            context,
        })
    }
}
```

**Step 2: Add Constraint Validation to Tag Scanner**
```rust
// In tags.rs
impl TagScanner {
    pub fn validate_constraints(&self, tags: &[AuditTag]) -> Vec<LTNConstraint> {
        tags.iter()
            .filter(|t| matches!(t.tag_type, AuditTagType::Freeze | AuditTagType::Security))
            .map(|t| self.tag_to_constraint(t))
            .collect()
    }
}
```

**Step 3: Integrate Dual-Pathway Decision in Task Generator**
```rust
// In tasks.rs
impl TaskGenerator {
    pub fn generate_with_decision(
        &self,
        state: &ForwardState,
        decision: &DualPathwayDecision,
    ) -> Option<Task> {
        if !decision.should_act() {
            return None;
        }
        
        Some(Task::new(
            format!("Review {}", state.observation.path.display()),
            format!("Action score: {:.2}", decision.action_score),
            state.observation.path.clone(),
            None,
            self.determine_priority(&decision),
            Category::from_path(&state.observation.path.to_string_lossy()),
        ))
    }
}
```

### Vector Database Integration (Future)

**Schema Storage in Qdrant**:
```rust
use qdrant_client::prelude::*;

pub struct SchemaStore {
    client: QdrantClient,
    collection: String,
}

impl SchemaStore {
    pub async fn store_schema(&self, schema: &Schema) -> Result<()> {
        let point = PointStruct::new(
            schema.id.clone(),
            schema.centroid.clone(),
            json!({
                "member_count": schema.member_count,
                "avg_reward": schema.avg_reward,
                "volatility": schema.volatility,
                "description": schema.description,
            }),
        );
        
        self.client
            .upsert_points(&self.collection, vec![point], None)
            .await?;
        
        Ok(())
    }
    
    pub async fn search_similar(&self, embedding: &[f64], limit: usize) -> Result<Vec<Schema>> {
        let results = self.client
            .search_points(&SearchPoints {
                collection_name: self.collection.clone(),
                vector: embedding.to_vec(),
                limit: limit as u64,
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await?;
        
        // Convert results to Schema objects
        Ok(results.result.into_iter().map(|r| self.point_to_schema(r)).collect())
    }
}
```

---

## References

1. **Janus Paper**: [Project JANUS: Neuromorphic Trading Intelligence](https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex)
2. **Part 2**: Forward Service (Janus Bifrons)
3. **Part 3**: Backward Service (Janus Consivius)
4. **Part 4**: Neuromorphic Architecture
5. **Part 5**: Rust Implementation

---

## Next Steps

1. ✅ **Completed**: Core Janus structures implemented in `janus.rs`
2. 🔄 **In Progress**: Integration with existing scanner/task generator
3. 📋 **Planned**: 
   - Vector database integration for schema storage
   - LLM pattern extraction pipeline
   - UMAP visualization dashboard
   - Parametric UMAP for real-time embedding
   - Full backward consolidation workflow

---

## Contributing

When adding new features, ensure they map to the theoretical framework:

1. Identify the brain region (Visual Cortex, Hippocampus, etc.)
2. Reference the corresponding paper section
3. Implement the mathematical formulation
4. Document the mapping in this file
5. Add tests that validate the theoretical properties

**Example**:
```rust
/// Implements Amygdala threat detection (Section 4.5)
/// 
/// Mahalanobis distance: D_M(s_t) = sqrt((s_t - μ)^T Σ^-1 (s_t - μ))
/// Circuit breaker triggers when D_M(s_t) > τ_danger
pub fn detect_threat(state: &ForwardState, threshold: f64) -> bool {
    let distance = mahalanobis_distance(&state.features);
    distance > threshold
}
```
