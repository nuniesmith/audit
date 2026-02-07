# Quick Start Guide - New Features

This guide helps you test the newly integrated Ideas, Docs, and Activity Feed features.

## 1. Setup

### Database Migration

If you haven't run migrations yet:

```bash
# Set database location
export DATABASE_URL=sqlite:./data/rustassistant.db

# Create database and run migrations
sqlx database create
sqlx migrate run
```

### Run the Server

```bash
cargo run --bin server
```

The server will start on `http://localhost:3000` by default.

## 2. Feature Overview

### 📝 Ideas - Quick Thought Capture

**What it does:** Lightweight note-taking for fleeting thoughts, TODOs, and research ideas.

**URL:** `http://localhost:3000/ideas`

**Try it:**
1. Enter a quick thought in the text area
2. Add tags (comma-separated): `rust, performance, research`
3. Set priority (1-5)
4. Choose category: feature, bug, improvement, research, question, random
5. Click "Capture Idea"

**Filtering:**
- Click tag badges to filter by tag
- Use query params: `?status=inbox&category=research&tag=rust`

**Actions:**
- Mark as active/done/archived via status links
- Delete ideas you no longer need

---

### 📚 Docs - Knowledge Base

**What it does:** Store research notes, tutorials, architecture docs, code snippets.

**URL:** `http://localhost:3000/docs`

**Try it:**
1. Click "+ New Document"
2. Enter title and content (supports Markdown)
3. Choose doc type: research, reference, tutorial, architecture, decision, snippet
4. Add tags for categorization
5. Optionally add source URL if imported from web

**Search:**
- Uses FTS5 full-text search
- Search in title, content, and tags
- Fast and efficient for large knowledge bases

**Filter:**
- Click doc type badges to filter by type
- Search bar for full-text queries

---

### 📊 Activity Feed - Real-Time Scanner Events

**What it does:** Shows live activity from the auto-scanner and system events.

**URL:** `http://localhost:3000/activity`

**Try it:**
1. Enable auto-scan on a repository
2. Wait for a scan to run or force one
3. Watch events appear in real-time:
   - 🔍 Scan started
   - ✅ Scan completed
   - ❌ Errors
   - 📥 Repository cloned
   - 🔄 Git updates
   - ⚠️ Issues found

**Auto-refresh:**
- Page refreshes every 10 seconds automatically
- No need to reload manually

**Event levels:**
- Info (default) - normal operations
- Warn - non-critical issues
- Error - failures requiring attention

---

### ⚙️ Repository Settings

**What it does:** Configure auto-scan behavior per repository.

**URL:** `http://localhost:3000/repos/:id/settings`

**Try it:**
1. Go to Repositories page
2. Click on a repo card
3. Click "⚙️ Settings" button
4. Adjust scan interval (minutes)
5. Save settings

**Scan intervals:**
- 15 minutes - very frequent (for active development)
- 60 minutes - hourly (default)
- 240 minutes - every 4 hours (low priority)
- 1440 minutes - daily (archived repos)

---

## 3. Integration Testing

### Test Auto-Scanner Event Logging

1. Enable auto-scan on a repo:
   ```bash
   curl -X POST http://localhost:3000/repos/:repo_id/toggle-scan
   ```

2. Force a scan:
   ```bash
   curl -X GET http://localhost:3000/scanner/:repo_id/force
   ```

3. Check activity feed:
   - Visit `http://localhost:3000/activity`
   - Should see scan_start, scan_complete events
   - Check timing, file counts, issues found

### Test Ideas Workflow

1. Create idea with tag "urgent"
2. Filter by "urgent" tag
3. Mark as "active"
4. Filter by status "active"
5. Mark as "done"
6. Verify it moves to done section

### Test Docs Search

1. Create 3-4 documents with different content
2. Use search bar to find specific terms
3. Verify FTS5 ranks results correctly
4. Test tag filtering in combination with search

### Test Navigation

1. Start at Dashboard
2. Click "Ideas" - should load ideas page
3. Click "Docs" - should load docs page
4. Click "Activity" - should load activity feed
5. Click "Repos" - should return to repos
6. Verify timezone selector appears on all pages

---

## 4. API Endpoints (JSON)

### Scan Progress API

**GET** `/api/scan-progress`

Returns JSON with current scan status for all repos:

```json
[
  {
    "id": "repo-uuid",
    "name": "rustassistant",
    "scan_status": "scanning",
    "scan_files_done": 42,
    "scan_files_total": 100,
    "scan_issues_found": 3,
    "scan_duration_ms": null,
    "last_scan_error": null,
    "percent": 42.0
  }
]
```

**Use case:** Build custom dashboards, monitoring tools

### Activity Feed Partial (HTMX)

**GET** `/activity/feed`

Returns HTML fragment with recent events (for HTMX auto-refresh):

```html
<div class="event-row event-info">
  <span class="event-icon">🔍</span>
  <span class="event-type">scan_start</span>
  <span class="event-msg">Starting scan of rustassistant</span>
  <span class="event-time" data-utc="1234567890">—</span>
</div>
```

---

## 5. Database Queries (for debugging)

### Check recent scan events

```sql
SELECT * FROM scan_events ORDER BY created_at DESC LIMIT 20;
```

### Check ideas by status

```sql
SELECT * FROM ideas WHERE status = 'inbox' ORDER BY created_at DESC;
```

### Check documents with full-text search

```sql
SELECT * FROM documents_fts WHERE documents_fts MATCH 'rust performance';
```

### Check tag usage

```sql
SELECT * FROM tags ORDER BY usage_count DESC LIMIT 10;
```

---

## 6. Troubleshooting

### No events in activity feed

**Cause:** Auto-scanner not running or no scans triggered

**Fix:**
1. Check `AUTO_SCAN_ENABLED=true` in environment
2. Enable auto-scan on at least one repository
3. Force a scan manually or wait for interval
4. Check server logs for scanner activity

### Ideas not saving

**Cause:** Missing `ideas` table

**Fix:**
Run migration to create ideas table:
```sql
CREATE TABLE IF NOT EXISTS ideas (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    tags TEXT,
    project TEXT,
    repo_id TEXT,
    priority INTEGER NOT NULL DEFAULT 3,
    status TEXT NOT NULL DEFAULT 'inbox',
    category TEXT,
    linked_doc_id TEXT,
    linked_task_id TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
```

### FTS5 search not working

**Cause:** Missing `documents_fts` virtual table

**Fix:**
```sql
CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
    title,
    content,
    tags,
    content='documents',
    content_rowid='rowid'
);
```

### Timezone selector not appearing

**Cause:** Navigation template not updated

**Fix:** Verify `timezone_selector_html()` is called in nav sections

---

## 7. Next Steps

- **Bulk import:** Add documents from external sources (GitHub wikis, Notion exports)
- **Smart tagging:** Auto-suggest tags based on content
- **Idea prioritization:** Sort by urgency/importance matrix
- **Export/backup:** Export ideas/docs as Markdown files
- **Search improvements:** Add fuzzy matching, filters by date range
- **Analytics:** Track most-viewed docs, most-used tags

---

## 8. Environment Variables

```bash
# Required
DATABASE_URL=sqlite:./data/rustassistant.db

# Auto-scanner config
AUTO_SCAN_ENABLED=true
AUTO_SCAN_INTERVAL=60           # minutes between scans
AUTO_SCAN_MAX_CONCURRENT=3      # max parallel scans

# Server config
BIND_ADDRESS=0.0.0.0:3000
REPOS_DIR=/path/to/repos        # where cloned repos are stored

# Optional: GitHub integration
GITHUB_TOKEN=ghp_xxxxx          # for private repos
```

---

## Support

If you encounter issues:

1. Check server logs for error messages
2. Verify database migrations have run successfully
3. Ensure all required tables exist (ideas, documents, tags, scan_events)
4. Check `docs/INTEGRATION_FIXES_COMPLETED.md` for known issues
5. File an issue on GitHub with reproduction steps

Happy testing! 🎉