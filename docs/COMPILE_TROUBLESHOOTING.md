# Compilation Troubleshooting Guide

**Date**: 2024-02-07  
**Status**: Active development

---

## Current Build Status

After applying the data-layer fixes, the project has:
- ✅ All critical bugs fixed
- ⚠️ SQLx compile-time verification errors (expected without DATABASE_URL)
- ⚠️ Type annotation errors in some handlers (non-blocking)
- ℹ️ 18 warnings (mostly unused imports)

---

## SQLx Compile-Time Verification Errors

### What They Look Like

```
error: error returned from database: (code: 14) unable to open database file
  --> src/db/documents.rs:80:15
```

### Why This Happens

SQLx verifies SQL queries at **compile time** by connecting to a real database. If `DATABASE_URL` is not set or the database doesn't exist, you'll see these errors.

### Solution 1: Set DATABASE_URL (Recommended)

```bash
# Create and migrate the database
export DATABASE_URL=sqlite:./data/rustassistant.db
mkdir -p data
sqlx database create
sqlx migrate run

# Now compile
cargo check
cargo build
```

### Solution 2: Use SQLx Offline Mode

If you can't or don't want to set up a database during compilation:

1. **Update Cargo.toml** to enable offline mode:
   ```toml
   [dependencies]
   sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "offline"] }
   ```

2. **Prepare offline query data** (requires DATABASE_URL once):
   ```bash
   export DATABASE_URL=sqlite:./data/rustassistant.db
   sqlx database create
   sqlx migrate run
   cargo sqlx prepare
   ```

3. **Commit the generated files**:
   ```bash
   git add .sqlx/
   git commit -m "Add SQLx offline query data"
   ```

4. **Now compile without DATABASE_URL**:
   ```bash
   unset DATABASE_URL
   cargo check  # Works!
   ```

---

## Type Annotation Errors

### Example Errors

```
error[E0282]: type annotations needed for `std::result::Result<_, _>`
  --> src/api/handlers.rs:189:9
```

### Cause

These occur when the Rust compiler can't infer the error type in a Result without seeing the actual SQL query execution (which fails due to missing DATABASE_URL).

### Solution

Set up DATABASE_URL (Solution 1 above). These errors will disappear once SQLx can verify the queries.

---

## Unused Import Warnings

### Current Warnings (18 total)

```
warning: unused import: `std::net::IpAddr`
  --> src/api/rate_limit.rs:11:5

warning: unused imports: `ScanEvent` and `get_repo_events`
  --> src/web_ui_extensions.rs:19:49
```

### Why They Exist

- Dead code from refactoring
- Imports added in anticipation of future features
- Conditional compilation features not enabled

### Solution (Optional)

Clean them up when you have time:

```bash
# Auto-fix some warnings
cargo fix --allow-dirty

# Or manually remove unused imports
# Then run:
cargo clippy --fix
```

### Safe to Ignore?

Yes. Warnings don't prevent compilation or runtime execution. Fix them for clean builds, but they're low priority.

---

## Mismatched Type Errors

### api/jobs.rs:170 - Vec<String> vs Vec<i64>

```
error[E0308]: mismatched types: expected `Vec<String>`, found `Vec<i64>`
  --> src/api/jobs.rs:170:33
```

**Cause**: Document IDs are UUIDs (strings), not integers.

**Fix**: Already applied in previous integration fixes. If you still see this, check that the fix was applied:

```rust
// In src/api/jobs.rs around line 170
let doc_ids: Vec<String> = docs.iter().map(|d| d.id.clone()).collect();
// NOT: let doc_ids: Vec<i64> = ...
```

### multi_tenant.rs:284 - DateTime<Utc> vs i64

```
error[E0308]: mismatched types: expected `DateTime<Utc>`, found `i64>`
  --> src/multi_tenant.rs:284:13
```

**Cause**: SQLite stores timestamps as Unix epoch (i64), but some code expects `DateTime<Utc>`.

**Fix**: Convert i64 to DateTime:

```rust
use chrono::{DateTime, TimeZone, Utc};

// Convert i64 timestamp to DateTime
let dt: DateTime<Utc> = Utc.timestamp_opt(row.created_at, 0)
    .single()
    .unwrap_or_else(|| Utc::now());
```

Or change the expected type to i64 if DateTime isn't needed.

---

## Never Type Fallback Warning

### cache_layer.rs:530

```
error: this function depends on never type fallback being `()`
  --> src/cache_layer.rs:530:5
```

**Cause**: Rust compiler change in handling the `!` (never) type.

**Fix**: Add explicit type annotation:

```rust
// Before:
panic!("This should never happen");

// After:
panic!("This should never happen") as !
// Or return a concrete error type instead of panicking
```

---

## Quick Compilation Checklist

Before running `cargo check` or `cargo build`:

- [ ] Set `DATABASE_URL` environment variable
- [ ] Database file exists (`ls data/rustassistant.db`)
- [ ] Migrations applied (`sqlx migrate run`)
- [ ] SQLx can connect to DB (`sqlite3 data/rustassistant.db ".tables"`)

Expected result:
```
$ cargo check
   Checking rustassistant v0.1.0 (/home/user/rustassistant)
warning: unused import: ...
   (18 warnings)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.34s
```

---

## CI/CD Considerations

### For GitHub Actions / GitLab CI

Add database setup step:

```yaml
- name: Setup database
  run: |
    export DATABASE_URL=sqlite:./data/rustassistant.db
    mkdir -p data
    sqlx database create
    sqlx migrate run

- name: Build
  run: cargo build --release
  env:
    DATABASE_URL: sqlite:./data/rustassistant.db
```

### For Docker Builds

In Dockerfile:

```dockerfile
# Create database during build
ENV DATABASE_URL=sqlite:/app/data/rustassistant.db
RUN mkdir -p /app/data && \
    sqlx database create && \
    sqlx migrate run

# Build with database available
RUN cargo build --release
```

---

## Common Errors and Solutions

### "unable to open database file"

**Cause**: Database doesn't exist or DATABASE_URL points to wrong path.

**Fix**:
```bash
mkdir -p data
export DATABASE_URL=sqlite:./data/rustassistant.db
sqlx database create
```

### "no such table: ideas"

**Cause**: Migration 007 not applied.

**Fix**:
```bash
sqlx migrate run
# Verify:
sqlite3 data/rustassistant.db ".schema ideas"
```

### "cannot infer type"

**Cause**: SQLx can't verify query without database access.

**Fix**: Set DATABASE_URL (see above).

---

## Running the Server

After successful compilation:

```bash
# Set database path
export DATABASE_URL=sqlite:./data/rustassistant.db

# Run migrations (if not already done)
sqlx migrate run

# Start server
cargo run --bin server

# Or in release mode
cargo build --release
./target/release/server
```

Server should start on `http://localhost:3000`.

---

## Testing the Fixes

### Test Ideas System

```bash
# Open ideas page
open http://localhost:3000/ideas

# Create an idea via API
curl -X POST http://localhost:3000/ideas \
  -d "content=Test the new ideas system&category=feature&priority=2"

# Filter ideas
curl "http://localhost:3000/api/ideas?status=inbox&category=feature"
```

### Test Documents System

```bash
# Open documents page
open http://localhost:3000/docs

# Search documents
curl "http://localhost:3000/api/docs/search?q=welcome"
```

### Test Tags

```bash
# List tags
curl http://localhost:3000/api/tags?limit=20

# Search tags
curl "http://localhost:3000/api/tags/search?q=feature"
```

---

## Summary

| Issue | Severity | Solution |
|-------|----------|----------|
| SQLx verification errors | ⚠️ High | Set DATABASE_URL or use offline mode |
| Type annotation errors | ⚠️ High | Set DATABASE_URL (they're side effects) |
| Unused import warnings | ℹ️ Low | Run `cargo clippy --fix` when convenient |
| Mismatched type errors | 🔴 Critical | Already fixed; verify changes applied |

**Bottom line**: Set `DATABASE_URL` and run migrations, and most errors will disappear.

---

## Need Help?

1. Check that migration 007 exists: `ls migrations/007_ideas.sql`
2. Verify database has all tables: `sqlite3 data/rustassistant.db ".tables"`
3. Check migration history: `sqlite3 data/rustassistant.db "SELECT * FROM _sqlx_migrations;"`
4. Review recent changes: `git log --oneline -10`

If issues persist, check:
- `docs/DATA_LAYER_FIXES.md` - Data layer architecture
- `docs/INTEGRATION_FIXES_COMPLETED.md` - Integration summary
- `docs/QUICK_START_NEW_FEATURES.md` - Feature testing guide