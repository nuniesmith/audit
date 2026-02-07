-- Migration 010: Scan checkpoints for resumable scanning
-- Allows the auto-scanner to persist progress after each file analysis
-- so it can resume from where it left off after a crash or restart.

CREATE TABLE IF NOT EXISTS scan_checkpoints (
    repo_id TEXT NOT NULL,
    last_completed_index INTEGER NOT NULL,
    last_completed_file TEXT NOT NULL,
    files_analyzed INTEGER NOT NULL DEFAULT 0,
    files_cached INTEGER NOT NULL DEFAULT 0,
    cumulative_cost REAL NOT NULL DEFAULT 0.0,
    total_files INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (repo_id)
);
