-- ============================================================================
-- GitHub Integration Schema
-- Migration: 002_github_integration.sql
-- ============================================================================
-- This migration adds tables for syncing GitHub data locally, enabling:
-- - Cost-free GitHub API queries (vs expensive LLM calls)
-- - Offline access to GitHub data
-- - Fast search across repos, issues, PRs, and commits
-- - Bidirectional sync between GitHub and rustassistant
-- ============================================================================

-- ============================================================================
-- Repositories Table
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_repositories (
    id INTEGER PRIMARY KEY,
    node_id TEXT NOT NULL,
    name TEXT NOT NULL,
    full_name TEXT NOT NULL UNIQUE,
    owner_login TEXT NOT NULL,
    owner_id INTEGER NOT NULL,
    description TEXT,
    html_url TEXT NOT NULL,
    clone_url TEXT NOT NULL,
    ssh_url TEXT NOT NULL,
    homepage TEXT,
    language TEXT,

    -- Visibility
    private INTEGER NOT NULL DEFAULT 0,
    fork INTEGER NOT NULL DEFAULT 0,
    archived INTEGER NOT NULL DEFAULT 0,
    disabled INTEGER NOT NULL DEFAULT 0,

    -- Statistics
    stargazers_count INTEGER NOT NULL DEFAULT 0,
    watchers_count INTEGER NOT NULL DEFAULT 0,
    forks_count INTEGER NOT NULL DEFAULT 0,
    open_issues_count INTEGER NOT NULL DEFAULT 0,
    size INTEGER NOT NULL DEFAULT 0, -- KB

    -- Features
    topics TEXT, -- JSON array
    has_issues INTEGER NOT NULL DEFAULT 1,
    has_projects INTEGER NOT NULL DEFAULT 1,
    has_wiki INTEGER NOT NULL DEFAULT 1,
    has_pages INTEGER NOT NULL DEFAULT 0,
    has_downloads INTEGER NOT NULL DEFAULT 1,

    -- Branches
    default_branch TEXT NOT NULL DEFAULT 'main',

    -- Timestamps (Unix epoch)
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    pushed_at INTEGER,

    -- Sync metadata
    last_synced_at INTEGER NOT NULL,
    sync_enabled INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_github_repos_owner ON github_repositories(owner_login);
CREATE INDEX IF NOT EXISTS idx_github_repos_language ON github_repositories(language);
CREATE INDEX IF NOT EXISTS idx_github_repos_archived ON github_repositories(archived);
CREATE INDEX IF NOT EXISTS idx_github_repos_sync ON github_repositories(sync_enabled);
CREATE INDEX IF NOT EXISTS idx_github_repos_full_name ON github_repositories(full_name);

-- ============================================================================
-- Issues Table
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_issues (
    id INTEGER PRIMARY KEY,
    node_id TEXT NOT NULL,
    repo_id INTEGER NOT NULL,
    number INTEGER NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    body_text TEXT, -- Plain text without markdown

    -- User info
    user_login TEXT NOT NULL,
    user_id INTEGER NOT NULL,

    -- State
    state TEXT NOT NULL CHECK(state IN ('open', 'closed')),
    state_reason TEXT, -- completed, not_planned, reopened
    locked INTEGER NOT NULL DEFAULT 0,

    -- Metadata
    labels TEXT, -- JSON array of label names
    assignees TEXT, -- JSON array of user logins
    milestone_id INTEGER,
    comments INTEGER NOT NULL DEFAULT 0,

    -- URLs
    html_url TEXT NOT NULL,

    -- Timestamps
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    closed_at INTEGER,

    -- PR relationship
    is_pull_request INTEGER NOT NULL DEFAULT 0,

    -- Sync metadata
    last_synced_at INTEGER NOT NULL,

    FOREIGN KEY (repo_id) REFERENCES github_repositories(id) ON DELETE CASCADE,
    UNIQUE(repo_id, number)
);

CREATE INDEX IF NOT EXISTS idx_github_issues_repo ON github_issues(repo_id);
CREATE INDEX IF NOT EXISTS idx_github_issues_state ON github_issues(state);
CREATE INDEX IF NOT EXISTS idx_github_issues_user ON github_issues(user_login);
CREATE INDEX IF NOT EXISTS idx_github_issues_updated ON github_issues(updated_at);
CREATE INDEX IF NOT EXISTS idx_github_issues_pr ON github_issues(is_pull_request);
CREATE INDEX IF NOT EXISTS idx_github_issues_title ON github_issues(title);

-- Full-text search on issues
CREATE VIRTUAL TABLE IF NOT EXISTS github_issues_fts USING fts5(
    title,
    body,
    content='github_issues',
    content_rowid='id'
);

-- Triggers to keep FTS table in sync
CREATE TRIGGER IF NOT EXISTS github_issues_fts_insert AFTER INSERT ON github_issues BEGIN
    INSERT INTO github_issues_fts(rowid, title, body) VALUES (new.id, new.title, new.body);
END;

CREATE TRIGGER IF NOT EXISTS github_issues_fts_delete AFTER DELETE ON github_issues BEGIN
    DELETE FROM github_issues_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS github_issues_fts_update AFTER UPDATE ON github_issues BEGIN
    DELETE FROM github_issues_fts WHERE rowid = old.id;
    INSERT INTO github_issues_fts(rowid, title, body) VALUES (new.id, new.title, new.body);
END;

-- ============================================================================
-- Pull Requests Table
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_pull_requests (
    id INTEGER PRIMARY KEY,
    node_id TEXT NOT NULL,
    repo_id INTEGER NOT NULL,
    number INTEGER NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    body_text TEXT,

    -- User info
    user_login TEXT NOT NULL,
    user_id INTEGER NOT NULL,

    -- State
    state TEXT NOT NULL CHECK(state IN ('open', 'closed')),
    draft INTEGER NOT NULL DEFAULT 0,
    merged INTEGER NOT NULL DEFAULT 0,
    mergeable INTEGER, -- NULL if unknown
    mergeable_state TEXT,

    -- Branch info
    head_ref TEXT NOT NULL,
    head_sha TEXT NOT NULL,
    head_repo_id INTEGER, -- May be NULL if fork
    base_ref TEXT NOT NULL,
    base_sha TEXT NOT NULL,

    -- Review info
    requested_reviewers TEXT, -- JSON array
    labels TEXT, -- JSON array
    milestone_id INTEGER,

    -- Statistics
    commits INTEGER NOT NULL DEFAULT 0,
    additions INTEGER NOT NULL DEFAULT 0,
    deletions INTEGER NOT NULL DEFAULT 0,
    changed_files INTEGER NOT NULL DEFAULT 0,
    comments INTEGER NOT NULL DEFAULT 0,
    review_comments INTEGER NOT NULL DEFAULT 0,

    -- URLs
    html_url TEXT NOT NULL,
    diff_url TEXT NOT NULL,
    patch_url TEXT NOT NULL,

    -- Timestamps
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    closed_at INTEGER,
    merged_at INTEGER,
    merged_by_login TEXT,

    -- Sync metadata
    last_synced_at INTEGER NOT NULL,

    FOREIGN KEY (repo_id) REFERENCES github_repositories(id) ON DELETE CASCADE,
    UNIQUE(repo_id, number)
);

CREATE INDEX IF NOT EXISTS idx_github_prs_repo ON github_pull_requests(repo_id);
CREATE INDEX IF NOT EXISTS idx_github_prs_state ON github_pull_requests(state);
CREATE INDEX IF NOT EXISTS idx_github_prs_user ON github_pull_requests(user_login);
CREATE INDEX IF NOT EXISTS idx_github_prs_draft ON github_pull_requests(draft);
CREATE INDEX IF NOT EXISTS idx_github_prs_merged ON github_pull_requests(merged);
CREATE INDEX IF NOT EXISTS idx_github_prs_updated ON github_pull_requests(updated_at);
CREATE INDEX IF NOT EXISTS idx_github_prs_base_ref ON github_pull_requests(base_ref);

-- Full-text search on PRs
CREATE VIRTUAL TABLE IF NOT EXISTS github_prs_fts USING fts5(
    title,
    body,
    content='github_pull_requests',
    content_rowid='id'
);

CREATE TRIGGER IF NOT EXISTS github_prs_fts_insert AFTER INSERT ON github_pull_requests BEGIN
    INSERT INTO github_prs_fts(rowid, title, body) VALUES (new.id, new.title, new.body);
END;

CREATE TRIGGER IF NOT EXISTS github_prs_fts_delete AFTER DELETE ON github_pull_requests BEGIN
    DELETE FROM github_prs_fts WHERE rowid = old.id;
END;

CREATE TRIGGER IF NOT EXISTS github_prs_fts_update AFTER UPDATE ON github_pull_requests BEGIN
    DELETE FROM github_prs_fts WHERE rowid = old.id;
    INSERT INTO github_prs_fts(rowid, title, body) VALUES (new.id, new.title, new.body);
END;

-- ============================================================================
-- Commits Table
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_commits (
    sha TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    repo_id INTEGER NOT NULL,

    -- Author (Git signature)
    author_name TEXT NOT NULL,
    author_email TEXT NOT NULL,
    author_date INTEGER NOT NULL,

    -- Committer (Git signature)
    committer_name TEXT NOT NULL,
    committer_email TEXT NOT NULL,
    committer_date INTEGER NOT NULL,

    -- GitHub user (may be NULL if no GitHub account)
    author_github_login TEXT,
    committer_github_login TEXT,

    -- Message
    message TEXT NOT NULL,
    comment_count INTEGER NOT NULL DEFAULT 0,

    -- Statistics
    additions INTEGER,
    deletions INTEGER,
    total_changes INTEGER,

    -- Verification
    verified INTEGER NOT NULL DEFAULT 0,

    -- URLs
    html_url TEXT NOT NULL,

    -- Sync metadata
    created_at INTEGER NOT NULL,
    last_synced_at INTEGER NOT NULL,

    FOREIGN KEY (repo_id) REFERENCES github_repositories(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_github_commits_repo ON github_commits(repo_id);
CREATE INDEX IF NOT EXISTS idx_github_commits_author ON github_commits(author_name);
CREATE INDEX IF NOT EXISTS idx_github_commits_date ON github_commits(author_date);
CREATE INDEX IF NOT EXISTS idx_github_commits_verified ON github_commits(verified);

-- Full-text search on commits
CREATE VIRTUAL TABLE IF NOT EXISTS github_commits_fts USING fts5(
    message,
    author_name,
    content='github_commits',
    content_rowid='rowid'
);

CREATE TRIGGER IF NOT EXISTS github_commits_fts_insert AFTER INSERT ON github_commits BEGIN
    INSERT INTO github_commits_fts(rowid, message, author_name)
    VALUES ((SELECT rowid FROM github_commits WHERE sha = new.sha), new.message, new.author_name);
END;

CREATE TRIGGER IF NOT EXISTS github_commits_fts_delete AFTER DELETE ON github_commits BEGIN
    DELETE FROM github_commits_fts WHERE rowid = (SELECT rowid FROM github_commits WHERE sha = old.sha);
END;

CREATE TRIGGER IF NOT EXISTS github_commits_fts_update AFTER UPDATE ON github_commits BEGIN
    DELETE FROM github_commits_fts WHERE rowid = (SELECT rowid FROM github_commits WHERE sha = old.sha);
    INSERT INTO github_commits_fts(rowid, message, author_name)
    VALUES ((SELECT rowid FROM github_commits WHERE sha = new.sha), new.message, new.author_name);
END;

-- ============================================================================
-- Labels Table
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_labels (
    id INTEGER PRIMARY KEY,
    node_id TEXT NOT NULL,
    repo_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL, -- Hex color code
    description TEXT,
    is_default INTEGER NOT NULL DEFAULT 0,

    FOREIGN KEY (repo_id) REFERENCES github_repositories(id) ON DELETE CASCADE,
    UNIQUE(repo_id, name)
);

CREATE INDEX IF NOT EXISTS idx_github_labels_repo ON github_labels(repo_id);
CREATE INDEX IF NOT EXISTS idx_github_labels_name ON github_labels(name);

-- ============================================================================
-- Milestones Table
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_milestones (
    id INTEGER PRIMARY KEY,
    node_id TEXT NOT NULL,
    repo_id INTEGER NOT NULL,
    number INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    state TEXT NOT NULL CHECK(state IN ('open', 'closed')),
    open_issues INTEGER NOT NULL DEFAULT 0,
    closed_issues INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    due_on INTEGER,
    closed_at INTEGER,
    creator_login TEXT,

    FOREIGN KEY (repo_id) REFERENCES github_repositories(id) ON DELETE CASCADE,
    UNIQUE(repo_id, number)
);

CREATE INDEX IF NOT EXISTS idx_github_milestones_repo ON github_milestones(repo_id);
CREATE INDEX IF NOT EXISTS idx_github_milestones_state ON github_milestones(state);

-- ============================================================================
-- Sync History Table (for tracking sync operations)
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_sync_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at INTEGER NOT NULL,
    completed_at INTEGER NOT NULL,
    duration_secs REAL NOT NULL,
    repos_synced INTEGER NOT NULL DEFAULT 0,
    issues_synced INTEGER NOT NULL DEFAULT 0,
    prs_synced INTEGER NOT NULL DEFAULT 0,
    commits_synced INTEGER NOT NULL DEFAULT 0,
    items_created INTEGER NOT NULL DEFAULT 0,
    items_updated INTEGER NOT NULL DEFAULT 0,
    errors_count INTEGER NOT NULL DEFAULT 0,
    errors TEXT, -- JSON array of error messages
    warnings TEXT -- JSON array of warnings
);

CREATE INDEX IF NOT EXISTS idx_github_sync_history_completed ON github_sync_history(completed_at);

-- ============================================================================
-- Webhook Events Table (for audit trail)
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_webhook_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    delivery_id TEXT NOT NULL UNIQUE,
    event_type TEXT NOT NULL,
    action TEXT,
    repo_id INTEGER,
    payload TEXT NOT NULL, -- Full JSON payload
    processed INTEGER NOT NULL DEFAULT 0,
    processed_at INTEGER,
    error TEXT,
    received_at INTEGER NOT NULL,

    FOREIGN KEY (repo_id) REFERENCES github_repositories(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_github_webhooks_type ON github_webhook_events(event_type);
CREATE INDEX IF NOT EXISTS idx_github_webhooks_processed ON github_webhook_events(processed);
CREATE INDEX IF NOT EXISTS idx_github_webhooks_received ON github_webhook_events(received_at);

-- ============================================================================
-- Views for Common Queries
-- ============================================================================

-- Active repositories (not archived, sync enabled)
CREATE VIEW IF NOT EXISTS github_active_repos AS
SELECT * FROM github_repositories
WHERE archived = 0 AND sync_enabled = 1;

-- Open issues (not PRs)
CREATE VIEW IF NOT EXISTS github_open_issues AS
SELECT i.*, r.full_name as repo_full_name
FROM github_issues i
JOIN github_repositories r ON i.repo_id = r.id
WHERE i.state = 'open' AND i.is_pull_request = 0;

-- Open PRs
CREATE VIEW IF NOT EXISTS github_open_prs AS
SELECT p.*, r.full_name as repo_full_name
FROM github_pull_requests p
JOIN github_repositories r ON p.repo_id = r.id
WHERE p.state = 'open';

-- PRs needing review (open, not draft)
CREATE VIEW IF NOT EXISTS github_prs_needing_review AS
SELECT p.*, r.full_name as repo_full_name
FROM github_pull_requests p
JOIN github_repositories r ON p.repo_id = r.id
WHERE p.state = 'open' AND p.draft = 0
ORDER BY p.updated_at DESC;

-- Recent activity (commits from last 7 days)
CREATE VIEW IF NOT EXISTS github_recent_commits AS
SELECT c.*, r.full_name as repo_full_name
FROM github_commits c
JOIN github_repositories r ON c.repo_id = r.id
WHERE c.author_date > (strftime('%s', 'now') - 604800) -- 7 days
ORDER BY c.author_date DESC;

-- ============================================================================
-- Initial Configuration
-- ============================================================================

-- Store GitHub API configuration
CREATE TABLE IF NOT EXISTS github_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Insert default configuration
INSERT OR IGNORE INTO github_config (key, value, updated_at) VALUES
    ('api_version', '2022-11-28', strftime('%s', 'now')),
    ('sync_interval_seconds', '3600', strftime('%s', 'now')),
    ('default_commits_limit', '100', strftime('%s', 'now')),
    ('auto_sync_enabled', '1', strftime('%s', 'now'));

-- ============================================================================
-- Migration Complete
-- ============================================================================
-- GitHub integration schema created successfully
-- Tables: 11
-- Indexes: 25
-- Views: 5
-- FTS tables: 3
-- ============================================================================

-- ============================================================================
-- Sync Metadata Table
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_sync_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

