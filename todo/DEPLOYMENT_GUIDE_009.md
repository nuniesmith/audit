# Deployment Guide: Queue Table + Scanner Rescan Fixes
## Date: 2026-02-07

Three issues to fix in this deployment:

1. **`queue_items` table missing** — web UI `/queue` page errors
2. **Scanner won't rescan** — stored commit hash blocks re-analysis  
3. **Force-scan doesn't clear commit hash** — only resets timing, not the diff state

---

## Step 1: Add Migration 009

Copy `009_queue_items.sql` to `migrations/` directory:

```bash
cp 009_queue_items.sql migrations/009_queue_items.sql
```

This creates the `queue_items`, `file_analysis`, `todo_items`, and `repo_cache` tables
that are referenced by `src/db/queue.rs` and `src/web_ui.rs` but were never in the
migration sequence (they were only in `create_queue_tables()` which isn't called at
startup when using migration-based initialization).

---

## Step 2: Fix force_scan to Also Clear Commit Hash

The current `force_scan()` in `src/auto_scanner.rs` only resets `last_scanned_at`,
which controls *when* the scanner runs. But "No changes detected" comes from
comparing `last_commit_hash` to current HEAD. If they match, it skips.

### In `src/auto_scanner.rs`, find:

```rust
/// Force a scan check for a repository (reset last_scan_check)
pub async fn force_scan(pool: &sqlx::SqlitePool, repo_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE repositories
        SET last_scanned_at = NULL
        WHERE id = ?
        "#,
    )
    .bind(repo_id)
    .execute(pool)
    .await?;

    info!("Forced scan check for repo {}", repo_id);

    Ok(())
}
```

### Replace with:

```rust
/// Force a full rescan for a repository (reset both timing AND commit hash)
pub async fn force_scan(pool: &sqlx::SqlitePool, repo_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE repositories
        SET last_scanned_at = NULL,
            last_commit_hash = NULL
        WHERE id = ?
        "#,
    )
    .bind(repo_id)
    .execute(pool)
    .await?;

    info!("Forced full rescan for repo {} (cleared commit hash + scan time)", repo_id);

    Ok(())
}
```

### Also fix the web UI force_scan_handler in `src/web_ui.rs`, find:

```rust
match sqlx::query("UPDATE repositories SET last_scanned_at = NULL WHERE id = ?")
```

### Replace with:

```rust
match sqlx::query("UPDATE repositories SET last_scanned_at = NULL, last_commit_hash = NULL WHERE id = ?")
```

---

## Step 3: Quick Fix — Clear Commit Hash Now (Before Rebuild)

To immediately unblock the scanner without waiting for a rebuild, you can run
this SQL directly against the running container's database:

```bash
# Option A: docker exec into container
docker exec -it rustassistant sqlite3 /app/data/rustassistant.db \
  "UPDATE repositories SET last_commit_hash = NULL; SELECT id, name, last_commit_hash FROM repositories;"

# Option B: from Pi directly if the db is volume-mounted
sqlite3 ./data/rustassistant.db \
  "UPDATE repositories SET last_commit_hash = NULL;"
```

After clearing the hash, the scanner will see `last_commit_hash = NULL` on the next
cycle (within 60 seconds), treat ALL files as changed, and start analyzing them.
Cache hits on the 22 already-analyzed files will be free; the remaining ~914 will
be new API calls.

---

## Step 4: Build and Deploy

```bash
cd ~/github/rustassistant

# Rebuild
docker compose build --no-cache

# Restart
docker compose up -d

# Watch logs
docker logs -f rustassistant
```

### What you should see:

```
INFO rustassistant::db::config: Running database migrations...
INFO rustassistant::db::config: Applied migration 009_queue_items    <-- NEW
INFO rustassistant::db::config: Migrations complete
...
INFO rustassistant::auto_scanner: Scanning repository: fks
INFO rustassistant::auto_scanner: Found 936 changed files in fks     <-- RESCAN!
INFO rustassistant::auto_scanner: Analyzing src/some_file.rs
INFO rustassistant::auto_scanner: Cache hit for src/already_cached.rs  <-- FREE
...
```

### Verify queue page works:

```
http://localhost:3001/queue    <-- should load without error
```

---

## Step 5: Verify Scanner Is Working

After a few minutes, check the logs:

```bash
docker logs rustassistant 2>&1 | grep -E "(Analyzing|Cache hit|cost|budget)"
```

Expected pattern:
- ~22 files show "Cache hit" (already analyzed in previous run)
- Remaining ~914 files show "Analyzing" with new API calls
- Cost accumulates at ~$0.0025/file
- Total should be ~$2.34 for all 936 files

---

## Summary of All Files to Change

| File | Change |
|------|--------|
| `migrations/009_queue_items.sql` | **NEW** — queue_items + related tables |
| `src/auto_scanner.rs` | Fix `force_scan()` to clear commit hash |
| `src/web_ui.rs` | Fix force_scan SQL to clear commit hash |

### Also apply from previous patch (if not already):
| File | Change |
|------|--------|
| `src/auto_scanner.rs` | Bump `DEFAULT_SCAN_COST_BUDGET` to `$3.00` |
| `src/auto_scanner.rs` | `analyze_file()` returns `(i64, f64)` with actual cost |
| `src/auto_scanner.rs` | `analyze_changed_files_with_progress()` uses actual cost + returns `budget_halted` |
| `src/auto_scanner.rs` | Only store commit hash when `!budget_halted` |
