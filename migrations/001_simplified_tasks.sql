-- Simplified Task Schema Migration
-- Consolidates QueueItem, FileAnalysis, and TodoItem into a single Task table
-- Run with: sqlx migrate run

-- ============================================================================
-- Drop old tables (backup first in production!)
-- ============================================================================

-- Uncomment these after backing up your data:
-- DROP TABLE IF EXISTS queue_items;
-- DROP TABLE IF EXISTS file_analysis;
-- DROP TABLE IF EXISTS todo_items;

-- ============================================================================
-- Core Task Table
-- ============================================================================

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY NOT NULL,
    
    -- Content
    content TEXT NOT NULL,              -- The task description/TODO/issue
    context TEXT,                       -- Surrounding code, file content for LLM
    llm_suggestion TEXT,                -- LLM-generated fix/implementation
    
    -- Source tracking
    source_type TEXT NOT NULL DEFAULT 'manual',  -- 'todo', 'scan', 'manual', 'idea'
    source_repo TEXT,                   -- Repository name (e.g., 'rustassistant')
    source_file TEXT,                   -- File path within repo
    source_line INTEGER,                -- Line number for TODOs
    content_hash TEXT,                  -- For deduplication
    
    -- Status & Priority
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, processing, review, ready, done, failed
    priority INTEGER NOT NULL DEFAULT 5,     -- 1-10, higher = more important
    category TEXT,                           -- 'bug', 'refactor', 'feature', 'docs', 'test'
    
    -- Grouping
    group_id TEXT,                      -- Links related tasks together
    group_reason TEXT,                  -- Why tasks are grouped (same file, similar issue)
    
    -- Processing metadata
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    tokens_used INTEGER,                -- Track LLM token usage
    
    -- Timestamps
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    processed_at INTEGER,
    completed_at INTEGER
);

-- ============================================================================
-- Indexes for common queries
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority DESC);
CREATE INDEX IF NOT EXISTS idx_tasks_source_repo ON tasks(source_repo);
CREATE INDEX IF NOT EXISTS idx_tasks_source_file ON tasks(source_file);
CREATE INDEX IF NOT EXISTS idx_tasks_group_id ON tasks(group_id);
CREATE INDEX IF NOT EXISTS idx_tasks_content_hash ON tasks(content_hash);
CREATE INDEX IF NOT EXISTS idx_tasks_category ON tasks(category);

-- Composite index for queue queries (pending tasks by priority)
CREATE INDEX IF NOT EXISTS idx_tasks_queue ON tasks(status, priority DESC, created_at);

-- ============================================================================
-- Repository Tracking (simplified)
-- ============================================================================

CREATE TABLE IF NOT EXISTS repositories (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,          -- e.g., 'rustassistant'
    url TEXT,                           -- GitHub URL
    local_path TEXT,                    -- Local clone path
    
    auto_scan INTEGER NOT NULL DEFAULT 1,  -- Boolean: auto-scan enabled
    scan_interval_mins INTEGER NOT NULL DEFAULT 60,
    last_scanned_at INTEGER,
    last_commit_hash TEXT,              -- Detect changes
    
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS idx_repos_auto_scan ON repositories(auto_scan, last_scanned_at);

-- ============================================================================
-- Task Groups (for batch IDE handoff)
-- ============================================================================

CREATE TABLE IF NOT EXISTS task_groups (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,                 -- e.g., "src/queue/processor.rs issues"
    description TEXT,                   -- LLM-generated summary
    
    combined_priority INTEGER NOT NULL DEFAULT 5,
    task_count INTEGER NOT NULL DEFAULT 0,
    
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, in_progress, done
    
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    exported_at INTEGER                 -- When copied to IDE
);

-- ============================================================================
-- Processing Stats (for cost tracking)
-- ============================================================================

CREATE TABLE IF NOT EXISTS llm_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT,
    operation TEXT NOT NULL,            -- 'analyze', 'suggest', 'batch_analyze'
    tokens_input INTEGER NOT NULL DEFAULT 0,
    tokens_output INTEGER NOT NULL DEFAULT 0,
    cost_usd REAL,                      -- Calculated cost
    model TEXT DEFAULT 'grok-4.1',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

CREATE INDEX IF NOT EXISTS idx_llm_usage_created ON llm_usage(created_at);

-- ============================================================================
-- Views for common queries
-- ============================================================================

-- Pending tasks queue (what to work on next)
CREATE VIEW IF NOT EXISTS v_task_queue AS
SELECT 
    t.*,
    g.name as group_name,
    g.task_count as group_size
FROM tasks t
LEFT JOIN task_groups g ON t.group_id = g.id
WHERE t.status IN ('pending', 'review', 'ready')
ORDER BY t.priority DESC, t.created_at ASC;

-- Daily stats
CREATE VIEW IF NOT EXISTS v_daily_stats AS
SELECT 
    date(created_at, 'unixepoch') as day,
    COUNT(*) as tasks_created,
    SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END) as tasks_completed,
    SUM(tokens_used) as total_tokens
FROM tasks
GROUP BY date(created_at, 'unixepoch')
ORDER BY day DESC;

-- ============================================================================
-- Triggers for updated_at
-- ============================================================================

CREATE TRIGGER IF NOT EXISTS tasks_updated_at 
AFTER UPDATE ON tasks
BEGIN
    UPDATE tasks SET updated_at = unixepoch() WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS repos_updated_at
AFTER UPDATE ON repositories
BEGIN
    UPDATE repositories SET updated_at = unixepoch() WHERE id = NEW.id;
END;
