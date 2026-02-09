-- Migration: Add cache_hash column to repositories table
-- This stores the precomputed cache directory hash so the web interface
-- doesn't need to recompute it (which can fail if the repo path doesn't
-- exist on the web server's filesystem).

ALTER TABLE repositories ADD COLUMN cache_hash TEXT;

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS idx_repositories_cache_hash ON repositories(cache_hash);
