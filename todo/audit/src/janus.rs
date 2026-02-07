//! Janus Theoretical Framework Mapping
//!
//! This module implements the theoretical structures from the Janus paper:
//! "Project JANUS: Neuromorphic Trading Intelligence"
//!
//! The audit service mirrors the dual-service architecture:
//! - **Forward Service**: Real-time static analysis (pattern detection)
//! - **Backward Service**: LLM consolidation (learning and schema formation)
//!
//! Reference: https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================
// PART 1: NEUROMORPHIC ARCHITECTURE
// ============================================

/// Neuromorphic brain regions mapped to code analysis components
///
/// Corresponds to Part 4 of the Janus paper: Neuromorphic Architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrainRegion {
    /// Visual Cortex: Pattern recognition (GAF/ViViT chart analysis)
    /// Maps to: Static pattern detection in code
    VisualCortex,

    /// Hippocampus: Episodic memory (experience replay buffer)
    /// Maps to: Recent audit findings and issue history
    Hippocampus,

    /// Prefrontal Cortex: Logic and planning (LTN constraint checking)
    /// Maps to: Tag-based constraints and compliance rules
    PrefrontalCortex,

    /// Basal Ganglia: Action selection (buy/sell/hold decisions)
    /// Maps to: Task generation and priority assignment
    BasalGanglia,

    /// Cerebellum: Motor prediction (market impact forecasting)
    /// Maps to: Impact analysis and risk assessment
    Cerebellum,

    /// Amygdala: Threat detection (risk circuit breakers)
    /// Maps to: Critical issue detection and frozen code violations
    Amygdala,

    /// Neocortex: Strategic planning & long-term memory
    /// Maps to: Schema formation from LLM analysis patterns
    Neocortex,
}

impl BrainRegion {
    /// Get the biological function description
    pub fn biological_function(&self) -> &'static str {
        match self {
            BrainRegion::VisualCortex => "Pattern recognition in visual data",
            BrainRegion::Hippocampus => "Episodic memory and rapid encoding",
            BrainRegion::PrefrontalCortex => "Executive function and logical reasoning",
            BrainRegion::BasalGanglia => "Action selection and motor control",
            BrainRegion::Cerebellum => "Predictive modeling and fine motor control",
            BrainRegion::Amygdala => "Threat detection and fear response",
            BrainRegion::Neocortex => "Abstract reasoning and long-term knowledge",
        }
    }

    /// Get the audit system mapping
    pub fn audit_mapping(&self) -> &'static str {
        match self {
            BrainRegion::VisualCortex => "Static code analysis and pattern detection",
            BrainRegion::Hippocampus => "Recent audit history and issue tracking",
            BrainRegion::PrefrontalCortex => "Constraint validation via audit tags",
            BrainRegion::BasalGanglia => "Task generation and prioritization",
            BrainRegion::Cerebellum => "Change impact prediction",
            BrainRegion::Amygdala => "Critical issue alerts and circuit breakers",
            BrainRegion::Neocortex => "LLM-based schema and pattern learning",
        }
    }
}

// ============================================
// PART 2: FORWARD SERVICE (Real-time Analysis)
// ============================================

/// Forward Service state representation
///
/// Corresponds to Section 5.2 of Janus paper: Core Data Structures
///
/// Mathematical representation:
/// S_t = (τ_t, f_t, O_t, c_t)
/// where:
/// - τ_t: timestamp
/// - f_t: feature vector (code metrics)
/// - O_t: observation state (file content/AST)
/// - c_t: contextual metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardState {
    /// Timestamp of analysis
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Feature vector: extracted code metrics
    pub features: FeatureVector,

    /// Observation: file content and AST representation
    pub observation: Observation,

    /// Context: environmental metadata
    pub context: ContextMetadata,
}

/// Feature vector extracted from code
///
/// Corresponds to mathematical feature vector f_t ∈ ℝ^d
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    /// Lines of code
    pub loc: usize,

    /// Cyclomatic complexity
    pub complexity: f64,

    /// Documentation coverage ratio
    pub doc_coverage: f64,

    /// Test coverage ratio
    pub test_coverage: f64,

    /// Security score [0.0, 1.0]
    pub security_score: f64,

    /// Type safety score [0.0, 1.0]
    pub type_safety: f64,

    /// Async safety score [0.0, 1.0]
    pub async_safety: f64,

    /// Custom metric vector for extensibility
    pub custom_metrics: HashMap<String, f64>,
}

/// Code observation (file content and structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// File path
    pub path: PathBuf,

    /// Raw content hash
    pub content_hash: String,

    /// Abstract syntax tree representation (serialized)
    pub ast_summary: Option<String>,

    /// Import/dependency graph
    pub dependencies: Vec<String>,
}

/// Contextual metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Repository information
    pub repository: String,

    /// Branch name
    pub branch: String,

    /// Commit hash
    pub commit: Option<String>,

    /// CI/CD context
    pub ci_context: Option<CIContext>,

    /// Volatility: rate of change (commits per day)
    pub volatility: f64,

    /// Spread: variance in code quality metrics
    pub spread: f64,

    /// Volume: total activity (commits, PRs, issues)
    pub volume: usize,
}

/// CI/CD execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIContext {
    /// Run ID
    pub run_id: String,

    /// Workflow name
    pub workflow: String,

    /// Test results
    pub test_pass: bool,

    /// Build status
    pub build_status: String,
}

/// Logic Tensor Network (LTN) constraint
///
/// Corresponds to Section 2 of Janus paper: Logic Tensor Networks
///
/// Mathematical representation:
/// C_k = (P_k, w_k)
/// where P_k: S → [0,1] is a predicate and w_k ∈ ℝ+ is the weight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LTNConstraint {
    /// Constraint identifier
    pub id: String,

    /// Predicate name
    pub predicate: String,

    /// Weight (importance)
    pub weight: f64,

    /// Satisfaction score [0.0, 1.0]
    pub satisfaction: Option<f64>,

    /// Variables bound to this constraint
    pub variables: Vec<String>,
}

impl LTNConstraint {
    /// Evaluate constraint satisfaction
    ///
    /// Returns score in [0, 1] where 1 = fully satisfied
    pub fn evaluate(&self, state: &ForwardState) -> f64 {
        // Placeholder: implement specific constraint logic
        match self.predicate.as_str() {
            "wash_sale" => self.evaluate_wash_sale(state),
            "frozen_code" => self.evaluate_frozen_code(state),
            "type_safety" => state.features.type_safety,
            "security" => state.features.security_score,
            _ => 0.5, // Unknown predicate
        }
    }

    fn evaluate_wash_sale(&self, _state: &ForwardState) -> f64 {
        // Example: check if code modifies frozen sections
        1.0 // Placeholder
    }

    fn evaluate_frozen_code(&self, state: &ForwardState) -> f64 {
        // Check for @audit-freeze tags in modified files
        // Returns 1.0 if no frozen code modified, 0.0 otherwise
        if state.observation.path.to_string_lossy().contains("frozen") {
            0.0
        } else {
            1.0
        }
    }
}

/// Łukasiewicz T-norm operations for fuzzy logic
///
/// Corresponds to Section 2.3 of Janus paper
pub struct LukasiewiczLogic;

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

/// Dual-pathway decision engine
///
/// Corresponds to Section 4 of Janus paper: Basal Ganglia Pathways
///
/// Mathematical representation:
/// - Direct pathway (Go): d_direct = ReLU(W_d * h + b_d)
/// - Indirect pathway (No-Go): d_indirect = ReLU(W_i * h + b_i)
/// - Action: a_t = softmax(d_direct - λ * d_indirect)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualPathwayDecision {
    /// Direct pathway activation (encourage action)
    pub direct_pathway: f64,

    /// Indirect pathway activation (inhibit action)
    pub indirect_pathway: f64,

    /// Inhibition weight λ
    pub inhibition_weight: f64,

    /// Final action score
    pub action_score: f64,
}

impl DualPathwayDecision {
    /// Compute decision from state
    pub fn from_features(features: &FeatureVector) -> Self {
        // Direct pathway: encourage action for high-quality, safe code
        let direct = (features.security_score + features.type_safety + features.async_safety) / 3.0;

        // Indirect pathway: inhibit for low quality or high risk
        let indirect = 1.0 - ((features.doc_coverage + features.test_coverage) / 2.0);

        let inhibition_weight = 0.5;
        let action_score = (direct - inhibition_weight * indirect).max(0.0);

        Self {
            direct_pathway: direct,
            indirect_pathway: indirect,
            inhibition_weight,
            action_score,
        }
    }

    /// Should generate a task?
    pub fn should_act(&self) -> bool {
        self.action_score > 0.5
    }
}

// ============================================
// PART 3: BACKWARD SERVICE (Consolidation)
// ============================================

/// Three-timescale memory hierarchy
///
/// Corresponds to Section 1 of Janus Backward paper: Memory Hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHierarchy {
    /// Short-term memory (Hippocampus): recent audit findings
    pub hippocampus: HippocampalMemory,

    /// Medium-term consolidation (SWR): prioritized replay
    pub swr_buffer: SWRBuffer,

    /// Long-term memory (Neocortex): learned schemas
    pub neocortex: NeocorticalSchemas,
}

/// Hippocampal episodic memory buffer
///
/// Mathematical representation:
/// D_hippo = {(s_t, a_t, r_t, s_{t+1}, c_t)}_{t=1}^T
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HippocampalMemory {
    /// Episodic buffer of recent experiences
    pub episodes: Vec<Episode>,

    /// Maximum buffer size
    pub capacity: usize,
}

impl HippocampalMemory {
    /// Create new hippocampal memory
    pub fn new(capacity: usize) -> Self {
        Self {
            episodes: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Add new episode
    pub fn store(&mut self, episode: Episode) {
        if self.episodes.len() >= self.capacity {
            self.episodes.remove(0); // FIFO eviction
        }
        self.episodes.push(episode);
    }
}

/// Single experience episode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// Unique identifier for this episode
    pub id: String,

    /// State at time t
    pub state: ForwardState,

    /// Action taken
    pub action: String,

    /// Reward/outcome (1.0 = good, 0.0 = bad)
    pub reward: f64,

    /// Next state
    pub next_state: Option<ForwardState>,

    /// Context vector
    pub context: ContextMetadata,
}

/// Sharp-Wave Ripple (SWR) prioritized replay buffer
///
/// Corresponds to Section 2.2 of Janus Backward paper
///
/// Priority: p_i = |δ_i| + ε
/// where δ_i is the TD-error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SWRBuffer {
    /// Experiences with priorities
    pub experiences: Vec<(Episode, f64)>,

    /// Priority exponent α ∈ [0,1]
    pub alpha: f64,

    /// Importance sampling correction β ∈ [0,1]
    pub beta: f64,

    /// Small constant for numerical stability
    pub epsilon: f64,
}

impl SWRBuffer {
    /// Create new SWR buffer
    pub fn new() -> Self {
        Self {
            experiences: Vec::new(),
            alpha: 0.6,
            beta: 0.4,
            epsilon: 1e-6,
        }
    }

    /// Add experience with priority
    pub fn store(&mut self, episode: Episode, td_error: f64) {
        let priority = td_error.abs() + self.epsilon;
        self.experiences.push((episode, priority));
    }

    /// Sample batch with prioritization using weighted random sampling
    ///
    /// Implements Prioritized Experience Replay (PER) sampling:
    /// P(i) = p_i^α / Σ_k p_k^α
    ///
    /// Where p_i is the priority of experience i and α controls
    /// how much prioritization is used (0 = uniform, 1 = full priority)
    pub fn sample(&self, batch_size: usize) -> Vec<Episode> {
        use rand::Rng;

        if self.experiences.is_empty() {
            return Vec::new();
        }

        let actual_batch_size = batch_size.min(self.experiences.len());

        // Compute priority^alpha for each experience
        let priorities_alpha: Vec<f64> = self
            .experiences
            .iter()
            .map(|(_, p)| p.powf(self.alpha))
            .collect();

        // Compute total for normalization
        let total: f64 = priorities_alpha.iter().sum();

        if total <= 0.0 {
            // Fallback to uniform sampling if priorities are invalid
            let mut rng = rand::rng();
            let mut indices: Vec<usize> = (0..self.experiences.len()).collect();

            // Fisher-Yates shuffle for first batch_size elements
            for i in 0..actual_batch_size {
                let j = rng.random_range(i..self.experiences.len());
                indices.swap(i, j);
            }

            return indices
                .into_iter()
                .take(actual_batch_size)
                .map(|i| self.experiences[i].0.clone())
                .collect();
        }

        // Compute cumulative distribution function (CDF)
        let mut cdf = Vec::with_capacity(self.experiences.len());
        let mut cumsum = 0.0;
        for p_alpha in &priorities_alpha {
            cumsum += p_alpha / total;
            cdf.push(cumsum);
        }

        // Sample using inverse transform sampling
        let mut rng = rand::rng();
        let mut sampled = Vec::with_capacity(actual_batch_size);
        let mut sampled_indices = std::collections::HashSet::new();

        // Sample without replacement
        while sampled.len() < actual_batch_size {
            let r: f64 = rng.random();

            // Binary search to find the index
            let idx = match cdf.binary_search_by(|probe| {
                probe.partial_cmp(&r).unwrap_or(std::cmp::Ordering::Equal)
            }) {
                Ok(i) => i,
                Err(i) => i.min(self.experiences.len() - 1),
            };

            // Avoid sampling the same experience twice
            if !sampled_indices.contains(&idx) {
                sampled_indices.insert(idx);
                sampled.push(self.experiences[idx].0.clone());
            }
        }

        sampled
    }

    /// Sample batch with importance sampling weights
    ///
    /// Returns (episodes, weights) where weights are used for
    /// importance sampling correction in learning updates:
    /// w_i = (N * P(i))^(-β) / max_j w_j
    pub fn sample_with_weights(&self, batch_size: usize) -> (Vec<Episode>, Vec<f64>) {
        use rand::Rng;

        if self.experiences.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let n = self.experiences.len() as f64;
        let actual_batch_size = batch_size.min(self.experiences.len());

        // Compute priority^alpha for each experience
        let priorities_alpha: Vec<f64> = self
            .experiences
            .iter()
            .map(|(_, p)| p.powf(self.alpha))
            .collect();

        let total: f64 = priorities_alpha.iter().sum();

        if total <= 0.0 {
            // Uniform weights if priorities invalid
            let uniform_weight = 1.0;
            let episodes = self
                .experiences
                .iter()
                .take(actual_batch_size)
                .map(|(e, _)| e.clone())
                .collect();
            let weights = vec![uniform_weight; actual_batch_size];
            return (episodes, weights);
        }

        // Compute probabilities
        let probs: Vec<f64> = priorities_alpha.iter().map(|p| p / total).collect();

        // Sample indices
        let mut rng = rand::rng();
        let mut sampled_indices = Vec::with_capacity(actual_batch_size);
        let mut sampled_set = std::collections::HashSet::new();

        // Build CDF for sampling
        let mut cdf = Vec::with_capacity(self.experiences.len());
        let mut cumsum = 0.0;
        for prob in &probs {
            cumsum += prob;
            cdf.push(cumsum);
        }

        while sampled_indices.len() < actual_batch_size {
            let r: f64 = rng.random();
            let idx = match cdf.binary_search_by(|probe| {
                probe.partial_cmp(&r).unwrap_or(std::cmp::Ordering::Equal)
            }) {
                Ok(i) => i,
                Err(i) => i.min(self.experiences.len() - 1),
            };

            if !sampled_set.contains(&idx) {
                sampled_set.insert(idx);
                sampled_indices.push(idx);
            }
        }

        // Compute importance sampling weights
        // w_i = (N * P(i))^(-β)
        let mut weights: Vec<f64> = sampled_indices
            .iter()
            .map(|&idx| (n * probs[idx]).powf(-self.beta))
            .collect();

        // Normalize by max weight for stability
        let max_weight = weights.iter().cloned().fold(f64::MIN, f64::max);
        if max_weight > 0.0 {
            for w in &mut weights {
                *w /= max_weight;
            }
        }

        let episodes = sampled_indices
            .iter()
            .map(|&idx| self.experiences[idx].0.clone())
            .collect();

        (episodes, weights)
    }

    /// Update priority for an experience after learning
    pub fn update_priority(&mut self, episode_id: &str, new_td_error: f64) {
        let new_priority = new_td_error.abs() + self.epsilon;

        for (episode, priority) in &mut self.experiences {
            if episode.id == episode_id {
                *priority = new_priority;
                break;
            }
        }
    }
}

impl Default for SWRBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Neocortical schema storage
///
/// Corresponds to Section 2.3 of Janus Backward paper
///
/// Schema representation: S_k = (id_k, z_k, M_k)
/// where z_k is centroid embedding and M_k is metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeocorticalSchemas {
    /// Learned schemas (patterns)
    pub schemas: Vec<Schema>,
}

impl NeocorticalSchemas {
    /// Create new neocortical storage
    pub fn new() -> Self {
        Self {
            schemas: Vec::new(),
        }
    }

    /// Add or update schema
    pub fn upsert(&mut self, schema: Schema) {
        if let Some(existing) = self.schemas.iter_mut().find(|s| s.id == schema.id) {
            *existing = schema;
        } else {
            self.schemas.push(schema);
        }
    }

    /// Retrieve nearest schema
    pub fn find_nearest(&self, embedding: &[f64]) -> Option<&Schema> {
        self.schemas.iter().min_by(|a, b| {
            let dist_a = cosine_distance(&a.centroid, embedding);
            let dist_b = cosine_distance(&b.centroid, embedding);
            dist_a.partial_cmp(&dist_b).unwrap()
        })
    }
}

impl Default for NeocorticalSchemas {
    fn default() -> Self {
        Self::new()
    }
}

/// Learned schema (pattern prototype)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Unique schema identifier
    pub id: String,

    /// Centroid embedding vector z_k ∈ ℝ^d
    pub centroid: Vec<f64>,

    /// Number of experiences in cluster
    pub member_count: usize,

    /// Average reward
    pub avg_reward: f64,

    /// Volatility (standard deviation of rewards)
    pub volatility: f64,

    /// Human-readable description
    pub description: String,
}

/// Compute cosine distance between vectors
fn cosine_distance(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        return f64::INFINITY;
    }

    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return f64::INFINITY;
    }

    1.0 - (dot / (norm_a * norm_b))
}

// ============================================
// PART 4: INTEGRATION LAYER
// ============================================

/// Janus audit system orchestrator
///
/// Integrates Forward (real-time) and Backward (consolidation) services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JanusOrchestrator {
    /// Memory hierarchy
    pub memory: MemoryHierarchy,

    /// Active constraints
    pub constraints: Vec<LTNConstraint>,

    /// System configuration
    pub config: JanusConfig,
}

/// Janus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JanusConfig {
    /// Hippocampal buffer capacity
    pub hippocampus_capacity: usize,

    /// SWR replay batch size
    pub swr_batch_size: usize,

    /// Decision threshold for task generation
    pub decision_threshold: f64,

    /// Enable amygdala threat detection
    pub enable_amygdala: bool,

    /// LLM provider for consolidation
    pub llm_provider: String,
}

impl Default for JanusConfig {
    fn default() -> Self {
        Self {
            hippocampus_capacity: 1000,
            swr_batch_size: 32,
            decision_threshold: 0.5,
            enable_amygdala: true,
            llm_provider: "xai".to_string(),
        }
    }
}

impl JanusOrchestrator {
    /// Create new orchestrator
    pub fn new(config: JanusConfig) -> Self {
        Self {
            memory: MemoryHierarchy {
                hippocampus: HippocampalMemory::new(config.hippocampus_capacity),
                swr_buffer: SWRBuffer::new(),
                neocortex: NeocorticalSchemas::new(),
            },
            constraints: Vec::new(),
            config,
        }
    }

    /// Process forward (real-time) analysis
    pub fn forward_pass(&mut self, state: ForwardState) -> DualPathwayDecision {
        // Evaluate constraints
        let constraint_satisfaction: f64 = self
            .constraints
            .iter()
            .map(|c| c.evaluate(&state) * c.weight)
            .sum::<f64>()
            / self.constraints.iter().map(|c| c.weight).sum::<f64>();

        // Compute decision
        let mut decision = DualPathwayDecision::from_features(&state.features);

        // Modulate by constraint satisfaction
        decision.action_score *= constraint_satisfaction;

        // Store episode
        let episode = Episode {
            id: uuid::Uuid::new_v4().to_string(),
            state,
            action: if decision.should_act() {
                "generate_task".to_string()
            } else {
                "skip".to_string()
            },
            reward: constraint_satisfaction,
            next_state: None,
            context: decision.action_score.into(),
        };

        self.memory.hippocampus.store(episode);

        decision
    }

    /// Process backward (consolidation) pass
    pub fn backward_pass(&mut self) {
        // Sample prioritized experiences
        let batch = self.memory.swr_buffer.sample(self.config.swr_batch_size);

        // TODO: Cluster experiences and form schemas
        // TODO: Update neocortical memory
        // TODO: Trigger LLM analysis for pattern extraction

        println!("Consolidated {} experiences into schemas", batch.len());
    }

    /// Add constraint
    pub fn add_constraint(&mut self, constraint: LTNConstraint) {
        self.constraints.push(constraint);
    }
}

// Temporary conversion from f64 to ContextMetadata for demo purposes
impl From<f64> for ContextMetadata {
    fn from(score: f64) -> Self {
        Self {
            repository: "unknown".to_string(),
            branch: "main".to_string(),
            commit: None,
            ci_context: None,
            volatility: score,
            spread: 0.0,
            volume: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lukasiewicz_logic() {
        assert_eq!(LukasiewiczLogic::and(0.8, 0.7), 0.5);
        assert_eq!(LukasiewiczLogic::or(0.3, 0.4), 0.7);
        assert_eq!(LukasiewiczLogic::not(0.6), 0.4);
        assert_eq!(LukasiewiczLogic::implies(0.8, 0.5), 0.7);
    }

    #[test]
    fn test_dual_pathway() {
        let features = FeatureVector {
            loc: 100,
            complexity: 5.0,
            doc_coverage: 0.8,
            test_coverage: 0.9,
            security_score: 0.95,
            type_safety: 0.9,
            async_safety: 0.85,
            custom_metrics: HashMap::new(),
        };

        let decision = DualPathwayDecision::from_features(&features);
        assert!(decision.should_act());
        assert!(decision.action_score > 0.5);
    }

    #[test]
    fn test_hippocampal_memory() {
        let mut memory = HippocampalMemory::new(2);
        let episode = Episode {
            id: "test-episode-1".to_string(),
            state: create_test_state(),
            action: "test".to_string(),
            reward: 0.8,
            next_state: None,
            context: create_test_context(),
        };

        memory.store(episode.clone());
        assert_eq!(memory.episodes.len(), 1);

        memory.store(episode.clone());
        assert_eq!(memory.episodes.len(), 2);

        memory.store(episode);
        assert_eq!(memory.episodes.len(), 2); // Should evict oldest
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let c = vec![1.0, 0.0, 0.0];

        assert!((cosine_distance(&a, &b) - 1.0).abs() < 1e-6); // Orthogonal
        assert!(cosine_distance(&a, &c).abs() < 1e-6); // Identical
    }

    fn create_test_state() -> ForwardState {
        ForwardState {
            timestamp: chrono::Utc::now(),
            features: FeatureVector {
                loc: 100,
                complexity: 5.0,
                doc_coverage: 0.8,
                test_coverage: 0.9,
                security_score: 0.95,
                type_safety: 0.9,
                async_safety: 0.85,
                custom_metrics: HashMap::new(),
            },
            observation: Observation {
                path: PathBuf::from("test.rs"),
                content_hash: "abc123".to_string(),
                ast_summary: None,
                dependencies: vec![],
            },
            context: create_test_context(),
        }
    }

    fn create_test_context() -> ContextMetadata {
        ContextMetadata {
            repository: "test-repo".to_string(),
            branch: "main".to_string(),
            commit: Some("abc123".to_string()),
            ci_context: None,
            volatility: 0.5,
            spread: 0.1,
            volume: 100,
        }
    }
}
