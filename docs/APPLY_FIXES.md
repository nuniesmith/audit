# Quick Reference: Apply Data Layer Fixes

**Last Updated**: 2024-02-07  
**Status**: Ready to apply

---

## 🎯 Quick Start (2 Minutes)

```bash
# 1. Apply migration 007
export DATABASE_URL=sqlite:./data/rustassistant.db
sqlx migrate run

# 2. Verify tables exist
sqlite3 data/rustassistant.db ".schema ideas"
sqlite3 data/rustassistant.db ".schema documents_fts"

# 3. Build and run
cargo build --release
./target/release/server
```

Server starts on `http://localhost:3000`

---

## ✅ What Was Fixed

| Issue | Impact | Fix |
|-------|--------|-----|
| `list_ideas` parameter binding bug | Runtime panic when filtering | Dynamic param numbering |
| Tag struct schema mismatch | 500 error on `/api/tags` | Aligned with migration 005 |
| Missing ideas table | 500 error on `/ideas` page | Migration 007 created |
| Missing documents_fts table | 500 error on FTS search | Included in migration 007 |
| Document ID type mismatches | Type errors in API handlers | Changed i64 to String (UUIDs) |
| Tenant created_at type mismatch | Type error in multi_tenant.rs | Convert i64 to DateTime<Utc> |

---

## 📋 Verification Steps

### 1. Check Migration Applied

```bash
sqlite3 data/rustassistant.db "SELECT version FROM _sqlx_migrations ORDER BY version;"
```

**Expected**: Should include `20240207000000` (migration 007)

### 2. Test Ideas API

```bash
# Create idea
curl -X POST http://localhost:3000/ideas \
  -d "content=Test filtering&category=bug&priority=1"

# Test filtering (this would panic before fix)
curl "http://localhost:3000/api/ideas?category=bug&status=inbox"
curl "http://localhost:3000/api/ideas?tag=urgent"
```

**Expected**: No panic, correct filtering results

### 3. Test Tags API

```bash
curl http://localhost:3000/api/tags?limit=10
```

**Expected**: JSON with tags (no `id` field, includes `description` and `updated_at`)

```json
{
  "tags": [
    {
      "name": "idea",
      "color": "#10b981",
      "description": "New ideas and brainstorming",
      "usage_count": 5,
      "created_at": 1234567890,
      "updated_at": 1234567890
    }
  ]
}
```

### 4. Test FTS Search

```bash
curl "http://localhost:3000/api/docs/search?q=welcome"
```

**Expected**: Returns matching documents using FTS5 index

### 5. Test UI Pages

- `http://localhost:3000/ideas` - Should load without errors
- `http://localhost:3000/docs` - Should load with search working
- Filter ideas by status/category/tags - Should work correctly

---

## 🔧 Troubleshooting

### "no such table: ideas"

**Cause**: Migration 007 not applied

**Fix**:
```bash
sqlx migrate run
```

### "unable to open database file" during build

**Cause**: DATABASE_URL not set

**Fix**:
```bash
export DATABASE_URL=sqlite:./data/rustassistant.db
sqlx database create
sqlx migrate run
```

### "column 'id' not found" when querying tags

**Cause**: Old code not using fixed Tag struct

**Fix**: Make sure you have the latest `src/db/documents.rs`

### Filtering ideas returns wrong results

**Cause**: Old parameter binding code

**Fix**: Verify `list_ideas` function uses dynamic numbering pattern

---

## 📚 Documentation

- **`docs/DATA_LAYER_FIXES.md`** - Detailed explanation of all fixes
- **`docs/COMPILE_TROUBLESHOOTING.md`** - Build issues and solutions
- **`docs/REVIEW_PASS_FIXES.md`** - Executive summary
- **`docs/INTEGRATION_FIXES_COMPLETED.md`** - Previous integration work

---

## 🧪 Test Scenarios

### Scenario 1: Mixed Filtering

```bash
# Should work with any combination of filters
curl "http://localhost:3000/api/ideas?status=inbox"
curl "http://localhost:3000/api/ideas?category=feature"
curl "http://localhost:3000/api/ideas?status=inbox&category=feature"
curl "http://localhost:3000/api/ideas?tag=urgent&project=rustassistant"
```

**Before fix**: Would panic or return wrong results  
**After fix**: Correctly filters ideas

### Scenario 2: Tag Management

```bash
# List all tags
curl http://localhost:3000/api/tags?limit=50

# Search tags
curl "http://localhost:3000/api/tags/search?q=bug"
```

**Before fix**: 500 error (id column missing)  
**After fix**: Returns tags with correct schema

### Scenario 3: Document Search

```bash
# Full-text search
curl "http://localhost:3000/api/docs/search?q=rust"
curl "http://localhost:3000/api/docs/search?q=welcome+system"
```

**Before fix**: 500 error (FTS table missing)  
**After fix**: Returns matching documents

---

## 🎨 UI Testing

1. **Ideas Page** (`/ideas`)
   - [ ] Page loads without errors
   - [ ] Can create new idea
   - [ ] Can filter by status (inbox, active, done)
   - [ ] Can filter by category
   - [ ] Can click tags to filter
   - [ ] Status changes work
   - [ ] Delete works

2. **Documents Page** (`/docs`)
   - [ ] Page loads without errors
   - [ ] Can create new document
   - [ ] Search box works (FTS)
   - [ ] Can filter by doc type
   - [ ] Can view/edit documents
   - [ ] Tags display correctly

3. **Activity Page** (`/activity`)
   - [ ] Shows scan events
   - [ ] Events include ideas/docs activity
   - [ ] Filtering works

---

## 💡 Key Changes Summary

### `src/db/documents.rs`

**Tag struct** (lines 704-713):
```rust
pub struct Tag {
    pub name: String,              // PRIMARY KEY (no id!)
    pub color: String,             // NOT NULL
    pub description: Option<String>,
    pub usage_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}
```

**list_ideas** (lines 750-800):
```rust
let mut binds: Vec<String> = Vec::new();
if let Some(s) = status {
    binds.push(s.to_string());
    query.push_str(&format!(" AND status = ?{}", binds.len()));
}
// ... dynamic numbering for all filters
```

### `migrations/007_ideas.sql`

- Ideas table with indexes
- documents_fts virtual table
- FTS sync triggers
- Helper views (active_ideas, ideas_by_category, recent_ideas_activity)

---

## ✨ Success Criteria

- [x] All 6 critical issues fixed (4 data-layer + 2 type mismatches)
- [x] Migration 007 created
- [x] Documentation complete
- [x] Document ID types fixed (i64 → String)
- [x] Tenant created_at conversion fixed
- [ ] Migration 007 applied to database
- [ ] Server builds without errors (only SQLx verification errors remain)
- [ ] All test scenarios pass
- [ ] UI pages work correctly

---

## 🚀 Ready to Deploy

Once verification passes:

1. Commit the changes:
   ```bash
   git add migrations/007_ideas.sql
   git add src/db/documents.rs
   git add src/api/types.rs
   git add src/api/jobs.rs
   git add src/multi_tenant.rs
   git add docs/*.md
   git add APPLY_FIXES.md
   git commit -m "Fix data layer issues: parameter binding, tag schema, ideas migration, document ID types"
   ```

2. Deploy/restart production:
   ```bash
   sqlx migrate run
   systemctl restart rustassistant
   ```

3. Verify production:
   - Check logs for errors
   - Test ideas/docs pages
   - Monitor for panics

---

**Status**: ✅ All fixes applied and ready for testing