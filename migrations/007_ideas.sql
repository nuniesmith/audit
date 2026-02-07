-- Migration: 007_ideas.sql
-- Description: Add ideas table for quick thought capture
-- Created: 2024-02-07
--
-- This migration adds the ideas table that was previously only in the todo/ directory.
-- The ideas table provides a lightweight capture system for thoughts, feature requests,
-- bugs, and other quick notes that may later be promoted to full tasks or documents.

-- ============================================================================
-- Ideas Table
-- ============================================================================

CREATE TABLE IF NOT EXISTS ideas (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    tags TEXT,                  -- Comma-separated tags
    project TEXT,               -- Optional project link
    repo_id TEXT,               -- Optional repo link
    priority INTEGER NOT NULL DEFAULT 3,
    -- 1=urgent, 2=high, 3=normal, 4=low, 5=someday
    status TEXT NOT NULL DEFAULT 'inbox',
    -- status: 'inbox', 'active', 'in_progress', 'done', 'archived'
    category TEXT,
    -- category: 'feature', 'bug', 'improvement', 'research', 'question', 'random'
    linked_doc_id TEXT,         -- Optional link to a document
    linked_task_id TEXT,        -- Optional link to a task
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE SET NULL,
    FOREIGN KEY (linked_doc_id) REFERENCES documents(id) ON DELETE SET NULL
);

-- ============================================================================
-- Indexes for Ideas
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_ideas_status ON ideas(status);
CREATE INDEX IF NOT EXISTS idx_ideas_priority ON ideas(priority);
CREATE INDEX IF NOT EXISTS idx_ideas_category ON ideas(category);
CREATE INDEX IF NOT EXISTS idx_ideas_tags ON ideas(tags);
CREATE INDEX IF NOT EXISTS idx_ideas_project ON ideas(project);
CREATE INDEX IF NOT EXISTS idx_ideas_created ON ideas(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ideas_repo ON ideas(repo_id)
    WHERE repo_id IS NOT NULL;

-- ============================================================================
-- Views for Ideas
-- ============================================================================

-- Active ideas by priority
CREATE VIEW IF NOT EXISTS active_ideas AS
SELECT
    id,
    content,
    tags,
    project,
    priority,
    status,
    category,
    created_at,
    updated_at,
    CASE priority
        WHEN 1 THEN 'urgent'
        WHEN 2 THEN 'high'
        WHEN 3 THEN 'normal'
        WHEN 4 THEN 'low'
        WHEN 5 THEN 'someday'
        ELSE 'unknown'
    END as priority_label
FROM ideas
WHERE status IN ('inbox', 'active', 'in_progress')
ORDER BY priority ASC, created_at DESC;

-- Ideas by category summary
CREATE VIEW IF NOT EXISTS ideas_by_category AS
SELECT
    COALESCE(category, 'uncategorized') as category,
    COUNT(*) as count,
    COUNT(CASE WHEN status = 'inbox' THEN 1 END) as inbox_count,
    COUNT(CASE WHEN status = 'active' THEN 1 END) as active_count,
    COUNT(CASE WHEN status = 'done' THEN 1 END) as done_count
FROM ideas
GROUP BY category
ORDER BY count DESC;

-- Recent ideas activity
CREATE VIEW IF NOT EXISTS recent_ideas_activity AS
SELECT
    i.id,
    i.content,
    i.status,
    i.category,
    i.priority,
    i.tags,
    i.project,
    r.name as repo_name,
    i.created_at,
    i.updated_at,
    datetime(i.created_at, 'unixepoch') as created_at_formatted,
    datetime(i.updated_at, 'unixepoch') as updated_at_formatted
FROM ideas i
LEFT JOIN repositories r ON i.repo_id = r.id
ORDER BY i.updated_at DESC
LIMIT 100;

-- ============================================================================
-- Triggers for Ideas
-- ============================================================================

-- Update ideas updated_at timestamp on modification
CREATE TRIGGER IF NOT EXISTS update_idea_timestamp
AFTER UPDATE ON ideas
FOR EACH ROW
WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE ideas
    SET updated_at = strftime('%s', 'now')
    WHERE id = NEW.id;
END;

-- ============================================================================
-- Full-Text Search for Documents (if not already created)
-- ============================================================================
-- Note: This was in the todo/003_scan_docs_ideas.sql but may not have been
-- applied if migration 006 already created the documents table.
-- We use IF NOT EXISTS to make this idempotent.

CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
    title,
    content,
    tags,
    content='documents',
    content_rowid='rowid'
);

-- Triggers to keep FTS in sync with documents table
-- (Only create if they don't exist - migration 006 may have already created these)

-- Note: SQLite doesn't support CREATE TRIGGER IF NOT EXISTS before version 3.32.0
-- We'll use a workaround by dropping and recreating

DROP TRIGGER IF EXISTS documents_ai;
CREATE TRIGGER documents_ai AFTER INSERT ON documents BEGIN
    INSERT INTO documents_fts(rowid, title, content, tags)
    VALUES (new.rowid, new.title, new.content, new.tags);
END;

DROP TRIGGER IF EXISTS documents_ad;
CREATE TRIGGER documents_ad AFTER DELETE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, title, content, tags)
    VALUES ('delete', old.rowid, old.title, old.content, old.tags);
END;

DROP TRIGGER IF EXISTS documents_au;
CREATE TRIGGER documents_au AFTER UPDATE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, title, content, tags)
    VALUES ('delete', old.rowid, old.title, old.content, old.tags);
    INSERT INTO documents_fts(rowid, title, content, tags)
    VALUES (new.rowid, new.title, new.content, new.tags);
END;

-- ============================================================================
-- Sample Data (Optional)
-- ============================================================================

-- Add a welcome idea as an example
INSERT OR IGNORE INTO ideas (
    id,
    content,
    tags,
    priority,
    status,
    category
) VALUES (
    'welcome-idea',
    'Welcome to the Ideas system! Use this to capture quick thoughts, feature requests, bugs, and todos. Ideas can be filtered by status, priority, category, and tags.',
    'welcome,getting-started',
    3,
    'inbox',
    'random'
);

-- ============================================================================
-- Migration Complete
-- ============================================================================

-- Summary:
-- - Created ideas table for lightweight thought capture
-- - Added indexes for efficient querying
-- - Created views for common queries (active ideas, categories, recent activity)
-- - Added triggers for timestamp management
-- - Ensured documents_fts virtual table and triggers exist
-- - Added welcome idea as example

-- Next steps:
-- 1. Use /ideas page in web UI to view and manage ideas
-- 2. Convert important ideas to tasks or documents
-- 3. Use tags and categories to organize ideas
