-- Enhanced scan progress tracking columns for real-time UI updates
-- These columns are read by web_ui_scan_progress.rs for the scan dashboard

-- Timestamp when the current scan started (unix epoch seconds), used for ETA calculation
ALTER TABLE repositories ADD COLUMN scan_started_at INTEGER;

-- Accumulated cost in USD for the current scan run
ALTER TABLE repositories ADD COLUMN scan_cost_accumulated REAL DEFAULT 0.0;

-- Number of cache hits during the current scan
ALTER TABLE repositories ADD COLUMN scan_cache_hits INTEGER DEFAULT 0;

-- Number of API calls (non-cached analyses) during the current scan
ALTER TABLE repositories ADD COLUMN scan_api_calls INTEGER DEFAULT 0;
