# Cache Implementation Roadmap

**Based on:** `cache-research.md`  
**Target:** Phased rollout optimized for Raspberry Pi deployment  
**Budget:** $3/month ($0.20/M tokens = 15M tokens/month)  
**Timeline:** 4 weeks (2-3 hours per phase)

---

## Phase 0: Foundation (Week 1, 6 hours)

### 0.1 Migrate to Centralized Cache (3 hours)

**Goal:** Move from per-repo to centralized `~/.rustassistant/cache/repos/`

**Tasks:**
- [ ] Implement `CacheStrategy` enum (Centralized vs Local)
- [ ] Update `RepoCache::new()` to support both strategies
- [ ] Compute stable repo hash: `SHA256(canonical_path)` → use first 8 chars
- [ ] Create `meta.json` per repo: `{ "path": "/full/path", "hash": "...", "schema_version": 1 }`
- [ ] Add config support in `~/.rustassistant/config.toml`:
  ```toml
  [cache]
  strategy = "centralized"  # default
  location = "~/.rustassistant/cache"
  max_size_mb = 500
  ```

**Migration Script:**
```bash
#!/bin/bash
# scripts/migrate_to_centralized_cache.sh
mkdir -p ~/.rustassistant/cache/repos

# For each repo with local cache
for repo in ~/github/*/; do
    if [ -d "$repo/.rustassistant/cache" ]; then
        HASH=$(echo -n "$(realpath $repo)" | sha256sum | cut -c1-8)
        mkdir -p ~/.rustassistant/cache/repos/$HASH
        cp -r "$repo/.rustassistant/cache"/* ~/.rustassistant/cache/repos/$HASH/
        echo "{\"path\":\"$(realpath $repo)\",\"hash\":\"$HASH\",\"schema_version\":1}" > \
            ~/.rustassistant/cache/repos/$HASH/meta.json
    fi
done
```

**Success Criteria:**
- ✅ All existing caches migrated
- ✅ fks repo no longer triggers CI/CD on cache commits
- ✅ Cache lookups still work (verify with `rustassistant cache status`)

---

### 0.2 Multi-Factor Cache Keys (2 hours)

**Goal:** Prevent stale analysis from model/prompt changes

**Current Key:** `SHA256(file_content)`  
**New Key:** `SHA256(file_content + model_id + prompt_hash + schema_version + config_hash)`

**Implementation:**

```rust
// src/repo_cache.rs

pub struct CacheKey {
    pub file_hash: String,        // SHA256 of file content
    pub model_id: String,         // "grok-beta", "grok-4.1"
    pub prompt_hash: String,      // First 16 chars of SHA256(prompt_template)
    pub schema_version: u32,      // Current: 1
    pub config_hash: String,      // Hash of analysis config options
}

impl CacheKey {
    pub fn compute(&self) -> String {
        let combined = format!(
            "{}:{}:{}:{}:{}",
            self.file_hash,
            self.model_id,
            self.prompt_hash,
            self.schema_version,
            self.config_hash
        );
        sha256::digest(combined)
    }
}
```

**Prompt Hashing:**
```rust
// src/prompt_templates.rs

lazy_static! {
    pub static ref REFACTOR_PROMPT_HASH: String = {
        let template = include_str!("../prompts/refactor.txt");
        sha256::digest(template)[..16].to_string()
    };
    
    pub static ref DOCS_PROMPT_HASH: String = {
        let template = include_str!("../prompts/docs.txt");
        sha256::digest(template)[..16].to_string()
    };
}
```

**Update `RepoCacheEntry`:**
```rust
pub struct RepoCacheEntry {
    pub file_path: String,
    pub file_hash: String,
    pub cache_key: String,        // NEW: full multi-factor key
    pub analyzed_at: String,
    pub model_id: String,
    pub prompt_hash: String,      // NEW
    pub schema_version: u32,      // NEW
    pub config_hash: String,      // NEW
    pub result: serde_json::Value,
    pub tokens_used: Option<usize>,
    pub file_size: usize,
    pub cache_type: String,
}
```

**Success Criteria:**
- ✅ Changing prompt invalidates cache
- ✅ Changing model version invalidates cache
- ✅ Same file content + config = cache hit

---

### 0.3 Token Tracking (1 hour)

**Goal:** Capture actual token usage from API responses

**Implementation:**

```rust
// src/grok_client.rs

pub struct AnalysisResponse {
    pub content: String,
    pub tokens: TokenUsage,  // NEW
}

pub struct TokenUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
}

impl GrokClient {
    pub async fn analyze(&self, prompt: &str) -> Result<AnalysisResponse> {
        let response = self.client.post(...)
            .json(&json!({
                "messages": [...],
                "model": self.model,
            }))
            .send()
            .await?;
            
        let json: serde_json::Value = response.json().await?;
        
        // Extract token usage
        let usage = json["usage"].as_object()
            .ok_or_else(|| anyhow!("No usage data in response"))?;
            
        let tokens = TokenUsage {
            input_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as usize,
            output_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as usize,
            total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as usize,
        };
        
        Ok(AnalysisResponse {
            content: json["choices"][0]["message"]["content"]
                .as_str().unwrap_or("").to_string(),
            tokens,
        })
    }
}
```

**Update CLI to Store Tokens:**
```rust
// src/bin/cli.rs

let response = assistant.analyze_file(&file).await?;

cache.set(CacheSetParams {
    cache_type: CacheType::Refactor,
    file_path: &file,
    content: &file_content,
    provider: &config.llm.provider,
    model: &config.llm.model,
    result: serde_json::to_value(&response.content)?,
    tokens_used: Some(response.tokens.total_tokens),  // NOW POPULATED
})?;
```

**Success Criteria:**
- ✅ All cache entries have token counts
- ✅ `rustassistant cache status` shows total tokens used
- ✅ Cost estimation: `tokens * $0.20 / 1,000,000`

---

## Phase 1: SQLite Metadata Backbone (Week 2, 8 hours)

### 1.1 Database Schema Design (2 hours)

**Goal:** Replace JSON file cache with SQLite for better querying

**Schema:**
```sql
-- migrations/001_initial_cache_schema.sql

CREATE TABLE cache_metadata (
    schema_version INTEGER PRIMARY KEY,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

INSERT INTO cache_metadata VALUES (1, strftime('%s', 'now'), strftime('%s', 'now'));

CREATE TABLE file_analysis (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- File identification
    file_path TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    cache_key TEXT UNIQUE NOT NULL,
    
    -- Analysis result
    analysis_json BLOB NOT NULL,  -- zstd compressed
    analysis_type TEXT NOT NULL,  -- 'refactor', 'docs', 'analysis', 'todos'
    
    -- Invalidation tracking
    model_id TEXT NOT NULL,
    prompt_hash TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    config_hash TEXT NOT NULL,
    
    -- Cost tracking for eviction
    generation_time_ms INTEGER,
    api_cost_millicents INTEGER,  -- cost in 1/1000 cents
    input_tokens INTEGER,
    output_tokens INTEGER,
    total_tokens INTEGER,
    
    -- Access tracking for LRU
    created_at INTEGER NOT NULL,
    last_accessed INTEGER NOT NULL,
    access_count INTEGER DEFAULT 1,
    size_bytes INTEGER NOT NULL,  -- compressed size
    
    -- Repository association
    repo_id TEXT NOT NULL,  -- hash from meta.json
    repo_path TEXT NOT NULL
);

CREATE INDEX idx_cache_key ON file_analysis(cache_key);
CREATE INDEX idx_last_accessed ON file_analysis(last_accessed);
CREATE INDEX idx_repo ON file_analysis(repo_id);
CREATE INDEX idx_content_hash ON file_analysis(content_hash);
CREATE INDEX idx_api_cost ON file_analysis(api_cost_millicents DESC);

CREATE TABLE file_dependencies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_file TEXT NOT NULL,
    dependent_file TEXT NOT NULL,
    dependency_type TEXT NOT NULL,  -- 'module', 'type', 'function'
    symbol_name TEXT,
    repo_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE(source_file, dependent_file, dependency_type, repo_id)
);

CREATE INDEX idx_deps_source ON file_dependencies(source_file, repo_id);
CREATE INDEX idx_deps_dependent ON file_dependencies(dependent_file, repo_id);

CREATE TABLE pruning_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pruned_at INTEGER NOT NULL,
    entries_removed INTEGER NOT NULL,
    bytes_freed INTEGER NOT NULL,
    pruning_strategy TEXT NOT NULL,  -- 'lru', 'cost-aware', 'manual'
    total_entries_after INTEGER NOT NULL,
    total_bytes_after INTEGER NOT NULL
);
```

**Migration Framework:**
```rust
// src/cache_migrations.rs

use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> anyhow::Result<()> {
    let user_version: u32 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    
    let migrations = vec![
        include_str!("../migrations/001_initial_cache_schema.sql"),
        // Future migrations...
    ];
    
    for (i, migration) in migrations.iter().enumerate().skip(user_version as usize) {
        conn.execute_batch(migration)?;
        conn.pragma_update(None, "user_version", &(i + 1))?;
    }
    
    Ok(())
}
```

---

### 1.2 SQLite Cache Implementation (4 hours)

**New Module:** `src/sqlite_cache.rs`

```rust
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

pub struct SqliteCache {
    conn: Connection,
    repo_id: String,
    repo_path: PathBuf,
}

impl SqliteCache {
    pub fn new(repo_path: &Path) -> anyhow::Result<Self> {
        let repo_hash = compute_repo_hash(repo_path);
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("No home directory"))?
            .join(".rustassistant/cache/repos")
            .join(&repo_hash);
            
        fs::create_dir_all(&cache_dir)?;
        
        let db_path = cache_dir.join("analysis.db");
        let conn = Connection::open(&db_path)?;
        
        // Enable performance optimizations
        conn.execute_batch("
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA cache_size=-64000;  -- 64MB cache
            PRAGMA temp_store=MEMORY;
        ")?;
        
        crate::cache_migrations::run_migrations(&conn)?;
        
        Ok(Self {
            conn,
            repo_id: repo_hash,
            repo_path: repo_path.to_path_buf(),
        })
    }
    
    pub fn get(&mut self, key: &CacheKey) -> anyhow::Result<Option<CachedAnalysis>> {
        let cache_key = key.compute();
        
        let mut stmt = self.conn.prepare_cached("
            SELECT analysis_json, total_tokens, created_at, access_count
            FROM file_analysis
            WHERE cache_key = ?1
        ")?;
        
        let result = stmt.query_row(params![cache_key], |row| {
            let compressed: Vec<u8> = row.get(0)?;
            let tokens: Option<i64> = row.get(1)?;
            let created_at: i64 = row.get(2)?;
            let access_count: i64 = row.get(3)?;
            
            // Decompress
            let json_bytes = zstd::decode_all(&compressed[..])?;
            let result: serde_json::Value = serde_json::from_slice(&json_bytes)?;
            
            Ok(CachedAnalysis {
                result,
                tokens_used: tokens.map(|t| t as usize),
                created_at,
                access_count: access_count as usize,
            })
        });
        
        match result {
            Ok(analysis) => {
                // Update access tracking
                self.conn.execute("
                    UPDATE file_analysis
                    SET last_accessed = strftime('%s', 'now'),
                        access_count = access_count + 1
                    WHERE cache_key = ?1
                ", params![cache_key])?;
                
                Ok(Some(analysis))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    pub fn set(&mut self, key: &CacheKey, analysis: &AnalysisResult) -> anyhow::Result<()> {
        let cache_key = key.compute();
        
        // Compress JSON
        let json_bytes = serde_json::to_vec(&analysis.result)?;
        let compressed = zstd::encode_all(&json_bytes[..], 3)?;  // level 3 for speed
        
        let api_cost = analysis.tokens.total_tokens as f64 * 0.20 / 1_000_000.0;
        let api_cost_millicents = (api_cost * 100_000.0) as i64;
        
        self.conn.execute("
            INSERT OR REPLACE INTO file_analysis (
                file_path, content_hash, cache_key, analysis_json, analysis_type,
                model_id, prompt_hash, schema_version, config_hash,
                generation_time_ms, api_cost_millicents,
                input_tokens, output_tokens, total_tokens,
                created_at, last_accessed, access_count, size_bytes,
                repo_id, repo_path
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
                strftime('%s', 'now'), strftime('%s', 'now'), 1, ?15, ?16, ?17
            )
        ", params![
            key.file_path,
            key.file_hash,
            cache_key,
            compressed,
            analysis.cache_type,
            key.model_id,
            key.prompt_hash,
            key.schema_version,
            key.config_hash,
            analysis.generation_time_ms,
            api_cost_millicents,
            analysis.tokens.input_tokens as i64,
            analysis.tokens.output_tokens as i64,
            analysis.tokens.total_tokens as i64,
            compressed.len() as i64,
            self.repo_id,
            self.repo_path.display().to_string(),
        ])?;
        
        Ok(())
    }
    
    pub fn stats(&self) -> anyhow::Result<CacheStats> {
        let mut stmt = self.conn.prepare("
            SELECT 
                COUNT(*) as total_entries,
                SUM(size_bytes) as total_bytes,
                SUM(total_tokens) as total_tokens,
                SUM(api_cost_millicents) as total_cost_millicents,
                AVG(access_count) as avg_access_count
            FROM file_analysis
            WHERE repo_id = ?1
        ")?;
        
        stmt.query_row(params![self.repo_id], |row| {
            Ok(CacheStats {
                total_entries: row.get(0)?,
                total_bytes: row.get(1)?,
                total_tokens: row.get::<_, Option<i64>>(2)?.unwrap_or(0) as usize,
                total_cost_usd: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as f64 / 100_000.0,
                avg_access_count: row.get::<_, Option<f64>>(4)?.unwrap_or(0.0),
            })
        }).map_err(Into::into)
    }
}
```

---

### 1.3 Cost-Aware Pruning (2 hours)

**Goal:** Keep cache under 500MB using cost-weighted eviction

**Implementation:**
```rust
// src/cache_pruning.rs

pub struct CachePruner {
    conn: Connection,
    max_size_bytes: usize,
    watermark_high: f32,  // 0.90
    watermark_low: f32,   // 0.75
}

impl CachePruner {
    pub fn prune_to_watermark(&mut self) -> anyhow::Result<PruneStats> {
        let current_size = self.current_size()?;
        let max_size = self.max_size_bytes;
        
        if current_size < (max_size as f32 * self.watermark_high) as usize {
            return Ok(PruneStats::default());  // No pruning needed
        }
        
        let target_size = (max_size as f32 * self.watermark_low) as usize;
        let bytes_to_free = current_size - target_size;
        
        // Eviction score: lower = prune first
        // Formula: (1 / access_count) * (1 / api_cost) * age_factor
        let mut stmt = self.conn.prepare("
            SELECT id, size_bytes, api_cost_millicents, access_count,
                   (strftime('%s', 'now') - last_accessed) as age_seconds
            FROM file_analysis
            ORDER BY (
                1.0 / COALESCE(access_count, 1) *
                1.0 / COALESCE(api_cost_millicents, 1) *
                (1.0 + age_seconds / 86400.0)  -- age in days
            ) DESC  -- Highest score = evict first
        ")?;
        
        let candidates: Vec<(i64, usize)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        
        let mut freed = 0;
        let mut removed_ids = Vec::new();
        
        for (id, size) in candidates {
            if freed >= bytes_to_free {
                break;
            }
            removed_ids.push(id);
            freed += size;
        }
        
        // Delete in transaction
        let tx = self.conn.transaction()?;
        for id in &removed_ids {
            tx.execute("DELETE FROM file_analysis WHERE id = ?1", params![id])?;
        }
        tx.execute("
            INSERT INTO pruning_stats (
                pruned_at, entries_removed, bytes_freed, pruning_strategy,
                total_entries_after, total_bytes_after
            ) VALUES (
                strftime('%s', 'now'), ?1, ?2, 'cost-aware',
                (SELECT COUNT(*) FROM file_analysis),
                (SELECT SUM(size_bytes) FROM file_analysis)
            )
        ", params![removed_ids.len(), freed])?;
        tx.commit()?;
        
        Ok(PruneStats {
            entries_removed: removed_ids.len(),
            bytes_freed: freed,
        })
    }
}
```

**CLI Command:**
```bash
rustassistant cache prune [--max-size 500M] [--strategy cost-aware]
```

---

## Phase 2: LanceDB Vector Storage (Week 3, 10 hours)

### 2.1 Setup LanceDB (2 hours)

**Dependencies:**
```toml
[dependencies]
lancedb = "0.9"
arrow-array = "53"
```

**Initialize:**
```rust
// src/vector_cache.rs

use lancedb::{Connection, Table};

pub struct VectorCache {
    db: Connection,
    table: Table,
    repo_id: String,
}

impl VectorCache {
    pub async fn new(repo_path: &Path) -> anyhow::Result<Self> {
        let repo_hash = compute_repo_hash(repo_path);
        let lance_path = dirs::home_dir()
            .ok_or_else(|| anyhow!("No home directory"))?
            .join(".rustassistant/cache/repos")
            .join(&repo_hash)
            .join("embeddings.lance");
            
        let db = lancedb::connect(&lance_path).execute().await?;
        
        // Create or open table
        let table = if db.table_names().await?.contains(&"code_chunks".to_string()) {
            db.open_table("code_chunks").execute().await?
        } else {
            // Create schema
            db.create_empty_table("code_chunks", create_schema()).await?
        };
        
        Ok(Self {
            db,
            table,
            repo_id: repo_hash,
        })
    }
}

fn create_schema() -> arrow_schema::Schema {
    arrow_schema::Schema::new(vec![
        arrow_schema::Field::new("chunk_id", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new(
            "vector",
            arrow_schema::DataType::FixedSizeList(
                Arc::new(arrow_schema::Field::new("item", arrow_schema::DataType::Float32, true)),
                384,  // bge-small-en dimensions
            ),
            false,
        ),
        arrow_schema::Field::new("text", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("file_path", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("content_hash", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("chunk_hash", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("model_id", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("entity_type", arrow_schema::DataType::Utf8, true),
        arrow_schema::Field::new("entity_name", arrow_schema::DataType::Utf8, true),
        arrow_schema::Field::new("language", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("start_line", arrow_schema::DataType::Int32, false),
        arrow_schema::Field::new("end_line", arrow_schema::DataType::Int32, false),
    ])
}
```

---

### 2.2 AST-Based Code Chunking (4 hours)

**Use tree-sitter for parsing:**
```toml
[dependencies]
tree-sitter = "0.24"
tree-sitter-rust = "0.23"
```

**Implementation:**
```rust
// src/code_chunker.rs

use tree_sitter::{Parser, Language, Node};

extern "C" { fn tree_sitter_rust() -> Language; }

pub struct CodeChunker {
    parser: Parser,
}

#[derive(Debug, Clone)]
pub struct CodeChunk {
    pub text: String,
    pub entity_type: String,  // "function", "struct", "impl", "module"
    pub entity_name: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub context: String,  // parent scope, imports
}

impl CodeChunker {
    pub fn new() -> anyhow::Result<Self> {
        let mut parser = Parser::new();
        let language = unsafe { tree_sitter_rust() };
        parser.set_language(language)?;
        
        Ok(Self { parser })
    }
    
    pub fn chunk_file(&mut self, content: &str, file_path: &Path) -> anyhow::Result<Vec<CodeChunk>> {
        let tree = self.parser.parse(content, None)
            .ok_or_else(|| anyhow!("Failed to parse file"))?;
            
        let mut chunks = Vec::new();
        self.extract_chunks(tree.root_node(), content, &mut chunks)?;
        
        Ok(chunks)
    }
    
    fn extract_chunks(&self, node: Node, source: &str, chunks: &mut Vec<CodeChunk>) -> anyhow::Result<()> {
        match node.kind() {
            "function_item" | "struct_item" | "impl_item" | "trait_item" => {
                let text = node.utf8_text(source.as_bytes())?;
                let name = self.extract_name(&node, source)?;
                
                chunks.push(CodeChunk {
                    text: text.to_string(),
                    entity_type: node.kind().replace("_item", ""),
                    entity_name: name,
                    start_line: node.start_position().row + 1,
                    end_line: node.end_position().row + 1,
                    context: self.build_context(&node, source)?,
                });
            }
            _ => {
                // Recurse into children
                for child in node.children(&mut node.walk()) {
                    self.extract_chunks(child, source, chunks)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn extract_name(&self, node: &Node, source: &str) -> anyhow::Result<Option<String>> {
        for child in node.children(&mut node.walk()) {
            if child.kind() == "identifier" {
                return Ok(Some(child.utf8_text(source.as_bytes())?.to_string()));
            }
        }
        Ok(None)
    }
    
    fn build_context(&self, node: &Node, source: &str) -> anyhow::Result<String> {
        // Add parent module path, imports, etc.
        // Simplified version:
        Ok(String::new())
    }
}
```

---

### 2.3 Local Embedding Generation (4 hours)

**Use Candle for local inference:**
```toml
[dependencies]
candle-core = "0.8"
candle-nn = "0.8"
candle-transformers = "0.8"
tokenizers = "0.20"
```

**Download bge-small-en-v1.5:**
```bash
mkdir -p ~/.rustassistant/models
cd ~/.rustassistant/models
wget https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/model.safetensors
wget https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/tokenizer.json
wget https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/config.json
```

**Implementation:**
```rust
// src/embeddings.rs

use candle_core::{Device, Tensor};
use candle_transformers::models::bert::{BertModel, Config};
use tokenizers::Tokenizer;

pub struct EmbeddingModel {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl EmbeddingModel {
    pub fn load() -> anyhow::Result<Self> {
        let model_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("No home dir"))?
            .join(".rustassistant/models/bge-small-en-v1.5");
            
        let device = Device::Cpu;  // Or Device::cuda(0) if available
        
        let config = Config::from_file(model_dir.join("config.json"))?;
        let weights = candle_core::safetensors::load(model_dir.join("model.safetensors"), &device)?;
        let model = BertModel::load(weights, &config)?;
        
        let tokenizer = Tokenizer::from_file(model_dir.join("tokenizer.json"))?;
        
        Ok(Self { model, tokenizer, device })
    }
    
    pub fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let encoding = self.tokenizer.encode(text, true)?;
        let token_ids = Tensor::new(encoding.get_ids(), &self.device)?;
        
        let output = self.model.forward(&token_ids)?;
        
        // Mean pooling
        let embeddings = output.mean(1)?;
        
        // L2 normalize
        let norm = embeddings.sqr()?.sum_keepdim(1)?.sqrt()?;
        let normalized = embeddings.broadcast_div(&norm)?;
        
        normalized.to_vec1::<f32>().map_err(Into::into)
    }
    
    pub fn embed_batch(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
}
```

**Add to VectorCache:**
```rust
impl VectorCache {
    pub async fn add_chunks(&mut self, chunks: Vec<CodeChunk>, embeddings: Vec<Vec<f32>>) -> anyhow::Result<()> {
        // Convert to Arrow arrays
        // Insert into LanceDB table
        // ...
    }
    
    pub async fn search(&self, query_embedding: Vec<f32>, limit: usize) -> anyhow::Result<Vec<CodeChunk>> {
        let results = self.table
            .vector_search(query_embedding)?
            .limit(limit)
            .execute()
            .await?;
            
        // Convert Arrow records back to CodeChunk
        // ...
    }
}
```

---

## Phase 3: Batch Processing & Queue (Week 4, 8 hours)

### 3.1 Job Queue with Effectum (3 hours)

```toml
[dependencies]
effectum = "1.7"
uuid = "1.11"
```

**Implementation:**
```rust
// src/batch_processor.rs

use effectum::{Queue, Job, Worker, RunningJob};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisJob {
    pub job_id: Uuid,
    pub repo_path: PathBuf,
    pub file_path: PathBuf,
    pub analysis_type: CacheType,
    pub priority: JobPriority,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Critical = 0,   // main.rs, lib.rs
    High = 1,       // Recently modified
    Normal = 2,     // Standard files
    Low = 3,        // Tests, generated
}

impl Job for AnalysisJob {
    type Output = AnalysisResult;
    type Error = anyhow::Error;
    
    async fn run(&self, _context: &RunningJob) -> Result<Self::Output, Self::Error> {
        // Perform analysis
        let cache = SqliteCache::new(&self.repo_path)?;
        let content = fs::read_to_string(&self.file_path)?;
        
        // Check cache first
        let key = CacheKey::compute(&self.file_path, &content, ...);
        if let Some(cached) = cache.get(&key)? {
            return Ok(cached);
        }
        
        // Analyze
        let result = analyze_file(&content, self.analysis_type).await?;
        
        // Cache result
        cache.set(&key, &result)?;
        
        Ok(result)
    }
}

pub struct BatchProcessor {
    queue: Queue,
}

impl BatchProcessor {
    pub async fn new() -> anyhow::Result<Self> {
        let db_path = dirs::home_dir()
            .ok_or_else(|| anyhow!("No home dir"))?
            .join(".rustassistant/cache/job_queue.db");
            
        let queue = Queue::new(&db_path).await?;
        
        Ok(Self { queue })
    }
    
    pub async fn enqueue(&self, job: AnalysisJob) -> anyhow::Result<Uuid> {
        self.queue.insert(job).await.map_err(Into::into)
    }
    
    pub async fn process(&self, max_workers: usize) -> anyhow::Result<()> {
        let worker = Worker::new(self.queue.clone(), max_workers);
        worker.run().await.map_err(Into::into)
    }
}
```

---

### 3.2 Rate Limiting (2 hours)

```toml
[dependencies]
governor = "0.7"
```

**Implementation:**
```rust
// src/rate_limiter.rs

use governor::{Quota, RateLimiter, clock::DefaultClock};
use std::num::NonZeroU32;

pub struct ApiRateLimiter {
    limiter: RateLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, DefaultClock>,
}

impl ApiRateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap())
            .allow_burst(NonZeroU32::new(10).unwrap());
            
        let limiter = RateLimiter::direct(quota);
        
        Self { limiter }
    }
    
    pub async fn wait_for_token(&self) {
        self.limiter.until_ready().await;
    }
}

// Use in GrokClient:
impl GrokClient {
    pub async fn analyze_with_rate_limit(&self, content: &str) -> anyhow::Result<AnalysisResponse> {
        self.rate_limiter.wait_for_token().await;
        self.analyze(content).await
    }
}
```

---

### 3.3 Budget Tracking (3 hours)

```rust
// src/budget_tracker.rs

pub struct BudgetTracker {
    conn: Connection,
    monthly_budget_usd: f64,  // $3.00
}

impl BudgetTracker {
    pub fn new(monthly_budget_usd: f64) -> anyhow::Result<Self> {
        let db_path = dirs::home_dir()
            .ok_or_else(|| anyhow!("No home dir"))?
            .join(".rustassistant/cache/budget.db");
            
        let conn = Connection::open(db_path)?;
        
        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS token_usage (
                id INTEGER PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                repo_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                analysis_type TEXT NOT NULL,
                input_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                total_tokens INTEGER NOT NULL,
                cost_usd REAL NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_timestamp ON token_usage(timestamp);
        ")?;
        
        Ok(Self { conn, monthly_budget_usd })
    }
    
    pub fn record_usage(&self, tokens: &TokenUsage, repo_id: &str, file_path: &str, analysis_type: &str) -> anyhow::Result<()> {
        let cost = tokens.total_tokens as f64 * 0.20 / 1_000_000.0;
        
        self.conn.execute("
            INSERT INTO token_usage (
                timestamp, repo_id, file_path, analysis_type,
                input_tokens, output_tokens, total_tokens, cost_usd
            ) VALUES (
                strftime('%s', 'now'), ?1, ?2, ?3, ?4, ?5, ?6, ?7
            )
        ", params![repo_id, file_path, analysis_type,
                   tokens.input_tokens, tokens.output_tokens,
                   tokens.total_tokens, cost])?;
        
        Ok(())
    }
    
    pub fn current_month_cost(&self) -> anyhow::Result<f64> {
        let month_start = chrono::Utc::now()
            .with_day(1).unwrap()
            .timestamp();
            
        self.conn.query_row(
            "SELECT COALESCE(SUM(cost_usd), 0) FROM token_usage WHERE timestamp >= ?1",
            params![month_start],
            |row| row.get(0)
        ).map_err(Into::into)
    }
    
    pub fn check_budget(&self) -> anyhow::Result<BudgetStatus> {
        let spent = self.current_month_cost()?;
        let remaining = self.monthly_budget_usd - spent;
        let percent_used = (spent / self.monthly_budget_usd) * 100.0;
        
        Ok(BudgetStatus {
            spent_usd: spent,
            remaining_usd: remaining,
            percent_used,
            over_budget: spent > self.monthly_budget_usd,
        })
    }
}
```

**CLI Commands:**
```bash
rustassistant budget status
rustassistant budget set --monthly 3.00
rustassistant budget history --days 30
```

---

## Phase 4: Integration & Testing (Ongoing)

### 4.1 Update CLI Commands

**Batch build:**
```bash
rustassistant cache build <repo> [--types refactor,docs] [--priority normal] [--max-files 100]
```

**Cache sync:**
```bash
rustassistant cache sync <repo> [--stale-after 7d]
```

**Budget check:**
```bash
rustassistant budget status
# Output:
# 📊 Budget Status (January 2025)
# Spent: $1.23 / $3.00 (41%)
# Remaining: $1.77
# Estimated files remaining: ~4,250
```

---

### 4.2 Testing Checklist

- [ ] Centralized cache migration works
- [ ] Multi-factor keys invalidate correctly
- [ ] Token tracking captures usage
- [ ] SQLite cache faster than JSON
- [ ] Pruning keeps cache under 500MB
- [ ] LanceDB vector search returns relevant results
- [ ] Batch processing completes without errors
- [ ] Rate limiting prevents API errors
- [ ] Budget tracking accurate within 5%
- [ ] Crash recovery works (kill during batch)

---

## Success Metrics

**Performance:**
- ✅ Cache lookup: <10ms (SQLite)
- ✅ Vector search: <100ms (LanceDB)
- ✅ Embedding generation: <50ms/chunk (bge-small)
- ✅ Full repo scan: 10-50 files/minute (rate limited)

**Cost:**
- ✅ Monthly budget: ≤$3.00
- ✅ Cache hit rate: >60%
- ✅ Cost per file: ~$0.001

**Storage:**
- ✅ Total cache: <500MB
- ✅ Per-repo: <25MB average
- ✅ Compression ratio: 60-80%

**Reliability:**
- ✅ Zero data loss on crash
- ✅ Automatic cache invalidation
- ✅ Graceful degradation (cache miss → re-analyze)

---

## Timeline Summary

| Week | Phase | Hours | Key Deliverables |
|------|-------|-------|-----------------|
| 1 | Foundation | 6 | Centralized cache, multi-factor keys, token tracking |
| 2 | SQLite | 8 | Database schema, cache impl, pruning |
| 3 | LanceDB | 10 | Vector storage, chunking, embeddings |
| 4 | Batch | 8 | Job queue, rate limiting, budget tracking |

**Total:** 32 hours over 4 weeks

---

## Next Steps

1. **Start with Phase 0.1:** Centralized cache migration (3 hours)
   - Solves the fks CI/CD problem immediately
   - Foundation for all other phases

2. **Verify with real usage:** Test on rustassistant + fks repos
   - Measure cache hit rates
   - Track actual costs
   - Identify bottlenecks

3. **Iterate based on data:** Adjust pruning, rate limits, budget based on real usage patterns

---

**Ready to begin? Start with Phase 0.1 - Centralized Cache Migration!**