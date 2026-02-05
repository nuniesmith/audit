# Phase 1 Quick Start Guide

**Goal:** Implement Query Intelligence features to optimize Grok API costs

**Estimated Time:** 1-2 days  
**Difficulty:** Intermediate  
**Value:** High (reduces API costs by 60-80%)

---

## 🎯 What You're Building

Three core features that will drastically reduce your Grok API costs:

1. **Query Router** - Routes queries intelligently (already implemented in `src/query_router.rs`)
2. **Cost Tracker** - Monitors spending (already implemented in `src/cost_tracker.rs`)
3. **Content Deduplication** - Prevents duplicate notes

---

## ✅ Prerequisites

You already have:
- ✅ `src/query_router.rs` - Query intent classification
- ✅ `src/cost_tracker.rs` - Cost tracking and budgets
- ✅ `src/response_cache.rs` - Response caching
- ✅ `src/context_builder.rs` - Context assembly
- ✅ Database schema (notes, repos, tasks)

---

## 📝 Step 1: Database Migration

Add cost tracking table and note deduplication:

```sql
-- File: migrations/003_phase1_features.sql

-- Cost tracking table
CREATE TABLE IF NOT EXISTS llm_costs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    operation TEXT NOT NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    cached_tokens INTEGER DEFAULT 0,
    cost_usd REAL NOT NULL,
    query_hash TEXT,
    cache_hit BOOLEAN DEFAULT FALSE,
    user_query TEXT,
    response_summary TEXT
);

CREATE INDEX IF NOT EXISTS idx_costs_timestamp ON llm_costs(timestamp);
CREATE INDEX IF NOT EXISTS idx_costs_operation ON llm_costs(operation);
CREATE INDEX IF NOT EXISTS idx_costs_cache_hit ON llm_costs(cache_hit);

-- Add hash columns to notes table for deduplication
ALTER TABLE notes ADD COLUMN content_hash TEXT;
ALTER TABLE notes ADD COLUMN normalized_content TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_notes_content_hash ON notes(content_hash);
```

**Run migration:**

```bash
# Manual migration (we'll add automatic migrations later)
sqlite3 data/rustassistant.db < migrations/003_phase1_features.sql
```

---

## 🔧 Step 2: Update Database Module

Add deduplication to `src/db.rs`:

```rust
// Add to db.rs

use sha2::{Digest, Sha256};

/// Normalize content for deduplication
fn normalize_content(content: &str) -> String {
    content
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Generate SHA-256 hash
fn sha256_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Check for duplicate note by hash
async fn get_note_by_hash(pool: &SqlitePool, hash: &str) -> DbResult<Option<Note>> {
    sqlx::query_as::<_, Note>(
        r#"
        SELECT id, content, tags, project, status, created_at, updated_at
        FROM notes
        WHERE content_hash = ?
        "#,
    )
    .bind(hash)
    .fetch_optional(pool)
    .await
    .map_err(|e| DbError::QueryFailed(e.to_string()))
}

// Update create_note function:
pub async fn create_note(
    pool: &SqlitePool,
    content: &str,
    tags: Option<&str>,
    project: Option<&str>,
) -> DbResult<Note> {
    // 1. Normalize and hash content
    let normalized = normalize_content(content);
    let content_hash = sha256_hash(&normalized);
    
    // 2. Check for exact duplicate
    if let Some(existing) = get_note_by_hash(pool, &content_hash).await? {
        tracing::warn!("Duplicate note detected: {}", existing.id);
        return Err(DbError::Duplicate {
            existing_id: existing.id,
            message: "Identical note already exists".into(),
        });
    }
    
    // 3. Generate ID and timestamps
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    
    // 4. Insert note with hash
    sqlx::query(
        r#"
        INSERT INTO notes (
            id, content, tags, project, status, created_at, updated_at,
            content_hash, normalized_content
        )
        VALUES (?, ?, ?, ?, 'inbox', ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(content)
    .bind(tags)
    .bind(project)
    .bind(now)
    .bind(now)
    .bind(&content_hash)
    .bind(&normalized)
    .execute(pool)
    .await
    .map_err(|e| DbError::InsertFailed(e.to_string()))?;
    
    // 5. Fetch and return created note
    get_note(pool, &id).await
}
```

**Also add to DbError enum:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    // ... existing variants ...
    
    #[error("Duplicate content found: {existing_id} - {message}")]
    Duplicate {
        existing_id: String,
        message: String,
    },
}
```

---

## 🎨 Step 3: Add CLI Commands

Add new commands to `src/bin/cli.rs`:

```rust
// Add to Commands enum:
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...
    
    /// Ask a question (routes intelligently)
    Ask {
        /// Your question
        query: String,
        
        /// Current repository context
        #[arg(short, long)]
        repo: Option<String>,
        
        /// Current project context
        #[arg(short, long)]
        project: Option<String>,
    },
    
    /// View cost statistics
    Costs {
        #[command(subcommand)]
        action: CostsAction,
    },
}

#[derive(Subcommand)]
enum CostsAction {
    /// Show today's costs
    Today,
    
    /// Show this week's costs
    Week,
    
    /// Show this month's costs
    Month,
    
    /// Show budget status
    Budget,
    
    /// Generate daily report
    Report,
}

// Add handlers:

async fn handle_ask(
    pool: &SqlitePool,
    query: &str,
    repo: Option<String>,
    project: Option<String>,
) -> anyhow::Result<()> {
    use rustassistant::{QueryRouter, UserContext};
    
    println!("{} Processing query...\n", "🤖".dimmed());
    
    // Create router
    let mut router = QueryRouter::new(pool.clone(), "data/cache.db").await?;
    
    // Build user context
    let context = UserContext {
        current_repo: repo,
        current_project: project,
        ..Default::default()
    };
    
    // Route query
    let action = router.route(query, &context).await?;
    
    match action {
        Action::CachedResponse(response) => {
            println!("{} Using cached response\n", "⚡".green());
            println!("{}", response);
        }
        
        Action::DirectResponse(response) => {
            println!("{}", response);
        }
        
        Action::SearchDatabase(search_query) => {
            println!("{} Searching local database...\n", "🔍".blue());
            let notes = search_notes(pool, &search_query, 10).await?;
            
            if notes.is_empty() {
                println!("No results found.");
            } else {
                println!("Found {} notes:\n", notes.len());
                for note in notes {
                    print_note(&note);
                }
            }
        }
        
        Action::CallGrok(context) => {
            println!("{} Calling Grok API...", "🤖".yellow());
            println!("{} Context: {} files, {} tokens\n", 
                "ℹ".dimmed(), 
                context.files.len(),
                context.estimated_tokens()
            );
            
            // TODO: Actually call Grok API
            println!("⚠️  Grok integration not yet implemented");
            println!("Next: Implement GrokClient call with context");
        }
        
        Action::CallGrokMinimal(query) => {
            println!("{} Calling Grok API (minimal context)...\n", "🤖".yellow());
            
            // TODO: Call Grok with minimal context
            println!("⚠️  Grok integration not yet implemented");
        }
    }
    
    // Show routing stats
    let stats = router.get_stats();
    println!("\n{} Routing Stats:", "📊".dimmed());
    println!("  Cache hit rate: {:.1}%", router.cache_hit_rate());
    println!("  Cost avoidance: {:.1}%", router.cost_avoidance_rate());
    
    Ok(())
}

async fn handle_costs_action(pool: &SqlitePool, action: CostsAction) -> anyhow::Result<()> {
    use rustassistant::CostTracker;
    
    let tracker = CostTracker::new(pool.clone()).await?;
    
    match action {
        CostsAction::Today => {
            let stats = tracker.get_daily_stats().await?;
            println!("{} Today's Costs\n", "💰");
            print_cost_stats(&stats);
        }
        
        CostsAction::Week => {
            let stats = tracker.get_weekly_stats().await?;
            println!("{} This Week's Costs\n", "💰");
            print_cost_stats(&stats);
        }
        
        CostsAction::Month => {
            let stats = tracker.get_monthly_stats().await?;
            println!("{} This Month's Costs\n", "💰");
            print_cost_stats(&stats);
        }
        
        CostsAction::Budget => {
            let status = tracker.get_budget_status().await?;
            println!("{} Budget Status\n", "📊");
            print_budget_status(&status);
        }
        
        CostsAction::Report => {
            let report = tracker.daily_report().await?;
            println!("{}", report);
        }
    }
    
    Ok(())
}

fn print_cost_stats(stats: &CostStats) {
    println!("  {} {}", "Total Queries:".dimmed(), stats.total_queries);
    println!("  {} ${:.4}", "Total Cost:".dimmed(), stats.total_cost_usd);
    println!("  {} {:.1}%", "Cache Hit Rate:".dimmed(), stats.cache_hit_rate);
    println!("  {} ${:.4}", "Saved (Cache):".dimmed(), stats.cost_saved_from_cache);
    println!("\n  {} Tokens:", "📊".dimmed());
    println!("    Input:  {}", format_tokens(stats.total_input_tokens));
    println!("    Output: {}", format_tokens(stats.total_output_tokens));
    println!("    Cached: {}", format_tokens(stats.total_cached_tokens));
}

fn print_budget_status(status: &BudgetStatus) {
    println!("  {} Daily Budget:", "📅".dimmed());
    println!("    Spent:     ${:.4} / ${:.2}", status.daily_spend, status.daily_budget);
    println!("    Remaining: ${:.4}", status.daily_remaining);
    println!("    Used:      {:.1}%", status.daily_percent_used);
    
    println!("\n  {} Monthly Budget:", "📆".dimmed());
    println!("    Spent:     ${:.4} / ${:.2}", status.monthly_spend, status.monthly_budget);
    println!("    Remaining: ${:.4}", status.monthly_remaining);
    println!("    Used:      {:.1}%", status.monthly_percent_used);
    
    if !status.alerts.is_empty() {
        println!("\n  {} Alerts:", "⚠️");
        for alert in &status.alerts {
            println!("    {}", alert);
        }
    }
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.2}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.2}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}
```

---

## 🧪 Step 4: Test It!

### Test Deduplication

```bash
# Create a note
cargo run --bin rustassistant -- note add "Testing deduplication" --tags test

# Try to create the same note (should fail)
cargo run --bin rustassistant -- note add "Testing deduplication" --tags test
# Expected: Error: Duplicate content found
```

### Test Query Router

```bash
# Greeting (should not call Grok)
cargo run --bin rustassistant -- ask "hi"

# Note search (database only)
cargo run --bin rustassistant -- ask "find my notes about testing"

# Task generation (would call Grok)
cargo run --bin rustassistant -- ask "what should I work on next?"
```

### Test Cost Tracking

```bash
# View today's costs
cargo run --bin rustassistant -- costs today

# View budget status
cargo run --bin rustassistant -- costs budget

# Generate report
cargo run --bin rustassistant -- costs report
```

---

## 📊 Expected Results

After implementing Phase 1, you should see:

- **60-80% cost reduction** from intelligent routing
- **No duplicate notes** in your database
- **Clear visibility** into API spending
- **Fast responses** for greetings and searches (no API calls)

### Cost Breakdown Example

```
Before Phase 1:
- 20 queries/day × $0.03/query = $0.60/day = $18/month

After Phase 1:
- 20 queries/day:
  - 4 greetings (free)
  - 8 note searches (free)
  - 4 cached responses (free)
  - 4 Grok API calls × $0.03 = $0.12/day
- Total: $3.60/month

Savings: $14.40/month (80% reduction)
```

---

## 🐛 Troubleshooting

### Issue: "Failed to initialize response cache"

**Solution:** Create the cache database:
```bash
mkdir -p data
touch data/cache.db
```

### Issue: "content_hash column doesn't exist"

**Solution:** Run the migration:
```bash
sqlite3 data/rustassistant.db < migrations/003_phase1_features.sql
```

### Issue: Compilation errors in query_router.rs

**Solution:** The tests use `todo!()` placeholders. Either:
1. Comment out the test module, or
2. Create mock implementations

---

## 🎯 Next Steps

Once Phase 1 is working:

1. **Week 6:** Implement actual Grok API integration
2. **Week 7:** Add semantic caching (similar query detection)
3. **Week 8:** Evaluate if you need full RAG or if context stuffing is sufficient

---

## 📚 Reference

### Key Files
- `src/query_router.rs` - Intent classification and routing
- `src/cost_tracker.rs` - Cost monitoring and budgets
- `src/response_cache.rs` - Response caching
- `src/context_builder.rs` - Context assembly
- `src/db.rs` - Database operations (update for deduplication)

### Metrics to Track
- Cache hit rate (target: >60%)
- Cost avoidance rate (target: >70%)
- Daily spend (target: <$0.20)
- Query latency (target: <2s cached, <10s uncached)

---

**Questions?** See `IMPLEMENTATION_ROADMAP.md` for the full picture.

**Ready?** Start with the database migration and work your way through the steps!

🚀 Happy coding!