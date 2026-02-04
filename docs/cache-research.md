# Cache Architecture for LLM-Powered Rust Code Analysis

A centralized, content-addressable cache with SQLite metadata and LanceDB vector storage provides the optimal architecture for rustassistant. The system should use **multi-factor cache keys** combining file hashes, model versions, and prompt hashes to enable precise invalidation while maximizing cache reuse across repositories. For a solo developer managing 10-50 repositories on a Raspberry Pi with a $3/month budget, this design achieves sub-second lookups while keeping storage under 500MB.

The key insight from studying Cargo, Bazel, and rust-analyzer is that **content-addressable storage with lazy invalidation** dramatically outperforms timestamp-based approaches. Bazel's remote cache achieves 95%+ hit rates using SHA-256 content hashing, while rust-analyzer's Salsa framework demonstrates that query-level memoization with early cutoff optimization reduces redundant computation by 80% or more in typical editing scenarios.

---

## Centralized cache with content-addressed storage

The cache should live in `~/.rustassistant/cache/repos/` with each analyzed repository mapped to a subdirectory based on a stable hash of its absolute path. This mirrors Bazel's `outputBase` approach, which computes `MD5(workspace_path)` to create isolated cache directories while enabling cross-machine portability.

**Recommended directory structure:**

```
~/.rustassistant/
├── cache/
│   ├── repos/
│   │   ├── a7f3b2c1/              # Hash of /home/user/projects/myapp
│   │   │   ├── analysis.db        # SQLite: file analysis, scores, issues
│   │   │   ├── embeddings.lance/  # LanceDB: vector embeddings
│   │   │   └── meta.json          # Repo path, last sync, schema version
│   │   └── e9d4f8a2/              # Another repository
│   ├── global.db                   # Cross-repo patterns, shared logic
│   └── embedding_models/           # Cached local embedding models
├── config.toml
└── logs/
```

Cargo's approach of storing registry crates by `<name>-<version>` in a shared `$CARGO_HOME` demonstrates that centralized caching provides significant disk savings—a typical developer's `~/.cargo` contains deduplicated dependencies shared across all projects. For rustassistant, centralizing cache means a security finding identified in one repository can be cross-referenced against similar patterns in others without re-analysis.

The **path-to-cache mapping** should use the first 8 characters of `SHA256(canonical_repo_path)` as the subdirectory name, with the full hash and original path stored in `meta.json`. This provides collision resistance while maintaining human-debuggable directory names. Bazel uses MD5 for this purpose, but SHA-256 is preferable for consistency with file content hashing throughout the system.

---

## Multi-factor cache keys prevent stale analysis

Cache invalidation is the critical challenge for LLM-based analysis tools. Unlike traditional build caches where outputs are deterministic, LLM outputs depend on model versions, prompt templates, and configuration—any of which can change independently of source files.

The cache key formula should combine all factors that affect output:

```
cache_key = SHA256(
    file_content_hash +
    tool_version +
    model_id +           // e.g., "grok-4.1"
    prompt_hash[:16] +   // First 16 chars of SHA256(prompt_template)
    schema_version +     // Output format version
    config_hash[:16]     // Analysis options
)
```

This approach, adapted from Bazel's action digest computation, ensures that any change to inputs, tools, or configuration triggers re-analysis while unchanged combinations return cached results instantly. Turborepo's remote cache demonstrates **96% hit rates** in production using similar multi-factor hashing.

**Content hashing outperforms timestamps** for several reasons. Git doesn't preserve modification times—checking out the same commit twice produces different mtimes. Docker builds notoriously suffer from mtime instability. Content hashing sidesteps these issues entirely: identical content produces identical hashes regardless of when files were touched. The hybrid approach used by mypy and Bazel—checking mtime+size first, falling back to content hash only on mismatch—provides the best of both worlds: **O(1) stat() calls for unchanged files** with correctness guarantees when content changes.

For prompt and model versioning, include the model identifier directly in the cache namespace: `v3/grok-4.1/a1b2c3d4/{file_hash}`. When Grok releases version 4.2, cached results from 4.1 remain accessible but won't be returned for new queries, triggering fresh analysis with the updated model.

**Schema evolution** requires a migration strategy. The lazy migration pattern works well: on cache read, check if `entry.schema_version < CURRENT_SCHEMA_VERSION`. If the difference is minor (additive fields), migrate in-place. If breaking (removed/renamed fields), delete and regenerate. Store a `min_compatible_version` in the cache metadata to enable automatic cleanup of incompatible entries during pruning.

---

## Cost-aware pruning keeps expensive analysis longer

Standard LRU eviction doesn't account for regeneration cost—evicting a 30-second LLM analysis result to make room for a 10ms file hash wastes API budget and user time. The cache should use **cost-weighted eviction** that considers both access recency and regeneration expense.

The eviction score formula balances multiple factors:

```
eviction_priority = (1 / access_frequency) × (1 / regeneration_cost) × age_factor
```

Where `regeneration_cost` combines API token cost (tracked per entry), generation time, and computational complexity. Items with low priority scores get evicted first. The **moka** crate implements Window TinyLFU, which combines LFU admission filtering with LRU eviction—this prevents "one-hit wonders" from polluting the cache while protecting frequently-accessed items.

For rustassistant targeting **500MB maximum cache size** on Raspberry Pi, watermark-based pruning provides smooth operation. When cache reaches **90% capacity (450MB)**, begin background pruning to **75% (375MB)**. This headroom prevents the system from thrashing between prune and rebuild cycles when actively analyzing files.

Cargo's garbage collection (available via nightly `gc` feature) uses tiered TTLs: **7 days for recreatable artifacts, 30 days for downloaded data**. For LLM analysis, a similar approach makes sense—keep expensive full-file analyses for 30 days, but allow cheaper incremental updates to expire in 7 days. Entries with `api_cost_millicents > 100` (roughly $0.001 or 5000 tokens at $0.20/million) should receive priority protection.

**Per-repository quotas** prevent a single large monorepo from consuming the entire cache. Setting a soft limit of 25% per repository ensures fair distribution while allowing overflow when other repos are inactive. The Quota Manager in Firefox enforces similar per-origin limits (20% of global, max 2GB per eTLD+1 group).

---

## SQLite provides the metadata backbone

SQLite excels as the primary storage for file analysis results, scores, change tracking, and cache metadata. It handles concurrent reads from multiple processes, supports transactions for atomic updates during pruning, and provides indexed queries for efficient lookups.

**Core schema design:**

```sql
CREATE TABLE file_analysis (
    id INTEGER PRIMARY KEY,
    file_path TEXT NOT NULL,
    content_hash TEXT NOT NULL,       -- SHA256 of file content
    cache_key TEXT UNIQUE NOT NULL,   -- Multi-factor cache key
    analysis_json BLOB NOT NULL,      -- Compressed analysis result
    
    -- Invalidation tracking
    model_id TEXT NOT NULL,
    prompt_hash TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    
    -- Cost tracking for eviction
    generation_time_ms INTEGER,
    api_cost_millicents INTEGER,
    input_tokens INTEGER,
    output_tokens INTEGER,
    
    -- Access tracking
    created_at INTEGER NOT NULL,
    last_accessed INTEGER NOT NULL,
    access_count INTEGER DEFAULT 1,
    size_bytes INTEGER NOT NULL,
    
    -- Repo association
    repo_id TEXT NOT NULL
);

CREATE INDEX idx_cache_key ON file_analysis(cache_key);
CREATE INDEX idx_last_accessed ON file_analysis(last_accessed);
CREATE INDEX idx_repo ON file_analysis(repo_id);
CREATE INDEX idx_content_hash ON file_analysis(content_hash);
```

For the actual analysis JSON, using **zstd compression** reduces storage by 60-80% compared to raw JSON. SQLite's BLOB type handles compressed data efficiently, and the rusqlite crate integrates well with zstd. A benchmark converting 18GB of JSON to SQLite achieved **4.8GB with multiple tables, 725MB with compressed content**—representing 25x reduction.

The `content_hash` index enables a critical optimization: when the same file exists in multiple repositories (shared libraries, vendored dependencies), the cache can serve results without re-analysis. This mirrors pnpm's content-addressable store, which deduplicates identical files across thousands of projects.

---

## LanceDB handles vector search for RAG

LanceDB provides serverless, embedded vector storage that integrates naturally with SQLite metadata. Its **Lance columnar format** stores embeddings on disk with memory-mapped access, making it suitable for Raspberry Pi's limited RAM. The hybrid search capability—combining vector similarity with full-text search—enables queries like "find functions similar to this authentication handler that also contain 'password' in comments."

**Embedding cache architecture:**

```rust
struct CodeChunk {
    // LanceDB vector field
    vector: Vector<384>,  // bge-small-en dimensions
    
    // Searchable text
    text: String,
    
    // Metadata for filtering
    file_path: String,
    content_hash: String,    // Links to SQLite file_analysis
    chunk_hash: String,      // Hash of this specific chunk
    model_id: String,        // Embedding model version
    entity_type: String,     // function, class, module, import
    entity_name: String,
    language: String,
    start_line: i32,
    end_line: i32,
}
```

For code, **AST-based chunking** significantly outperforms naive line-based splitting. Research from CMU shows **5.5 point gains** on code retrieval benchmarks when chunks respect function and class boundaries. Using tree-sitter for parsing, extract each function, class, and significant code block as a separate chunk with contextual metadata (parent scope, imports used).

**Hybrid search with reranking** achieves the best retrieval quality. LanceDB's built-in `RRFReranker` (Reciprocal Rank Fusion) combines vector similarity and BM25 keyword scores without requiring external APIs. Benchmarks show **92.4% hit rate** with hybrid search versus 81.3% for vector-only—a 14% improvement that translates directly to better LLM context.

The embedding model choice balances quality, speed, and cost. For local embedding on Raspberry Pi, **bge-small-en-v1.5** (384 dimensions, ~130MB model size) provides excellent quality with fast inference. ONNX runtime or OpenVINO quantization can provide 2-3x speedup on ARM64. Batch embeddings during overnight indexing runs, storing results in LanceDB with `chunk_hash` keys to skip re-embedding unchanged code.

LanceDB is **not git-friendly** (binary Lance format), so add `*.lance` and `.lancedb/` to `.gitignore`. Version the schema and configuration in git, not the vector data itself. For backup and portability, export metadata to JSON periodically.

---

## Dependency-aware invalidation minimizes re-analysis

When a Rust file changes, dependent files may need re-analysis—but only if the change affects their semantics. rust-analyzer's Salsa framework demonstrates **query-level granularity** with early cutoff: if recomputing a query produces the same result (e.g., whitespace-only change doesn't affect AST), dependent queries aren't invalidated.

**Implementing minimal invalidation:**

Track file dependencies at two levels—module structure (what files import this module) and type structure (what types from this file are used elsewhere). Store a directed graph in SQLite:

```sql
CREATE TABLE file_dependencies (
    source_file TEXT NOT NULL,      -- The file being depended upon
    dependent_file TEXT NOT NULL,   -- The file with the dependency
    dependency_type TEXT NOT NULL,  -- 'module', 'type', 'function'
    symbol_name TEXT,               -- Specific symbol if tracked
    PRIMARY KEY (source_file, dependent_file, dependency_type)
);
```

When a file changes, compute its new "interface hash"—a hash of only the public API (function signatures, type definitions, exported symbols) rather than implementation details. If the interface hash is unchanged, **don't propagate invalidation** to dependents. This mirrors Cargo's fingerprinting, which tracks whether public interfaces changed rather than any file modification.

The **durability system** from Salsa provides another optimization. Classify inputs by change frequency: user source files are LOW durability (change often), workspace dependencies are MEDIUM, and standard library types are HIGH. Maintain a version vector per durability level. When checking if a cached result is valid, compare only against the appropriate durability levels—avoiding traversal of stdlib queries when only user code changed.

For rustassistant, this means: don't re-analyze how a file uses `std::collections::HashMap` just because the user edited an unrelated function. The durability optimization reduced rust-analyzer's validation time from **~300ms to near-zero** for unchanged durability levels.

---

## Batch processing with rate limiting and crash recovery

Processing 100+ repositories requires a robust job queue with rate limiting, priority scheduling, and crash recovery. SQLite-based persistent queues enable resuming interrupted analysis after power loss or system restart—critical for overnight batch runs on Raspberry Pi.

**Queue architecture using the effectum crate:**

```rust
struct AnalysisJob {
    job_id: Uuid,
    repo_path: PathBuf,
    file_path: PathBuf,
    priority: JobPriority,
    created_at: Instant,
    retry_count: u32,
}

enum JobPriority {
    Critical,   // Entry points, main.rs, lib.rs
    High,       // Recently modified files
    Normal,     // Standard files
    Low,        // Generated code, tests
}
```

The **governor crate** provides token-bucket rate limiting for Grok API calls. Configure for your API limits (e.g., 60 requests/minute) with burst allowance:

```rust
let quota = Quota::per_minute(NonZeroU32::new(60).unwrap())
    .allow_burst(NonZeroU32::new(10).unwrap());
let rate_limiter = RateLimiter::direct(quota);
```

For the **$3/month budget at $0.20/million tokens**, you can process approximately **15 million tokens**. A typical file analysis request uses 2000-5000 tokens (input + output), allowing **3000-7500 file analyses per month**. With 10-50 repositories averaging 500 files each, this budget covers full analysis every 1-2 weeks with incremental updates in between. Track token usage per request and implement daily/weekly budget caps to prevent overage.

**Worker pool architecture** combines tokio for async I/O (API calls, file reads) with rayon for CPU-bound work (parsing, hashing). The critical rule: never call `tokio::block_on` from rayon threads. Use oneshot channels to bridge:

```rust
async fn analyze_file(content: String) -> Analysis {
    // CPU work on rayon
    let parsed = tokio_rayon::spawn(|| parse_ast(&content)).await;
    
    // IO work on tokio
    let analysis = api_client.analyze(&parsed).await?;
    
    analysis
}
```

**Retry logic** with exponential backoff handles transient API failures. The backoff crate provides async-native retry with jitter:

```rust
backoff::future::retry(
    ExponentialBackoff::default(),
    || async {
        match client.analyze(content).await {
            Ok(result) => Ok(result),
            Err(e) if e.status() == 429 => Err(backoff::Error::transient(e)),
            Err(e) => Err(backoff::Error::permanent(e)),
        }
    }
).await
```

A **dead letter queue** captures jobs that fail after max retries, enabling manual inspection and replay. Store failed jobs with error messages and timestamps in a separate SQLite table.

---

## Context window optimization for large codebases

Grok 4.1's 2-million-token context window enables including extensive code context, but research shows **focused, relevant chunks outperform naive dumping**. The goal is maximizing signal-to-noise ratio in the context provided to the LLM.

**Smart context construction pipeline:**

1. **Retrieve candidates**: Hybrid search (vector + keyword) returns top 50 relevant chunks
2. **Rerank for relevance**: RRF or cross-encoder scoring narrows to top 20
3. **Dependency expansion**: Include direct dependencies of selected functions
4. **Token budget management**: Greedy filling to target context size (8K-16K typical)
5. **Structural formatting**: Add file paths, scope context, import statements

For a typical analysis query, **8,000-16,000 tokens of context** provides optimal results. Beyond this, diminishing returns set in as the LLM struggles with needle-in-haystack retrieval from massive contexts. Reserve the remaining context budget for the analysis prompt and expected output.

**Contextualized chunk text** improves retrieval quality. Rather than embedding raw code, prepend structural context:

```
# src/auth/handler.rs
# Module: auth::handler
# Function: verify_token(token: &str) -> Result<Claims, AuthError>
# Uses: jsonwebtoken, AuthError

pub fn verify_token(token: &str) -> Result<Claims, AuthError> {
    let key = get_secret_key()?;
    jsonwebtoken::decode(token, &key, &Validation::default())
        .map_err(AuthError::from)
}
```

This format, inspired by CocoIndex patterns, provides the embedding model with scope and signature information that improves semantic matching for code search queries.

---

## Practical implementation for Raspberry Pi

Given the Pi's constraints (**4GB RAM, SD card storage, ARM64 CPU**), several optimizations ensure smooth operation.

**Memory management**: Use moka's bounded cache with 100-200MB maximum for hot in-memory entries. Let SQLite and LanceDB handle disk I/O for the long tail. Enable `use_mmap=True` in LanceDB for memory-efficient large dataset access.

**Storage optimization**: Target 500MB total cache size with aggressive pruning. Use zstd compression for analysis results (60-80% reduction). Consider NVMe/SSD storage over SD cards for 4-10x I/O improvement.

**Embedding model selection**: bge-small-en-v1.5 at 384 dimensions balances quality and efficiency. The model loads in ~500MB RAM and produces embeddings in ~50ms per chunk on ARM64. Batch embedding operations and run during idle periods.

**Estimated storage footprint for a medium codebase (10K files across 25 repos):**

| Component | Size |
|-----------|------|
| SQLite analysis metadata | 10-20 MB |
| Compressed analysis results | 50-100 MB |
| LanceDB vectors (384 dims) | 100-200 MB |
| Full-text search index | 20-50 MB |
| **Total** | **200-400 MB** |

This fits comfortably within the 500MB budget while leaving headroom for growth.

---

## Summary of key architectural decisions

The rustassistant cache architecture combines proven patterns from Cargo, Bazel, rust-analyzer, and modern RAG systems into a cohesive design optimized for resource-constrained solo development:

**Storage layer**: SQLite for structured metadata and analysis results, LanceDB for vector embeddings, both stored in a centralized `~/.rustassistant/cache/` directory with per-repository subdirectories keyed by path hash.

**Invalidation strategy**: Multi-factor cache keys combining file content hash, model version, prompt hash, and schema version. Content-based hashing with mtime optimization for performance. Lazy invalidation with early cutoff to minimize cascade re-analysis.

**Eviction policy**: Cost-weighted LRU using moka's TinyLFU algorithm. Watermark-based pruning at 90%/75% thresholds. Per-repository quotas at 25% of global cache. Priority protection for expensive LLM analyses.

**Batch processing**: SQLite-backed job queue with effectum for crash recovery. tokio/rayon hybrid worker pool. governor rate limiting for API calls. Exponential backoff with dead letter queue for failures.

**RAG pipeline**: AST-based code chunking with tree-sitter. Local bge-small embeddings for cost efficiency. Hybrid vector + keyword search with RRF reranking. Smart context construction with dependency expansion.

This architecture enables efficient analysis of 10-50 repositories within a $3/month budget while maintaining sub-second query response times and robust crash recovery on Raspberry Pi hardware.