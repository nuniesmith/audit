-- Add columns expected by db/core.rs Task struct and auto_scanner create_task().
-- The tasks table was originally created by 001_simplified_tasks.sql with a
-- different column set (content, source_type, source_repo, source_file, etc.).
-- The consolidated code path uses title/description/source/repo_id/file_path
-- instead, so we add them here as nullable to coexist with the legacy columns.

ALTER TABLE tasks ADD COLUMN title TEXT;
ALTER TABLE tasks ADD COLUMN description TEXT;
ALTER TABLE tasks ADD COLUMN source TEXT DEFAULT 'manual';
ALTER TABLE tasks ADD COLUMN source_id TEXT;
ALTER TABLE tasks ADD COLUMN repo_id TEXT REFERENCES repositories(id);
ALTER TABLE tasks ADD COLUMN file_path TEXT;
ALTER TABLE tasks ADD COLUMN line_number INTEGER;
