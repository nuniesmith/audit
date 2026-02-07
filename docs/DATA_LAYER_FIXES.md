# Data Layer Fixes - Complete Summary

**Date**: 2024-02-07  
**Status**: ✅ All critical data-layer issues resolved

---

## Overview

This document summarizes the fixes applied to resolve critical data-layer mismatches between database schemas, migrations, and Rust structs that were blocking compilation and would have caused runtime failures.

---

## Issues Fixed

### 🔴 CRITICAL: `list_ideas` Parameter Binding Bug

**Problem**: The `list_ideas` function in `src/db/documents.rs` used hardcoded numbered parameters (`?1`, `?2`, `?3`, `?4`, `?5`) but bound them conditionally. When filters were `None`, SQLite's numbered parameter binding would mismatch.

**Example of the bug**:
```rust
// WRONG: If status=None and category=Some("bug"), the query contains
// "AND category = ?2" but category is bound as the FIRST parameter
if status.is_some() {
    query.push_str(" AND status = ?1");
}
if category.is_some() {
    query.push_str(" AND category = ?2");  // ❌ Wrong parameter number!
}
```

**Impact**: Runtime panic or incorrect query results when filtering ideas by status/category/tag/project.

**Fix**: Implemented dynamic parameter numbering pattern (as used in `todo/documents.rs`):
```rust
let mut binds: Vec<String> = Vec::new();

if let Some(s) = status {
    binds.push(s.to_string());
    query.push_str(&format!(" AND status = ?{}", binds.len()));  // ✅ Correct!
}
if let Some(c) = category {
    binds.push(c.to_string());
    query.push_str(&format!(" AND category = ?{}", binds.len())); // ✅ Correct!
}
// ... etc
```

**Files Changed**:
- `src/db/documents.rs` (lines 750-800)

---

### 🔴 CRITICAL: Tags Table Schema Mismatch

**Problem**: Two conflicting `Tag` struct definitions existed:

1. **Migration 005 schema** (`migrations/005_notes_enhancements.sql`):
   ```sql
   CREATE TABLE tags (
       name TEXT PRIMARY KEY,
       color TEXT DEFAULT '#3b82f6',
       description TEXT,
       usage_count INTEGER DEFAULT 0,
       created_at INTEGER,
       updated_at INTEGER
   );
   ```

2. **Ideas Tag struct** (`src/db/documents.rs`):
   ```rust
   pub struct Tag {
       pub id: i64,           // ❌ Column doesn't exist!
       pub name: String,
       pub color: Option<String>,
       pub usage_count: i64,
       pub created_at: i64,
   }
   ```

3. **Core Tag struct** (`src/db/core.rs`):
   ```rust
   pub struct Tag {
       pub name: String,
       pub color: String,
       pub description: Option<String>,
       pub usage_count: i64,
       pub created_at: i64,
       pub updated_at: i64,
   }
   ```

**Impact**: Runtime failure when querying `/api/tags` or `/ideas` page because sqlx cannot map the non-existent `id` column.

**Fix**: Aligned the `documents.rs` Tag struct with the production schema from migration 005:
```rust
pub struct Tag {
    pub name: String,              // PRIMARY KEY
    pub color: String,             // NOT NULL with default
    pub description: Option<String>,
    pub usage_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}
```

Updated SQL queries to select correct columns:
```rust
SELECT name, color, description, usage_count, created_at, updated_at
FROM tags
-- Instead of: SELECT id, name, color, ...
```

**Files Changed**:
- `src/db/documents.rs` (Tag struct and list_tags/search_tags functions)

---

### 🟡 MEDIUM: Missing Ideas Table Migration

**Problem**: The `ideas` table was defined in `todo/003_scan_docs_ideas.sql` but never migrated to production. The actual migration sequence was:
- ✅ 001: simplified_tasks
- ✅ 002: github_integration
- ✅ 003: scan_progress (different schema!)
- ✅ 004: require_git_url
- ✅ 005: notes_enhancements
- ✅ 006: documents
- ❌ **Missing**: ideas table

**Impact**: 500 errors on `/ideas` page with "no such table: ideas" when attempting to query or insert ideas.

**Fix**: Created **migration 007** (`migrations/007_ideas.sql`) containing:
- Ideas table schema
- Indexes for efficient querying (status, priority, category, tags, project)
- Views for common queries (active_ideas, ideas_by_category, recent_ideas_activity)
- Triggers for automatic timestamp updates
- Sample welcome idea

**Files Created**:
- `migrations/007_ideas.sql`

---

### 🟡 MEDIUM: Missing `documents_fts` Virtual Table

**Problem**: The `documents_fts` FTS5 virtual table for full-text search was defined in `todo/003_scan_docs_ideas.sql` but may not exist in production if that migration was never applied.

**Impact**: 500 errors when attempting FTS search in `/docs` page with "no such table: documents_fts".

**Fix**: Included in migration 007:
```sql
CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
    title,
    content,
    tags,
    content='documents',
    content_rowid='rowid'
);
```

Plus triggers to keep FTS index synchronized with documents table:
- `documents_ai`: After INSERT
- `documents_ad`: After DELETE
- `documents_au`: After UPDATE

**Files Changed**:
- `migrations/007_ideas.sql`

---

## Migration Path

### For Fresh Installations

Just run migrations in order:
```bash
export DATABASE_URL=sqlite:./data/rustassistant.db
sqlx database create
sqlx migrate run
```

All migrations 001-007 will apply cleanly.

---

### For Existing Installations

If you already have migrations 001-006 applied:

1. **Check current migration status**:
   ```bash
   sqlite3 data/rustassistant.db "SELECT * FROM _sqlx_migrations ORDER BY version;"
   ```

2. **Apply migration 007**:
   ```bash
   sqlx migrate run
   ```

3. **Verify ideas table exists**:
   ```bash
   sqlite3 data/rustassistant.db ".schema ideas"
   ```

4. **Verify FTS table exists**:
   ```bash
   sqlite3 data/rustassistant.db ".schema documents_fts"
   ```

---

## Verification Checklist

After applying fixes and migrations:

- [ ] `cargo check` completes (ignoring SQLx verification errors if DATABASE_URL not set)
- [ ] Database has `ideas` table with all columns
- [ ] Database has `documents_fts` virtual table
- [ ] `tags` table has `name` as PRIMARY KEY (no `id` column)
- [ ] `/ideas` page loads without errors
- [ ] `/docs` page FTS search works
- [ ] Filtering ideas by status/category/tag works correctly
- [ ] Tags display correctly on ideas and documents pages

---

## Testing Commands

```bash
# Test ideas CRUD
curl -X POST http://localhost:3000/ideas \
  -d "content=Test idea&category=feature&priority=2"

# Test ideas filtering
curl "http://localhost:3000/api/ideas?status=inbox&category=feature"

# Test tags query
curl http://localhost:3000/api/tags?limit=10

# Test documents FTS search
curl "http://localhost:3000/api/docs/search?q=rust"
```

---

## Related Files

### Modified
- `src/db/documents.rs` - Fixed Tag struct and list_ideas parameter binding
- `migrations/007_ideas.sql` - New migration for ideas and FTS

### Referenced
- `migrations/005_notes_enhancements.sql` - Source of truth for tags schema
- `migrations/006_documents.sql` - Documents table schema
- `todo/003_scan_docs_ideas.sql` - Original ideas/FTS definitions (not applied)
- `src/db/core.rs` - Canonical Tag struct definition (matched)

---

## Breaking Changes

None. All fixes maintain backward compatibility:
- Tag struct changes align with existing migration 005 schema
- Migration 007 uses `IF NOT EXISTS` for safety
- Existing data is preserved

---

## Future Improvements

1. **SQLx Offline Mode**: Add `offline` feature to avoid needing DATABASE_URL at compile time
   ```toml
   sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "offline"] }
   ```

2. **Migration Tests**: Add integration tests that verify schema matches struct definitions

3. **Schema Validation**: Add build-time check to ensure Rust structs match DB schema

4. **Consolidate Tag Definitions**: Single source of truth for Tag struct (currently in core.rs and documents.rs)

---

## Summary

All critical data-layer issues are now resolved:
- ✅ Parameter binding bug in `list_ideas` fixed with dynamic numbering
- ✅ Tag struct aligned with production schema (name as PK, added description/updated_at)
- ✅ Ideas table migration created (007_ideas.sql)
- ✅ documents_fts virtual table included in migration 007

The system is now ready for runtime testing. Apply migration 007, restart the server, and test the `/ideas` and `/docs` pages.