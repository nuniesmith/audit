# Cache Viewer Integration Guide

## Files

- **`src/web_ui_cache_viewer.rs`** — New module (the file you just received)

## Integration Steps

### 1. Add the module to `src/lib.rs`

```rust
pub mod web_ui_cache_viewer;
```

### 2. Make `RepoCacheSql.pool` public

In `src/repo_cache_sql.rs`, change:

```rust
// FROM:
pub struct RepoCacheSql {
    pool: SqlitePool,
    // ...
}

// TO:
pub struct RepoCacheSql {
    pub pool: SqlitePool,
    // ...
}
```

The cache viewer needs direct SQL access to the per-repo cache DB for flexible queries.

### 3. Merge routes in `src/bin/server.rs`

Where you build the router (after creating `WebAppState`):

```rust
use rustassistant::web_ui_cache_viewer::create_cache_viewer_router;

// Existing router setup
let web_router = create_router(state.clone());

// Add cache viewer routes
let app = web_router
    .merge(create_cache_viewer_router(Arc::new(state.clone())))
    // ... other merges
    ;
```

### 4. Update navigation in existing pages

Add "Cache Viewer" to the nav in `web_ui.rs` and `web_ui_extensions.rs`:

```html
<a href="/cache">Cache Viewer</a>
```

(The cache viewer module already includes this in its own nav helper.)

### 5. Fix the commit hash resume issue

In `src/auto_scanner.rs`, find where you call `update_last_commit_hash` after a scan completes. Change it so the hash is only stored when ALL files are analyzed (no budget cutoff):

```rust
// After analyze_changed_files_with_progress returns:
if files_skipped_budget == 0 {
    // All files analyzed — safe to store the hash
    if let Some(ref hash) = current_head {
        self.update_last_commit_hash(repo_id, hash).await?;
    }
} else {
    info!("Skipping commit hash update — {} files still pending from budget cap", files_skipped_budget);
}
```

This ensures the next scan cycle will re-diff, hit cache on the already-analyzed files (free), and continue analyzing the remaining files until budget is reached again. The cache naturally builds out over multiple cycles.

## Pages

| Route | What it shows |
|-------|--------------|
| `/cache` | Overview: all repos, stats, costs, cache hit rates |
| `/cache/:id` | File list for a repo: path, score, tokens, size, sort/filter |
| `/cache/:id/file?path=...` | Single file: full analysis JSON, metadata, score |
| `/cache/:id/gaps` | Coverage gaps: source files vs analyzed, grouped by directory |

## Dependencies

No new crate dependencies. Uses existing `sqlx`, `zstd`, `serde_json`, `chrono`, `axum`, and `tracing`.
