# Rustassistant Implementation Roadmap

**Based on RAG Article Analysis**  
**Created:** February 2, 2025  
**Status:** Ready for Development

---

## 🎯 Executive Summary

This roadmap applies learnings from production RAG systems to rustassistant, optimized for:
- **Solo developer workflow** (not enterprise scale)
- **Grok's 2M token context window** (leverage "context stuffing" before complex RAG)
- **Cost optimization** (semantic caching, smart routing)
- **Personal scale** (hundreds of notes, not millions of documents)

**Key Insight:** With Grok's massive context window, we can likely fit your entire workflow (notes + active repo files) into a single prompt. Start simple, add complexity only when needed.

---

## 📋 Current State Assessment

### ✅ What We Have
- Core database (notes, repos, tasks) ✓
- CLI and REST API ✓
- Docker deployment ✓
- CI/CD pipeline ✓
- **Response caching** (`response_cache.rs`) - SHA-256 based, TTL support ✓
- **Context builder** (`context_builder.rs`) - Already built! ✓
- **Grok client** (`grok_client.rs`) - API integration ready ✓
- TODO scanner (`todo_scanner.rs`) ✓
- Repository analysis (`repo_analysis.rs`) ✓

### 🚧 What We Need
- **Query Router** - Classify intent before expensive API calls
- **Semantic Cache** - Similar query detection (upgrade from exact-match cache)
- **Cost Tracker** - Track spend per query type
- **Content Deduplication** - Prevent duplicate notes
- **Smart Context Stuffing** - Leverage 2M token window efficiently

---

## 🗺️ Implementation Phases

### Phase 1: Query Intelligence (Week 5-6)
**Goal:** Make every Grok API call count

#### 1.1 Query Intent Classifier
**File:** `src/query_router.rs`

```rust
pub enum QueryIntent {
    Greeting,          // "hi", "thanks" → no API needed
    NoteSearch,        // "find my notes about X" → search DB only
    RepoAnalysis,      // "analyze auth.rs" → needs Grok
    TaskGeneration,    // "what should I work on?" → needs Grok + context
    CodeQuestion,      // "how does this work?" → needs Grok
    DirectAnswer,      // FAQ-style questions
}

pub struct QueryRouter {
    cache: ResponseCache,
    db: SqlitePool,
}

impl QueryRouter {
    pub async fn route(&self, query: &str, context: &UserContext) -> Action {
        // 1. Check semantic cache first (free!)
        if let Some(cached) = self.semantic_cache_lookup(query).await? {
            return Action::CachedResponse(cached);
        }
        
        // 2. Classify intent (cheap heuristics)
        let intent = self.classify_intent(query);
        
        match intent {
            QueryIntent::Greeting => Action::DirectResponse(greet()),
            QueryIntent::NoteSearch => Action::SearchDatabase(query),
            QueryIntent::RepoAnalysis => Action::CallGrok(build_analysis_context(query)),
            QueryIntent::TaskGeneration => Action::CallGrok(build_task_context(query)),
            _ => Action::CallGrok(build_generic_context(query)),
        }
    }
    
    fn classify_intent(&self, query: &str) -> QueryIntent {
        let lower = query.to_lowercase();
        
        // Simple pattern matching (good enough for solo dev)
        if matches!(lower.as_str(), "hi" | "hello" | "thanks" | "thank you") {
            return QueryIntent::Greeting;
        }
        
        if lower.contains("find") || lower.contains("search") || lower.contains("show me") {
            return QueryIntent::NoteSearch;
        }
        
        if lower.contains("analyze") || lower.contains("review") || lower.contains("score") {
            return QueryIntent::RepoAnalysis;
        }
        
        if lower.contains("what should") || lower.contains("next task") || lower.contains("recommend") {
            return QueryIntent::TaskGeneration;
        }
        
        QueryIntent::CodeQuestion
    }
}
```

**Deliverables:**
- [ ] `src/query_router.rs` - Intent classification
- [ ] Tests for all intent types
- [ ] Integration with CLI (`rustassistant ask "query"`)
- [ ] Metrics: % of queries that bypass Grok

#### 1.2 Cost Tracker
**File:** `src/cost_tracker.rs`

```rust
pub struct CostTracker {
    pool: SqlitePool,
}

// Database schema
CREATE TABLE IF NOT EXISTS llm_costs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL,
    operation TEXT NOT NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    cached_tokens INTEGER DEFAULT 0,
    cost_usd REAL NOT NULL,
    query_hash TEXT,
    cache_hit BOOLEAN DEFAULT FALSE
);

impl CostTracker {
    pub async fn log_call(&self, op: &str, usage: TokenUsage) -> Result<()> {
        let cost = calculate_grok_cost(&usage);
        // Insert into database
    }
    
    pub async fn get_stats(&self, since: DateTime<Utc>) -> Result<CostStats> {
        // Total spend, cost per operation type, cache hit rate
    }
    
    pub async fn daily_report(&self) -> Result<String> {
        // "Today: $0.32 (12 queries, 8 cached, 67% hit rate)"
    }
}

fn calculate_grok_cost(usage: &TokenUsage) -> f64 {
    let input_cost = (usage.input_tokens as f64 / 1_000_000.0) * 0.20;
    let output_cost = (usage.output_tokens as f64 / 1_000_000.0) * 0.50;
    let cache_cost = (usage.cached_tokens as f64 / 1_000_000.0) * 0.05;
    input_cost + output_cost + cache_cost
}
```

**Deliverables:**
- [ ] `src/cost_tracker.rs`
- [ ] Database migration for `llm_costs` table
- [ ] CLI command: `rustassistant costs --today`
- [ ] API endpoint: `/api/costs/stats`
- [ ] Alert when daily spend > threshold

#### 1.3 Content Hash Deduplication
**File:** Update `src/db.rs`

```rust
// Add to notes table schema
CREATE TABLE IF NOT EXISTS notes (
    -- ... existing fields ...
    content_hash TEXT UNIQUE,  -- SHA-256 of normalized content
    similarity_hash TEXT       -- MinHash for fuzzy matching
);

pub async fn create_note(
    pool: &SqlitePool,
    content: &str,
    tags: Option<&str>,
    project: Option<&str>,
) -> DbResult<Note> {
    // 1. Normalize content (trim, lowercase, remove extra spaces)
    let normalized = normalize_content(content);
    
    // 2. Generate hash
    let content_hash = sha256_hash(&normalized);
    
    // 3. Check for exact duplicate
    if let Some(existing) = get_note_by_hash(pool, &content_hash).await? {
        return Err(DbError::Duplicate {
            existing_id: existing.id,
            message: "Identical note already exists".into(),
        });
    }
    
    // 4. Optional: Check for similar notes (>90% similarity)
    let similar = find_similar_notes(pool, &normalized, 0.9).await?;
    if !similar.is_empty() {
        // Log warning but allow creation
        tracing::warn!("Creating note similar to existing: {:?}", similar);
    }
    
    // 5. Create note with hash
    // ... existing creation logic ...
}
```

**Deliverables:**
- [ ] Database migration to add hash columns
- [ ] Duplicate detection on note creation
- [ ] CLI warning on similar notes
- [ ] API returns existing note on duplicate

---

### Phase 2: Smart Context Stuffing (Week 7-8)
**Goal:** Maximize Grok's 2M token window before building RAG

#### 2.1 Intelligent Context Builder
**File:** Enhance `src/context_builder.rs`

```rust
pub struct SmartContextBuilder {
    db: SqlitePool,
    max_tokens: usize,  // Default: 1_900_000 (leave room for response)
}

impl SmartContextBuilder {
    pub async fn build_for_query(&self, query: &str) -> Result<Context> {
        let mut context = Context::new();
        let mut token_budget = self.max_tokens;
        
        // Priority 1: Recent notes (last 50)
        let notes = self.get_recent_notes(50).await?;
        let notes_context = format_notes(&notes);
        context.add_section("Recent Notes", &notes_context);
        token_budget -= estimate_tokens(&notes_context);
        
        // Priority 2: Relevant repository (if mentioned in query)
        if let Some(repo_name) = extract_repo_reference(query) {
            let repo = self.get_repo_by_name(&repo_name).await?;
            let tree = self.get_cached_tree(&repo.id).await?;
            let tree_context = format_tree(&tree, token_budget / 2);
            context.add_section(&format!("Repository: {}", repo_name), &tree_context);
            token_budget -= estimate_tokens(&tree_context);
        }
        
        // Priority 3: All projects summary
        let projects = self.get_project_summaries().await?;
        context.add_section("Projects", &projects);
        token_budget -= estimate_tokens(&projects);
        
        // Priority 4: Active tasks
        let tasks = self.get_active_tasks(20).await?;
        context.add_section("Active Tasks", &format_tasks(&tasks));
        
        context.set_metadata(ContextMetadata {
            total_tokens: self.max_tokens - token_budget,
            truncated: false,
            sources: vec!["notes", "repo", "tasks"],
        });
        
        Ok(context)
    }
}
```

**Key Strategy:** For a solo dev with ~500 notes and 5 repos:
- Notes: ~500 × 100 words = 50k words = ~67k tokens
- Repo trees: ~5 × 1000 files × 50 chars = ~300k tokens
- Tasks: ~100 × 50 words = 5k words = ~7k tokens
- **Total: ~374k tokens** (fits easily in 2M window!)

**Deliverables:**
- [ ] Enhanced context builder with priority loading
- [ ] Token budget management
- [ ] Repository mention detection
- [ ] Context summary visualization
- [ ] CLI: `rustassistant context preview "query"`

#### 2.2 Query Templates
**File:** Enhance `src/query_templates.rs`

```rust
pub struct QueryTemplate {
    pub name: String,
    pub category: TemplateCategory,
    pub system_prompt: String,
    pub user_template: String,
}

pub enum TemplateCategory {
    CodeAnalysis,
    TaskGeneration,
    NoteSearch,
    ProjectPlanning,
}

impl QueryTemplate {
    pub fn code_review() -> Self {
        Self {
            name: "code_review".into(),
            category: TemplateCategory::CodeAnalysis,
            system_prompt: "You are a senior code reviewer...".into(),
            user_template: r#"
Review this file and identify:
1. Code quality issues (complexity, maintainability)
2. Security concerns
3. Performance bottlenecks
4. Best practice violations

File: {file_path}
Language: {language}

```{language}
{file_content}
```

Provide actionable recommendations with specific line references.
            "#.into(),
        }
    }
    
    pub fn next_task() -> Self {
        Self {
            name: "next_task".into(),
            category: TemplateCategory::TaskGeneration,
            system_prompt: "You are a productivity coach for developers...".into(),
            user_template: r#"
Based on this developer's current state, recommend the highest-impact task:

ACTIVE TASKS:
{tasks}

RECENT NOTES:
{notes}

REPOSITORIES:
{repos}

Consider:
- Task priority and dependencies
- Recent work patterns
- Project deadlines
- Developer energy/focus

Recommend ONE specific task with rationale.
            "#.into(),
        }
    }
}
```

**Deliverables:**
- [ ] Template registry with 10+ common queries
- [ ] Template variable substitution
- [ ] CLI: `rustassistant ask --template code_review file.rs`
- [ ] Custom template support

---

### Phase 3: Semantic Caching (Week 9)
**Goal:** Cache similar queries, not just exact matches

#### 3.1 Upgrade Response Cache
**File:** `src/semantic_cache.rs`

```rust
use fastembed::{EmbeddingModel, TextEmbedding};

pub struct SemanticCache {
    exact_cache: ResponseCache,  // Existing SHA-256 cache
    embedding_model: TextEmbedding,
    vector_store: VectorStore,  // Simple in-memory for now
    similarity_threshold: f32,  // 0.85 = 85% similar
}

impl SemanticCache {
    pub async fn get(&self, query: &str) -> Result<Option<CachedResponse>> {
        // 1. Try exact match first (fastest)
        if let Some(exact) = self.exact_cache.get(query, "semantic").await? {
            return Ok(Some(exact));
        }
        
        // 2. Generate embedding for query
        let query_embedding = self.embed_query(query).await?;
        
        // 3. Search for similar cached queries
        let similar = self.vector_store.search(&query_embedding, 1).await?;
        
        if let Some(hit) = similar.first() {
            if hit.score >= self.similarity_threshold {
                tracing::info!("Semantic cache HIT: {:.2}% similar", hit.score * 100.0);
                return Ok(Some(hit.response.clone()));
            }
        }
        
        tracing::debug!("Semantic cache MISS");
        Ok(None)
    }
    
    pub async fn set(&self, query: &str, response: &str) -> Result<()> {
        // 1. Store in exact cache
        self.exact_cache.set(query, "semantic", response, None).await?;
        
        // 2. Generate and store embedding
        let embedding = self.embed_query(query).await?;
        self.vector_store.insert(embedding, response).await?;
        
        Ok(())
    }
}

// Simple in-memory vector store (good enough for personal scale)
pub struct VectorStore {
    entries: Vec<VectorEntry>,
}

struct VectorEntry {
    embedding: Vec<f32>,
    query: String,
    response: String,
    created_at: DateTime<Utc>,
}

impl VectorStore {
    pub async fn search(&self, query_embedding: &[f32], top_k: usize) -> Result<Vec<SearchHit>> {
        let mut results: Vec<_> = self.entries.iter()
            .map(|entry| {
                let score = cosine_similarity(query_embedding, &entry.embedding);
                SearchHit {
                    score,
                    query: entry.query.clone(),
                    response: entry.response.clone(),
                }
            })
            .collect();
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(results.into_iter().take(top_k).collect())
    }
}
```

**Why Not Full RAG Yet?**
- Personal scale: ~500 notes, 5 repos
- Grok's 2M context handles this easily
- Semantic cache gives 80% of RAG benefits
- Can add vector DB later if needed

**Deliverables:**
- [ ] `src/semantic_cache.rs`
- [ ] Embedding model integration (fastembed)
- [ ] In-memory vector store
- [ ] Cache hit/miss metrics
- [ ] CLI: `rustassistant cache stats --semantic`

---

### Phase 4: Advanced Features (Week 10+)
**Optional - Only if context stuffing isn't enough**

#### 4.1 Chunking Pipeline
**If/when:** Notes or repo exceed 2M tokens

```rust
pub struct ChunkingStrategy {
    chunk_size: usize,        // 512 tokens (from article)
    chunk_overlap: usize,     // 50 tokens (from article)
    separators: Vec<String>,  // ["\n\n", "\n", ". ", " "]
}

impl ChunkingStrategy {
    pub fn chunk_document(&self, doc: &str) -> Vec<Chunk> {
        // Recursive character splitting
        // Preserve semantic boundaries
    }
}
```

#### 4.2 Hybrid Search
**If/when:** Need better retrieval than full-context stuffing

```rust
pub struct HybridSearch {
    vector_search: VectorStore,
    metadata_search: SqlitePool,
}

impl HybridSearch {
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Parallel search
        let (vector_results, tag_results) = tokio::join!(
            self.vector_search.search_embeddings(query),
            self.metadata_search.search_by_tags_and_projects(query)
        );
        
        // Merge and deduplicate
        merge_and_rank(vector_results, tag_results)
    }
}
```

#### 4.3 Full Vector Database
**If/when:** In-memory vector store becomes bottleneck

Options:
- **LanceDB** (Rust-native, lightweight)
- **Qdrant** (Docker deployment)
- **Skip it** (likely unnecessary for solo dev)

---

## 🎯 Success Metrics

### Cost Optimization
- **Target:** <$5/month Grok spend
- **Measure:** Semantic cache hit rate >60%
- **Measure:** Avg tokens per query <100k

### Performance
- **Target:** Query response <2s (cached)
- **Target:** Query response <10s (uncached)
- **Measure:** P95 latency

### Quality
- **Target:** User marks >80% responses as helpful
- **Measure:** Task completion rate from recommendations

---

## 📦 Dependencies to Add

```toml
[dependencies]
# Semantic embeddings (if doing semantic cache)
fastembed = "3"  # Lightweight embedding models

# Vector operations
ndarray = "0.15"  # For cosine similarity

# Optional: Full RAG (only if needed later)
# lancedb = "0.4"  # Rust-native vector DB
# tantivy = "0.21"  # Full-text search
```

---

## 🚫 What NOT to Build

Based on article analysis, skip these for solo dev:

- ❌ Kubernetes orchestration
- ❌ Ray clusters
- ❌ Neo4j graph database (SQLite foreign keys sufficient)
- ❌ Multi-model orchestration
- ❌ Complex observability (simple metrics sufficient)
- ❌ Terraform infrastructure
- ❌ Self-hosted LLM (use Grok API)

---

## 🔄 Decision Points

### Week 8: Do We Need Full RAG?

**Evaluate:**
```
Total context size = (notes + repos + tasks)
If total_tokens < 1.5M:
    ✅ Continue with context stuffing
    ✅ Add semantic caching
    ❌ Skip vector DB
Else:
    ⚠️ Evaluate chunking + retrieval
```

### Week 10: Vector Database?

**Evaluate:**
```
If query_latency > 10s OR memory_usage > 500MB:
    Consider LanceDB
Else:
    Stick with in-memory vectors
```

---

## 📚 Learning from the Article

### Patterns We're Adopting
1. ✅ **512-token chunking** (if needed)
2. ✅ **Semantic caching** (huge cost saver)
3. ✅ **Query classification** (route before expensive calls)
4. ✅ **Hybrid retrieval concept** (vectors + metadata)
5. ✅ **Content-based deduplication**

### Patterns We're Skipping
1. ❌ Infrastructure complexity (K8s, Ray)
2. ❌ Multi-database architecture (Neo4j + Qdrant)
3. ❌ Self-hosted models
4. ❌ Complex autoscaling

### Our Secret Weapon
🎯 **Grok's 2M token context window** = We can "cheat" by stuffing everything into one prompt. This is a massive advantage over the article's approach!

---

## 🎬 Getting Started

### This Week (Week 5)
1. [ ] Implement `QueryRouter` with intent classification
2. [ ] Add `CostTracker` to monitor spend
3. [ ] Add content deduplication to notes
4. [ ] Test with 100 real notes

### Next Week (Week 6)
1. [ ] Enhance `ContextBuilder` with smart priority
2. [ ] Create query templates for common tasks
3. [ ] Add CLI command: `rustassistant ask "query"`
4. [ ] Measure context size for your actual data

### Week 7-8
1. [ ] Evaluate: Is context stuffing sufficient?
2. [ ] If yes: Add semantic caching and celebrate 🎉
3. [ ] If no: Start chunking pipeline

---

## 📊 Cost Projection

**Assumptions:**
- 20 queries/day
- 100k tokens average per query
- 60% cache hit rate (after week 7)

**Monthly Cost:**
```
Uncached queries: 20 × 30 × 0.4 = 240 queries/month
Tokens: 240 × 100k = 24M tokens
Input cost: 24 × $0.20 = $4.80
Output cost: ~$2.00 (assuming 50k output avg)
Total: ~$7/month

With better caching (80% hit rate): ~$3/month
```

**Verdict:** Very affordable for personal use! 🎯

---

## 🎉 Summary

**Start simple:**
1. Query routing (free optimizations)
2. Cost tracking (know your spend)
3. Context stuffing (leverage 2M window)

**Add complexity only when needed:**
4. Semantic caching (similar query detection)
5. Chunking (if context exceeds 2M)
6. Vector DB (if retrieval becomes slow)

**Your advantage:** Solo dev scale + massive context window = RAG might not be needed at all! 🚀

---

*Last updated: February 2, 2025*
*Next review: End of Week 6 (evaluate context stuffing effectiveness)*