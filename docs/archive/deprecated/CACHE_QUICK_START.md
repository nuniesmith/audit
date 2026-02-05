# Cache Implementation Quick Start

**Goal:** Get the improved cache architecture running in 1 hour  
**Based on:** `cache-research.md` and `CACHE_IMPLEMENTATION_ROADMAP.md`

---

## Step 1: Centralized Cache Migration (30 minutes)

### 1.1 Update RepoCache to Support Centralized Location

**Edit `src/repo_cache.rs`:**

```rust
// Add at top of file
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Add new enum before RepoCache struct
pub enum CacheStrategy {
    Centralized,  // ~/.rustassistant/cache/repos/<hash>/
    Local,        // <repo>/.rustassistant/cache/
}

impl Default for CacheStrategy {
    fn default() -> Self {
        Self::Centralized
    }
}

// Helper function to compute stable repo hash
fn compute_repo_hash(path: &Path) -> String {
    let canonical = path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    let path_str = canonical.display().to_string();
    
    // Use SHA256 for consistency with file hashing
    let hash = sha256::digest(path_str.as_bytes());
    hash[..8].to_string()
}

// Update RepoCache::new()
impl RepoCache {
    pub fn new_with_strategy(
        repo_root: impl AsRef<Path>,
        strategy: CacheStrategy,
    ) -> anyhow::Result<Self> {
        let repo_path = repo_root.as_ref();
        
        let cache_dir = match strategy {
            CacheStrategy::Centralized => {
                let repo_hash = compute_repo_hash(repo_path);
                let base = dirs::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("No home directory"))?
                    .join(".rustassistant/cache/repos")
                    .join(&repo_hash);
                
                // Create meta.json with repo info
                std::fs::create_dir_all(&base)?;
                let meta = serde_json::json!({
                    "path": repo_path.display().to_string(),
                    "hash": repo_hash,
                    "schema_version": 1,
                    "created_at": chrono::Utc::now().to_rfc3339(),
                });
                std::fs::write(
                    base.join("meta.json"),
                    serde_json::to_string_pretty(&meta)?
                )?;
                
                base
            }
            CacheStrategy::Local => {
                repo_path.join(REPO_CACHE_DIR)
            }
        };

        let cache = Self {
            cache_dir,
            enabled: true,
        };

        cache.ensure_cache_structure()?;
        Ok(cache)
    }
    
    // Keep old API for backwards compatibility
    pub fn new(repo_root: impl AsRef<Path>) -> anyhow::Result<Self> {
        Self::new_with_strategy(repo_root, CacheStrategy::default())
    }
}
```

### 1.2 Update CLI to Use Centralized Cache

**Edit `src/bin/cli.rs`:**

```rust
// In both refactor and docs handlers, change:
let cache = RepoCache::new(&repo_path)?;

// To:
let cache = RepoCache::new_with_strategy(&repo_path, CacheStrategy::Centralized)?;
```

### 1.3 Migration Script

**Create `scripts/migrate_to_centralized.sh`:**

```bash
#!/bin/bash
set -e

echo "🔄 Migrating to centralized cache..."

# Create centralized cache directory
mkdir -p ~/.rustassistant/cache/repos

# Function to migrate one repo
migrate_repo() {
    local repo_path="$1"
    local cache_dir="$repo_path/.rustassistant/cache"
    
    if [ ! -d "$cache_dir" ]; then
        return
    fi
    
    echo "📦 Migrating: $repo_path"
    
    # Compute hash
    local canonical_path=$(realpath "$repo_path")
    local hash=$(echo -n "$canonical_path" | sha256sum | cut -c1-8)
    
    # Create centralized location
    local dest="$HOME/.rustassistant/cache/repos/$hash"
    mkdir -p "$dest"
    
    # Copy cache files
    if [ -d "$cache_dir/refactor" ]; then
        mkdir -p "$dest/cache/refactor"
        cp -r "$cache_dir/refactor"/* "$dest/cache/refactor/" 2>/dev/null || true
    fi
    
    if [ -d "$cache_dir/docs" ]; then
        mkdir -p "$dest/cache/docs"
        cp -r "$cache_dir/docs"/* "$dest/cache/docs/" 2>/dev/null || true
    fi
    
    # Create meta.json
    cat > "$dest/meta.json" <<EOF
{
  "path": "$canonical_path",
  "hash": "$hash",
  "schema_version": 1,
  "migrated_at": "$(date -Iseconds)"
}
EOF
    
    echo "  ✓ Migrated to: $dest"
    echo "  ✓ Hash: $hash"
}

# Migrate known repos
if [ -d ~/github/rustassistant/.rustassistant ]; then
    migrate_repo ~/github/rustassistant
fi

if [ -d ~/github/fks/.rustassistant ]; then
    migrate_repo ~/github/fks
fi

# Find and migrate any other repos
echo ""
echo "🔍 Searching for other repos with cache..."
find ~/github -maxdepth 2 -type d -name ".rustassistant" 2>/dev/null | while read cache_path; do
    repo_path=$(dirname "$cache_path")
    migrate_repo "$repo_path"
done

echo ""
echo "✅ Migration complete!"
echo ""
echo "📊 Cache location summary:"
ls -lh ~/.rustassistant/cache/repos/ 2>/dev/null || echo "  (empty)"
echo ""
echo "⚠️  To remove old local caches after verifying:"
echo "   find ~/github -name '.rustassistant' -type d -exec rm -rf {} +"
```

**Run it:**
```bash
chmod +x scripts/migrate_to_centralized.sh
./scripts/migrate_to_centralized.sh
```

---

## Step 2: Multi-Factor Cache Keys (20 minutes)

### 2.1 Update CacheSetParams

**Edit `src/repo_cache.rs`:**

```rust
// Update CacheSetParams struct
pub struct CacheSetParams<'a> {
    pub cache_type: CacheType,
    pub file_path: &'a str,
    pub content: &'a str,
    pub provider: &'a str,
    pub model: &'a str,
    pub result: serde_json::Value,
    pub tokens_used: Option<usize>,
    
    // NEW: For cache key computation
    pub prompt_hash: Option<&'a str>,
    pub config_hash: Option<&'a str>,
}

// Update RepoCacheEntry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCacheEntry {
    pub file_path: String,
    pub file_hash: String,
    pub cache_key: String,        // NEW: multi-factor key
    pub analyzed_at: String,
    pub provider: String,
    pub model: String,
    pub prompt_hash: String,      // NEW
    pub schema_version: u32,      // NEW
    pub result: serde_json::Value,
    pub tokens_used: Option<usize>,
    pub file_size: usize,
    pub cache_type: String,
}

// Update set() method
pub fn set(&self, params: CacheSetParams) -> anyhow::Result<()> {
    if !self.enabled {
        return Ok(());
    }
    
    // Compute multi-factor cache key
    let prompt_hash = params.prompt_hash.unwrap_or("default");
    let config_hash = params.config_hash.unwrap_or("default");
    let schema_version = 1u32;
    
    let cache_key = format!(
        "{}:{}:{}:{}:{}",
        self.hash_content(params.content),
        params.model,
        prompt_hash,
        schema_version,
        config_hash
    );
    let cache_key = sha256::digest(cache_key.as_bytes());

    let entry = RepoCacheEntry {
        file_path: params.file_path.to_string(),
        file_hash: self.hash_content(params.content),
        cache_key,  // Use multi-factor key
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        provider: params.provider.to_string(),
        model: params.model.to_string(),
        prompt_hash: prompt_hash.to_string(),
        schema_version,
        result: params.result,
        tokens_used: params.tokens_used,
        file_size: params.content.len(),
        cache_type: params.cache_type.subdirectory().to_string(),
    };

    // ... rest of method
}

// Update get() to check cache_key
pub fn get(
    &self,
    cache_type: CacheType,
    file_path: &str,
    content: &str,
    model: &str,
    prompt_hash: Option<&str>,
) -> anyhow::Result<Option<RepoCacheEntry>> {
    if !self.enabled {
        return Ok(None);
    }

    let cache_file = self.cache_file_path(cache_type, file_path);

    if !cache_file.exists() {
        return Ok(None);
    }

    let entry: RepoCacheEntry = serde_json::from_str(&fs::read_to_string(&cache_file)?)?;

    // Verify cache key matches
    let expected_prompt_hash = prompt_hash.unwrap_or("default");
    let expected_cache_key = format!(
        "{}:{}:{}:{}:{}",
        self.hash_content(content),
        model,
        expected_prompt_hash,
        1u32,  // schema_version
        "default"  // config_hash
    );
    let expected_cache_key = sha256::digest(expected_cache_key.as_bytes());
    
    if entry.cache_key != expected_cache_key {
        debug!(
            "Cache MISS: key mismatch for {} (expected: {}, got: {})",
            file_path, expected_cache_key, entry.cache_key
        );
        return Ok(None);
    }

    debug!("Cache HIT: {} / {}", cache_type.subdirectory(), file_path);
    Ok(Some(entry))
}
```

---

## Step 3: Token Tracking (10 minutes)

### 3.1 Extract Tokens from Grok Response

**Edit `src/grok_client.rs`:**

```rust
// Add to response parsing
pub async fn chat(&self, messages: Vec<Message>) -> Result<String> {
    // ... existing code ...
    
    let json: serde_json::Value = response.json().await?;
    
    // Extract token usage if available
    if let Some(usage) = json["usage"].as_object() {
        let input_tokens = usage["prompt_tokens"].as_u64().unwrap_or(0);
        let output_tokens = usage["completion_tokens"].as_u64().unwrap_or(0);
        
        tracing::info!(
            "Token usage - Input: {}, Output: {}, Total: {}",
            input_tokens,
            output_tokens,
            input_tokens + output_tokens
        );
        
        // Store in thread-local or return with response
        // For now, just log it
    }
    
    // ... rest of method
}
```

**Better: Return token info from analysis methods:**

```rust
// In refactor_assistant.rs and doc_generator.rs
pub struct AnalysisResult<T> {
    pub content: T,
    pub tokens_used: Option<usize>,
}

// Update analyze_file to return tokens
pub async fn analyze_file(&self, file_path: &str) -> Result<AnalysisResult<RefactorAnalysis>> {
    // ... existing analysis code ...
    
    // After getting response from Grok
    let tokens = extract_tokens_from_response(&response)?;
    
    Ok(AnalysisResult {
        content: analysis,
        tokens_used: Some(tokens),
    })
}
```

---

## Step 4: Test the Changes (5 minutes)

### 4.1 Verify Centralized Cache Works

```bash
# Build
cargo build --release

# Test cache init
./target/release/rustassistant cache init

# Check location
ls -la ~/.rustassistant/cache/repos/

# Analyze a file
./target/release/rustassistant refactor analyze src/repo_cache.rs

# Check cache was created in centralized location
find ~/.rustassistant/cache/repos -name "*.json" -type f

# Analyze same file again - should be instant (cache hit)
time ./target/release/rustassistant refactor analyze src/repo_cache.rs
```

### 4.2 Verify fks Repo No Longer Has Cache

```bash
cd ~/github/fks
ls -la .rustassistant/  # Should not exist or be empty

# Analyze a file
rustassistant refactor analyze src/main.rs

# Verify cache went to centralized location, not local
ls -la .rustassistant/  # Still should not exist
ls -la ~/.rustassistant/cache/repos/  # Should have new entry
```

### 4.3 Commit Changes (Clean Repos)

```bash
# In fks repo
cd ~/github/fks
git rm -rf .rustassistant/ 2>/dev/null || true
echo ".rustassistant/" >> .gitignore
git add .gitignore
git commit -m "chore: remove local cache (using centralized cache now)"
git push

# This commit should NOT trigger CI/CD! ✅
```

---

## Step 5: Add Config File (5 minutes)

**Create `~/.rustassistant/config.toml`:**

```toml
[cache]
# "centralized" (default) | "local"
strategy = "centralized"

# Maximum cache size in MB
max_size_mb = 500

# Pruning watermarks
prune_at_percent = 90
prune_to_percent = 75

[budget]
# Monthly budget in USD
monthly_budget_usd = 3.00

# Token cost per million tokens
cost_per_million_tokens = 0.20

[llm]
provider = "xai"
model = "grok-beta"

# Optional: custom endpoints
# api_url = "https://api.x.ai/v1"
```

**Load config in CLI:**

```rust
// src/config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub cache: CacheConfig,
    pub budget: BudgetConfig,
    pub llm: LlmConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    #[serde(default)]
    pub strategy: String,  // "centralized" or "local"
    #[serde(default = "default_max_size")]
    pub max_size_mb: usize,
    #[serde(default = "default_prune_at")]
    pub prune_at_percent: f32,
    #[serde(default = "default_prune_to")]
    pub prune_to_percent: f32,
}

fn default_max_size() -> usize { 500 }
fn default_prune_at() -> f32 { 90.0 }
fn default_prune_to() -> f32 { 75.0 }

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("No home directory"))?
            .join(".rustassistant/config.toml");
            
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content).map_err(Into::into)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cache: CacheConfig {
                strategy: "centralized".to_string(),
                max_size_mb: 500,
                prune_at_percent: 90.0,
                prune_to_percent: 75.0,
            },
            budget: BudgetConfig {
                monthly_budget_usd: 3.0,
                cost_per_million_tokens: 0.20,
            },
            llm: LlmConfig {
                provider: "xai".to_string(),
                model: "grok-beta".to_string(),
            },
        }
    }
}
```

---

## Done! 🎉

You now have:
- ✅ Centralized cache (no more CI/CD pollution)
- ✅ Multi-factor cache keys (model/prompt versioning)
- ✅ Token tracking foundation
- ✅ Config file support

**Next steps:**
1. Use it for a few days and monitor cache hit rates
2. Implement SQLite backend (Phase 1)
3. Add LanceDB for vector search (Phase 2)
4. Add batch processing (Phase 3)

**Verify it's working:**
```bash
# Show cache statistics
rustassistant cache status

# Should show centralized location and cache hits
```

**Cost savings estimate:**
- Cache hit rate: 60%+
- Cost per analysis: ~$0.001
- Monthly budget: $3.00
- Files you can analyze: ~7,500/month with 60% hit rate = ~18,750 effective analyses