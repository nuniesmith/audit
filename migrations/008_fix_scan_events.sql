-- Migration: 008_fix_scan_events.sql
-- Description: Rebuild scan_events table to add 'details' and 'level' columns,
--              make repo_id nullable, and relax the event_type CHECK constraint
--              to support all event types used by the application.
--              Also fixes repository_health view bug (scan_interval_minutes alias
--              used in CASE expression instead of actual column scan_interval_mins).
-- Created: 2025-02-07

-- ============================================================================
-- Step 0: Drop views that depend on scan_events or have bugs
-- ============================================================================
-- recent_scan_activity references scan_events directly
DROP VIEW IF EXISTS recent_scan_activity;
-- repository_health has a bug: references alias scan_interval_minutes in CASE
DROP VIEW IF EXISTS repository_health;
-- active_scans doesn't reference scan_events but drop/recreate for consistency
DROP VIEW IF EXISTS active_scans;

-- ============================================================================
-- Step 1: Recreate scan_events table with correct schema
-- ============================================================================

-- Rename existing table
ALTER TABLE scan_events RENAME TO scan_events_old;

-- Create new table with correct schema
CREATE TABLE scan_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id TEXT,  -- nullable for system-level events
    event_type TEXT NOT NULL,
    message TEXT NOT NULL,
    details TEXT,       -- additional detail text
    metadata TEXT,      -- JSON blob for structured additional event data (preserved for compatibility)
    level TEXT NOT NULL DEFAULT 'info',  -- 'info', 'warn', 'error'
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    FOREIGN KEY (repo_id) REFERENCES repositories(id) ON DELETE CASCADE
);

-- Copy existing data, mapping old columns to new ones
INSERT INTO scan_events (id, repo_id, event_type, message, details, metadata, level, created_at)
SELECT id, repo_id, event_type, message, metadata, metadata, 'info', created_at
FROM scan_events_old;

-- Drop old table
DROP TABLE scan_events_old;

-- ============================================================================
-- Step 2: Recreate indexes on scan_events
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_scan_events_created
ON scan_events(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_scan_events_repo
ON scan_events(repo_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_scan_events_type
ON scan_events(event_type, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_scan_events_level
ON scan_events(level, created_at DESC);

-- ============================================================================
-- Step 3: Recreate views with fixes
-- ============================================================================

-- View for active scans (unchanged)
CREATE VIEW IF NOT EXISTS active_scans AS
SELECT
    id,
    name,
    scan_status,
    scan_progress,
    scan_current_file,
    scan_files_processed,
    scan_files_total,
    CASE
        WHEN scan_files_total > 0 THEN
            CAST((scan_files_processed * 100.0 / scan_files_total) AS INTEGER)
        ELSE 0
    END as progress_percentage,
    strftime('%Y-%m-%d %H:%M:%S', last_scanned_at, 'unixepoch') as scan_started_at
FROM repositories
WHERE scan_status = 'scanning';

-- View for recent scan activity (updated to include new columns)
CREATE VIEW IF NOT EXISTS recent_scan_activity AS
SELECT
    r.id,
    r.name,
    e.event_type,
    e.message,
    e.details,
    e.metadata,
    e.level,
    strftime('%Y-%m-%d %H:%M:%S', e.created_at, 'unixepoch') as event_time
FROM scan_events e
LEFT JOIN repositories r ON e.repo_id = r.id
ORDER BY e.created_at DESC
LIMIT 50;

-- View for repository health summary (FIXED: use scan_interval_mins instead of alias)
CREATE VIEW IF NOT EXISTS repository_health AS
SELECT
    id,
    name,
    scan_status,
    auto_scan,
    scan_interval_mins as scan_interval_minutes,
    last_scan_duration_ms,
    last_scan_files_found,
    last_scan_issues_found,
    CASE
        WHEN last_error IS NOT NULL THEN 'unhealthy'
        WHEN scan_status = 'error' THEN 'unhealthy'
        WHEN last_scanned_at IS NULL THEN 'never_scanned'
        WHEN (strftime('%s', 'now') - last_scanned_at) > (scan_interval_mins * 60 * 2) THEN 'stale'
        ELSE 'healthy'
    END as health_status,
    strftime('%Y-%m-%d %H:%M:%S', last_scanned_at, 'unixepoch') as last_scan
FROM repositories;
