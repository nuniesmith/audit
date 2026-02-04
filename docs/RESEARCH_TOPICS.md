# Research Topics for RustAssistant

**Priority:** Items requiring investigation before implementation  
**Status:** 🔬 Research Phase  
**Owner:** Development Team

---

## 🔴 High Priority (This Week)

### 1. Cache Invalidation Strategy

**Current State:**
- Only invalidates on file content hash change
- No versioning of prompts, models, or analysis logic

**Research Questions:**
- How to version prompt templates effectively?
- Should we store prompt hash in cache entries?
- How to handle backwards compatibility when prompts change?
- What happens when model version changes (grok-beta → grok-2)?
- How to detect when dependencies affect analysis results?

**Proposed Investigation:**
```rust
pub struct CacheVersion {
    schema_version: String,        // "1.0.0"
    prompt_template_hash: String,  // SHA-256 of prompt
    model_version: String,         // "grok-beta-20250115"
    rustassistant_version: String, // "0.2.0"
}
```

**Research Tasks:**
- [ ] Survey how other tools handle cache versioning (Cargo, npm, etc.)
- [ ] Design version comparison algorithm
- [ ] Determine migration strategy for schema changes
- [ ] Test cache invalidation performance impact
- [ ] Document versioning best practices

**Expected Outcome:** Cache versioning specification document

---

### 2. Cache Size Management & Pruning

**Current State:**
- Unbounded cache growth
- No cleanup mechanism
- No size limits or quotas

**Research Questions:**
- What's an acceptable cache size for typical usage? (100MB? 1GB? 10GB?)
- Which pruning strategy is most effective?
  - LRU (Least Recently Used)
  - LFU (Least Frequently Used)
  - Cost-based (keep expensive analyses)
  - Size-based (remove large entries first)
  - Hybrid approach?
- When should auto-pruning trigger?
- How to prevent thrashing (prune/rebuild cycle)?

**Proposed Investigation:**
```toml
[cache.limits]
max_size = "1GB"
max_age_days = 90
max_entries_per_repo = 1000
prune_strategy = "lru"  # lru, lfu, cost, size, hybrid
auto_prune = true
warn_threshold = "800MB"
```

**Research Tasks:**
- [ ] Profile cache size growth with real projects
- [ ] Benchmark different pruning strategies
- [ ] Measure prune operation performance
- [ ] Design prune metrics (what to keep vs remove)
- [ ] Test impact on cache hit rates after pruning

**Expected Outcome:** Cache management implementation plan

---

### 3. Multi-Repo Batch Processing

**Current State:**
- Processes one file at a time
- No coordination across repos
- No priority system

**Research Questions:**
- How to efficiently batch-process 100+ repositories?
- What's optimal parallelism? (CPU-bound vs API-rate-limited)
- How to implement priority queuing for repos/files?
- Should we use worker pool pattern?
- How to handle failures and retries?
- What progress tracking is most useful?

**Proposed Investigation:**
```rust
pub struct CacheManager {
    repos: Vec<PathBuf>,
    max_parallel: usize,
    rate_limit: RateLimit,
    priority_queue: PriorityQueue<CacheJob>,
}

pub struct BatchOptions {
    max_parallel_files: usize,
    max_parallel_repos: usize,
    rate_limit: Duration,
    retry_policy: RetryPolicy,
}
```

**Research Tasks:**
- [ ] Profile API rate limits for Grok
- [ ] Test optimal parallelism on Raspberry Pi
- [ ] Design job scheduling algorithm
- [ ] Implement progress tracking UI
- [ ] Measure throughput and cost

**Expected Outcome:** Batch processing architecture

---

## 🟡 Medium Priority (Next Week)

### 4. Token Usage Tracking & Cost Attribution

**Current State:**
- Token field exists but not populated
- No cost tracking per repo/file/type
- No budget management

**Research Questions:**
- How to accurately extract token counts from LLM responses?
- Should we estimate tokens for cached responses?
- How to attribute costs across repos, files, and analysis types?
- What cost reporting is most valuable?
- How to implement budget warnings/limits?

**Proposed Investigation:**
```rust
pub struct TokenMetrics {
    input_tokens: usize,
    output_tokens: usize,
    total_tokens: usize,
    estimated_cost_usd: f64,
}

pub struct CostReport {
    total_cost: f64,
    by_repo: HashMap<String, f64>,
    by_type: HashMap<CacheType, f64>,
    by_date: HashMap<String, f64>,
    cache_savings: f64,
}
```

**Research Tasks:**
- [ ] Document token extraction from Grok API responses
- [ ] Design cost calculation formulas (pricing tiers)
- [ ] Implement budget tracking database schema
- [ ] Create cost visualization UI
- [ ] Test accuracy of cost estimation

**Expected Outcome:** Token tracking implementation

---

### 5. Cache Distribution & Sharing

**Current State:**
- Local cache only
- No team sharing mechanism
- No remote cache support

**Research Questions:**
- Should teams be able to share cache?
- Network cache service vs. file-based sharing?
- Git LFS for large cache files?
- S3/object storage backend?
- Cache CDN architecture?
- Security considerations (cache poisoning)?

**Use Cases:**
- Team working on same codebase
- CI/CD cache sharing
- Cross-machine cache sync
- Public cache for open-source projects?

**Proposed Investigation:**
```rust
pub enum CacheBackend {
    Local(PathBuf),
    Network { url: String, auth: Auth },
    S3 { bucket: String, region: String },
    Git { repo: String, lfs: bool },
}
```

**Research Tasks:**
- [ ] Survey existing cache distribution solutions (Cargo, Bazel, etc.)
- [ ] Design cache sharing protocol
- [ ] Prototype network cache server
- [ ] Test cache sync performance
- [ ] Document security model

**Expected Outcome:** Cache sharing specification (Phase 3+)

---

### 6. Incremental Analysis & Dependency Tracking

**Current State:**
- Analyzes each file independently
- No dependency awareness
- Re-analyzes unchanged files with changed dependencies

**Research Questions:**
- How to track file dependencies in Rust projects?
- When should we invalidate cache based on dependency changes?
- Can we do incremental analysis (only changed parts)?
- How to represent dependency graph efficiently?
- What's the overhead of dependency tracking?

**Proposed Investigation:**
```rust
pub struct FileDependencies {
    file: PathBuf,
    imports: Vec<PathBuf>,
    used_by: Vec<PathBuf>,
    dependency_hash: String,
}

// Invalidate cache if:
// - File content changed
// - Any dependency changed
// - Any transitive dependency changed (configurable depth)
```

**Research Tasks:**
- [ ] Investigate Rust AST parsing for dependency extraction
- [ ] Design dependency graph data structure
- [ ] Test incremental analysis performance
- [ ] Measure cache hit rate improvement
- [ ] Compare with full re-analysis

**Expected Outcome:** Dependency-aware caching design

---

## 🟢 Low Priority (Future)

### 7. Cross-Language Support

**Research Questions:**
- How to extend caching to Python, JavaScript, etc.?
- Different analysis types per language?
- Universal cache format?

**Status:** Deferred to Phase 4+

---

### 8. Machine Learning for Cache Optimization

**Research Questions:**
- Can we predict which files are likely to change?
- ML model to prioritize cache building?
- Anomaly detection for cache invalidation?

**Status:** Experimental, Phase 5+

---

### 9. Distributed Cache Architecture

**Research Questions:**
- Multi-node cache cluster?
- Cache replication and consistency?
- Load balancing for cache requests?

**Status:** Only if scaling to 1000s of repos

---

## 📊 Research Methodology

### For Each Topic:

1. **Literature Review** (1-2 hours)
   - Survey existing solutions
   - Read relevant papers/blogs
   - Document prior art

2. **Prototype** (2-4 hours)
   - Build minimal POC
   - Test core assumptions
   - Measure performance

3. **Benchmarking** (1-2 hours)
   - Test with real data
   - Compare alternatives
   - Identify bottlenecks

4. **Documentation** (1 hour)
   - Write findings
   - Propose solution
   - Create implementation plan

5. **Review** (30 min)
   - Discuss trade-offs
   - Make decision
   - Update roadmap

**Total per topic:** ~6-10 hours

---

## 🎯 Research Priorities

### This Week
1. Cache Invalidation Strategy
2. Cache Size Management

### Next Week
3. Multi-Repo Batch Processing
4. Token Usage Tracking

### Later
5. Cache Distribution
6. Incremental Analysis
7. Cross-Language Support
8. ML Optimization
9. Distributed Architecture

---

## 📝 Research Outputs

Each research topic should produce:

1. **Findings Document**
   - Problem statement
   - Options evaluated
   - Benchmarks and data
   - Recommendation

2. **Implementation Plan**
   - Architecture design
   - API specification
   - Migration strategy
   - Testing approach

3. **Prototype Code**
   - POC in `research/` directory
   - Benchmarks
   - Examples

4. **Decision Record**
   - What was decided
   - Why
   - Alternatives considered
   - Trade-offs accepted

---

## 🔬 Active Research

### Currently Investigating:
- [ ] Cache Invalidation Strategy (Week 1)

### Up Next:
- [ ] Cache Size Management (Week 1)
- [ ] Multi-Repo Batch Processing (Week 2)

### Blocked/Waiting:
- None currently

---

## 💡 Research Ideas Welcome

If you discover new areas needing research:

1. Add to this document
2. Prioritize (High/Medium/Low)
3. Estimate effort
4. Tag with phase (2/3/4/etc.)

**Research is an investment in quality.**

---

**Last Updated:** 2025-01-XX  
**Next Review:** Weekly during development

---

## 📚 Useful Resources

### Cache Systems
- Redis cache strategies
- Cargo build cache
- Bazel remote cache
- npm cache design
- Turborepo remote caching

### Rust Tooling
- rust-analyzer incremental analysis
- clippy cache implementation
- cargo-watch file tracking

### Distributed Systems
- Raft consensus (if needed for distributed cache)
- CAP theorem considerations
- Eventual consistency models

---

**Remember:** Research should inform decisions, not delay shipping.

If research takes >1 week, ship MVP and iterate.