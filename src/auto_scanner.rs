//! Automatic Repository Scanner
//!
//! Provides background scanning of enabled repositories at configurable intervals.
//! Monitors git status and automatically re-analyzes changed files.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::db::scan_events;
use crate::db::{Database, Repository};

use crate::refactor_assistant::RefactorAssistant;
use crate::repo_cache_sql::RepoCacheSql;
use crate::repo_manager::RepoManager;

/// Maximum file size to send to LLM analysis (100 KB)
const MAX_ANALYSIS_FILE_SIZE: u64 = 100 * 1024;

/// Default per-scan cost budget in dollars
const DEFAULT_SCAN_COST_BUDGET: f64 = 3.00;

/// Grok 4.1 Fast pricing constants (mirrors grok_client.rs)
const COST_PER_MILLION_INPUT: f64 = 0.20;
const COST_PER_MILLION_OUTPUT: f64 = 0.50;

/// Directories to always skip during scanning
const SKIP_DIRS: &[&str] = &[
    "/dist/",
    "/build/",
    "/node_modules/",
    "/target/",
    "/.git/",
    "/vendor/",
    "/__pycache__/",
    "/.next/",
    "/out/",
    "/coverage/",
    "/.cache/",
];

/// File patterns to always skip (suffix match)
const SKIP_SUFFIXES: &[&str] = &[
    ".min.js",
    ".min.css",
    ".map",
    ".bundle.js",
    ".chunk.js",
    ".min.mjs",
    ".d.ts",
    ".lock",
];

/// Auto-scanner configuration
#[derive(Debug, Clone)]
pub struct AutoScannerConfig {
    /// Global enable/disable
    pub enabled: bool,
    /// Default scan interval in minutes
    pub default_interval_minutes: u64,
    /// Maximum concurrent scans
    pub max_concurrent_scans: usize,
    /// Per-scan cost budget in dollars (0.0 = unlimited)
    pub scan_cost_budget: f64,
}

impl Default for AutoScannerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_interval_minutes: 60,
            max_concurrent_scans: 2,
            scan_cost_budget: DEFAULT_SCAN_COST_BUDGET,
        }
    }
}

/// Git status for a file
#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Unmodified,
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
}

/// Repository scan state
#[derive(Debug, Clone)]
pub struct RepoScanState {
    pub repo_id: String,
    pub repo_path: PathBuf,
    pub last_scan: Option<i64>,
    pub last_git_hash: Option<String>,
    pub modified_files: Vec<PathBuf>,
}

/// Background repository scanner
pub struct AutoScanner {
    config: AutoScannerConfig,
    pool: sqlx::SqlitePool,
    repos_dir: PathBuf,
    scan_states: Arc<RwLock<HashMap<String, RepoScanState>>>,
    repo_manager: Arc<RepoManager>,
}

impl AutoScanner {
    /// Create a new auto-scanner
    pub fn new(config: AutoScannerConfig, pool: sqlx::SqlitePool, repos_dir: PathBuf) -> Self {
        // Get GitHub token from environment for private repos
        let github_token = std::env::var("GITHUB_TOKEN").ok();

        let repo_manager = Arc::new(
            RepoManager::new(&repos_dir, github_token).expect("Failed to create RepoManager"),
        );

        Self {
            config,
            pool,
            repos_dir,
            scan_states: Arc::new(RwLock::new(HashMap::new())),
            repo_manager,
        }
    }

    /// Start the background scanner
    pub async fn start(self: Arc<Self>) -> Result<()> {
        if !self.config.enabled {
            info!("Auto-scanner is disabled");
            return Ok(());
        }

        info!(
            "Starting auto-scanner with {} minute intervals",
            self.config.default_interval_minutes
        );

        // Main scan loop
        loop {
            if let Err(e) = self.scan_enabled_repos().await {
                error!("Error during scan cycle: {}", e);
            }

            // Sleep for 1 minute, then check which repos need scanning
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    /// Scan all enabled repositories
    async fn scan_enabled_repos(&self) -> Result<()> {
        let repos = self.get_enabled_repos().await?;

        if repos.is_empty() {
            debug!("No enabled repositories to scan");
            return Ok(());
        }

        info!("Checking {} enabled repositories", repos.len());

        // Process repos in parallel (limited concurrency)
        let semaphore = Arc::new(tokio::sync::Semaphore::new(
            self.config.max_concurrent_scans,
        ));
        let mut tasks = vec![];

        for repo in repos {
            let self_clone = Arc::new(self.clone_scanner());
            let semaphore_clone = semaphore.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.ok();
                if let Err(e) = self_clone.check_and_scan_repo(&repo).await {
                    error!("Failed to scan repo {}: {}", repo.name, e);
                }
            });

            tasks.push(task);
        }

        // Wait for all scans to complete
        for task in tasks {
            let _ = task.await;
        }

        Ok(())
    }

    /// Get all repositories with auto_scan_enabled = 1
    async fn get_enabled_repos(&self) -> Result<Vec<Repository>> {
        let repos = sqlx::query_as::<_, Repository>(
            r#"
            SELECT *
            FROM repositories
            WHERE auto_scan = 1
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(repos)
    }

    /// Check if repo needs scanning and scan if necessary
    async fn check_and_scan_repo(&self, repo: &Repository) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let interval_secs = repo.scan_interval_minutes * 60;

        // Check if enough time has passed since last scan
        if let Some(last_check) = repo.last_scan_check {
            if now - last_check < interval_secs {
                debug!(
                    "Skipping {} - scanned {} seconds ago",
                    repo.name,
                    now - last_check
                );
                return Ok(());
            }
        }

        info!("Scanning repository: {} ({})", repo.name, repo.path);

        // Track scan start time for duration calculation
        let scan_start = std::time::Instant::now();

        // Log scan start event
        if let Err(e) = scan_events::log_info(
            &self.pool,
            Some(&repo.id),
            "scan_start",
            &format!("Starting scan of {}", repo.name),
        )
        .await
        {
            warn!("Failed to log scan start event: {}", e);
        }

        // Ensure the repo exists locally — clone from git_url if missing
        let repo_path = PathBuf::from(&repo.path);
        let repo_path = if !repo_path.exists() || !repo_path.join(".git").exists() {
            if let Some(ref git_url) = repo.git_url {
                info!(
                    "Local path {} not found, cloning from {}",
                    repo_path.display(),
                    git_url
                );
                match self.clone_or_update_repo(git_url, &repo.name) {
                    Ok(cloned_path) => {
                        // Update the stored path in the database to the new clone location
                        let new_path = cloned_path.to_string_lossy().to_string();
                        if let Err(e) = self.update_repo_path(&repo.id, &new_path).await {
                            error!("Failed to update repo path in DB: {}", e);
                        }
                        info!("Cloned {} to {}", repo.name, cloned_path.display());

                        // Log clone event
                        if let Err(e) = scan_events::log_info(
                            &self.pool,
                            Some(&repo.id),
                            "repo_cloned",
                            &format!("Cloned repository to {}", cloned_path.display()),
                        )
                        .await
                        {
                            warn!("Failed to log clone event: {}", e);
                        }

                        cloned_path
                    }
                    Err(e) => {
                        error!("Failed to clone {} from {}: {}", repo.name, git_url, e);

                        // Log clone error event
                        if let Err(err) = scan_events::log_error(
                            &self.pool,
                            Some(&repo.id),
                            "clone_error",
                            &format!("Failed to clone {}", repo.name),
                            &e.to_string(),
                        )
                        .await
                        {
                            warn!("Failed to log clone error event: {}", err);
                        }

                        return Ok(());
                    }
                }
            } else {
                warn!(
                    "Repo {} path {} does not exist and no git_url configured — skipping",
                    repo.name,
                    repo_path.display()
                );
                return Ok(());
            }
        } else {
            repo_path
        };

        // Update repository if it exists (git pull)
        if let Some(ref git_url) = repo.git_url {
            match self.clone_or_update_repo(git_url, &repo.name) {
                Ok(_) => {
                    // Log successful update
                    if let Err(e) = scan_events::log_info(
                        &self.pool,
                        Some(&repo.id),
                        "git_update",
                        &format!("Updated repository {}", repo.name),
                    )
                    .await
                    {
                        warn!("Failed to log git update event: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Failed to update {}: {}", repo.name, e);
                }
            }
        }

        // Check for changes (both committed and uncommitted)
        let current_head = self.get_head_hash(&repo_path)?;
        let changed_files = self
            .get_changed_files(
                &repo_path,
                repo.last_commit_hash.as_deref(),
                current_head.as_deref(),
            )
            .await?;

        if changed_files.is_empty() {
            debug!("No changes detected in {}", repo.name);
            // Still update the commit hash so we don't re-diff the same range
            if let Some(ref hash) = current_head {
                self.update_last_commit_hash(&repo.id, hash).await?;
            }
            // Update last_scan_check for interval tracking
            self.update_last_scan_check(&repo.id, now).await?;
            return Ok(());
        }

        info!(
            "Found {} changed files in {}",
            changed_files.len(),
            repo.name
        );

        // Start progress tracking
        let total_files = changed_files.len() as i64;
        if let Err(e) = crate::db::core::start_scan(&self.pool, &repo.id, total_files).await {
            error!("Failed to start scan progress tracking: {}", e);
        }

        // Log scan progress event
        if let Err(e) =
            scan_events::mark_scan_started(&self.pool, &repo.id, total_files as i32).await
        {
            warn!("Failed to mark scan as started: {}", e);
        }

        // Analyze changed files with progress tracking
        let result = self
            .analyze_changed_files_with_progress(&repo.id, &repo_path, &changed_files)
            .await;

        match result {
            Ok((files_analyzed, issues_found, budget_halted)) => {
                // Calculate scan duration
                let duration_ms = scan_start.elapsed().as_millis() as i64;

                // Complete scan with metrics
                if let Err(e) = crate::db::core::complete_scan(
                    &self.pool,
                    &repo.id,
                    duration_ms,
                    files_analyzed,
                    issues_found,
                )
                .await
                {
                    error!("Failed to complete scan progress tracking: {}", e);
                }

                // Log scan completion event
                if let Err(e) = scan_events::mark_scan_complete(
                    &self.pool,
                    &repo.id,
                    files_analyzed as i32,
                    issues_found as i32,
                    duration_ms,
                )
                .await
                {
                    warn!("Failed to mark scan as complete: {}", e);
                }

                info!(
                    "Scan completed for {}: {} files, {} issues in {}ms",
                    repo.name, files_analyzed, issues_found, duration_ms
                );

                // Update last_analyzed timestamp
                self.update_last_analyzed(&repo.id, now).await?;

                // CRITICAL: Only store the commit hash if ALL files were analyzed.
                // If the budget cap halted the scan, we leave the hash unstored so
                // the next scan cycle will re-diff, hit cache on already-analyzed
                // files (free), and continue analyzing remaining files.
                if !budget_halted {
                    if let Some(ref hash) = current_head {
                        self.update_last_commit_hash(&repo.id, hash).await?;
                    }
                } else {
                    info!(
                        "Skipping commit hash update — budget halted scan. \
                         Next cycle will resume from cache hits."
                    );
                }
            }
            Err(e) => {
                error!("Scan failed for {}: {}", repo.name, e);
                if let Err(err) =
                    crate::db::core::fail_scan(&self.pool, &repo.id, &e.to_string()).await
                {
                    error!("Failed to mark scan as failed: {}", err);
                }

                // Log scan error event
                if let Err(err) =
                    scan_events::mark_scan_error(&self.pool, &repo.id, &e.to_string()).await
                {
                    warn!("Failed to log scan error: {}", err);
                }

                return Err(e);
            }
        }

        Ok(())
    }

    /// Clone or update a repository from a git URL into the repos directory
    fn clone_or_update_repo(&self, git_url: &str, name: &str) -> Result<PathBuf> {
        self.repo_manager
            .clone_or_update(git_url, name)
            .context(format!(
                "Failed to clone or update {} from {}",
                name, git_url
            ))
    }

    /// Update the stored path for a repository in the database
    async fn update_repo_path(&self, repo_id: &str, new_path: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE repositories
            SET local_path = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(new_path)
        .bind(chrono::Utc::now().timestamp())
        .bind(repo_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get the current HEAD commit hash for a repository
    fn get_head_hash(&self, repo_path: &Path) -> Result<Option<String>> {
        use std::process::Command;

        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_path)
            .output()
            .context("Failed to run git rev-parse HEAD")?;

        if !output.status.success() {
            warn!("git rev-parse HEAD failed for {}", repo_path.display());
            return Ok(None);
        }

        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if hash.is_empty() {
            Ok(None)
        } else {
            Ok(Some(hash))
        }
    }

    /// Get list of modified files from both committed and uncommitted changes
    async fn get_changed_files(
        &self,
        repo_path: &Path,
        last_commit_hash: Option<&str>,
        current_head: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        use std::collections::HashSet;
        use std::process::Command;

        let mut changed_set: HashSet<PathBuf> = HashSet::new();

        // 1. Check for committed changes since last known hash
        if let (Some(old_hash), Some(new_hash)) = (last_commit_hash, current_head) {
            if old_hash != new_hash {
                let output = Command::new("git")
                    .args(["diff", "--name-status", old_hash, new_hash])
                    .current_dir(repo_path)
                    .output();

                match output {
                    Ok(out) if out.status.success() => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        for line in stdout.lines() {
                            let parts: Vec<&str> = line.split('\t').collect();
                            if parts.len() < 2 {
                                continue;
                            }
                            let status = parts[0];
                            // Skip deleted files
                            if status.starts_with('D') {
                                continue;
                            }
                            // For renames (R100), the new path is the last element
                            let file_path = parts.last().unwrap().trim();
                            if Self::should_analyze_file(file_path) {
                                let full_path = repo_path.join(file_path);
                                if full_path.exists() {
                                    changed_set.insert(full_path);
                                } else {
                                    debug!(
                                        "Skipping {} - file does not exist on disk (deleted in later commit?)",
                                        file_path
                                    );
                                }
                            }
                        }
                        info!(
                            "Found {} files changed between commits {}..{}",
                            changed_set.len(),
                            &old_hash[..8.min(old_hash.len())],
                            &new_hash[..8.min(new_hash.len())]
                        );
                    }
                    Ok(out) => {
                        // git diff failed - old hash may no longer exist (force push, etc.)
                        // Fall back to listing all files in the latest commit
                        warn!(
                            "git diff failed for {}..{} ({}), falling back to HEAD diff",
                            &old_hash[..8.min(old_hash.len())],
                            &new_hash[..8.min(new_hash.len())],
                            String::from_utf8_lossy(&out.stderr).trim()
                        );
                        self.get_files_from_recent_commits(repo_path, &mut changed_set)?;
                    }
                    Err(e) => {
                        warn!("Failed to run git diff: {}", e);
                    }
                }
            }
        } else if last_commit_hash.is_none() && current_head.is_some() {
            // First scan - no stored hash yet. Check recent commits to seed initial analysis.
            info!(
                "First scan for {} - checking recent commits",
                repo_path.display()
            );
            self.get_files_from_recent_commits(repo_path, &mut changed_set)?;
        }

        // 2. Also check for uncommitted changes (working tree + staged)
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(repo_path)
            .output()
            .context("Failed to run git status")?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.len() < 3 {
                    continue;
                }

                let status = &line[0..2];
                let file_path = line[3..].trim();

                // Skip deleted files
                if status.contains('D') {
                    continue;
                }

                if Self::should_analyze_file(file_path) {
                    let full_path = repo_path.join(file_path);
                    if full_path.exists() {
                        changed_set.insert(full_path);
                    } else {
                        debug!("Skipping {} - file does not exist on disk", file_path);
                    }
                }
            }
        }

        Ok(changed_set.into_iter().collect())
    }

    /// Get changed files from recent commits (used for first scan or fallback)
    fn get_files_from_recent_commits(
        &self,
        repo_path: &Path,
        changed_set: &mut std::collections::HashSet<PathBuf>,
    ) -> Result<()> {
        use std::process::Command;

        // Look at files changed in the last 5 commits
        let output = Command::new("git")
            .args(["diff", "--name-only", "HEAD~5", "HEAD"])
            .current_dir(repo_path)
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    let file_path = line.trim();
                    if !file_path.is_empty() && Self::should_analyze_file(file_path) {
                        let full_path = repo_path.join(file_path);
                        if full_path.exists() {
                            changed_set.insert(full_path);
                        } else {
                            debug!("Skipping {} - file does not exist on disk", file_path);
                        }
                    }
                }
            }
            _ => {
                debug!("Could not get recent commits for {}", repo_path.display());
            }
        }

        Ok(())
    }

    /// Check if a file extension is one we should analyze
    fn is_analyzable_file(file_path: &str) -> bool {
        file_path.ends_with(".rs")
            || file_path.ends_with(".py")
            || file_path.ends_with(".js")
            || file_path.ends_with(".ts")
            || file_path.ends_with(".tsx")
            || file_path.ends_with(".sh")
            || file_path.ends_with(".kt")
            || file_path.ends_with(".java")
            || file_path.ends_with(".go")
            || file_path.ends_with(".rb")
    }

    /// Check if a file should be skipped based on path patterns.
    /// This catches generated/bundled/vendored code that wastes API budget.
    fn should_skip_path(file_path: &str) -> bool {
        // Normalize to forward slashes for consistent matching
        let normalized = file_path.replace('\\', "/");
        // Ensure we match directory components properly by wrapping in slashes
        let with_leading = if normalized.starts_with('/') {
            normalized.clone()
        } else {
            format!("/{}", normalized)
        };

        // Check directory patterns
        for dir in SKIP_DIRS {
            if with_leading.contains(dir) {
                return true;
            }
        }

        // Check suffix patterns (minified files, sourcemaps, etc.)
        for suffix in SKIP_SUFFIXES {
            if normalized.ends_with(suffix) {
                return true;
            }
        }

        false
    }

    /// Combined filter: is it a code file AND not in a skip path?
    fn should_analyze_file(file_path: &str) -> bool {
        Self::is_analyzable_file(file_path) && !Self::should_skip_path(file_path)
    }

    /// Analyze changed files with progress tracking and cost budget enforcement.
    /// Returns (files_analyzed, issues_found)
    async fn analyze_changed_files_with_progress(
        &self,
        repo_id: &str,
        repo_path: &Path,
        files: &[PathBuf],
    ) -> Result<(i64, i64, bool)> {
        let cache = RepoCacheSql::new_for_repo(repo_path).await?;
        let mut files_analyzed = 0i64;
        let mut issues_found = 0i64;
        let mut cumulative_cost = 0.0f64;
        let mut budget_halted = false;
        let progress_update_interval = 5; // Update progress every N files

        // Pre-filter files that match skip patterns (extra safety — get_changed_files
        // already filters, but files may have been added to the list via other paths)
        let analyzable_files: Vec<&PathBuf> = files
            .iter()
            .filter(|f| {
                let path_str = f.to_string_lossy();
                if Self::should_skip_path(&path_str) {
                    let rel = f.strip_prefix(repo_path).unwrap_or(f);
                    info!(
                        "Pre-filter: skipping {} — matches skip pattern",
                        rel.display()
                    );
                    false
                } else {
                    true
                }
            })
            .collect();

        let original_count = files.len();
        let filtered_count = analyzable_files.len();
        if original_count != filtered_count {
            info!(
                "Filtered {} → {} files ({} skipped by path/pattern rules)",
                original_count,
                filtered_count,
                original_count - filtered_count
            );
        }

        for (idx, file) in analyzable_files.iter().enumerate() {
            // Check cost budget before each file (using actual accumulated cost)
            if self.config.scan_cost_budget > 0.0 && cumulative_cost >= self.config.scan_cost_budget
            {
                warn!(
                    "⚠️  Scan cost budget reached (${:.4} >= ${:.2} limit). \
                     Stopping analysis with {} files remaining.",
                    cumulative_cost,
                    self.config.scan_cost_budget,
                    filtered_count - idx
                );
                budget_halted = true;
                break;
            }

            // Update progress periodically
            if idx % progress_update_interval == 0 || idx == filtered_count - 1 {
                let current_file = file
                    .strip_prefix(repo_path)
                    .unwrap_or(file)
                    .to_string_lossy()
                    .to_string();

                if let Err(e) = crate::db::core::update_scan_progress(
                    &self.pool,
                    repo_id,
                    idx as i64,
                    Some(&current_file),
                )
                .await
                {
                    error!("Failed to update scan progress: {}", e);
                }
            }

            match self.analyze_file(repo_path, file, &cache).await {
                Ok((found_issues, file_cost)) => {
                    files_analyzed += 1;
                    issues_found += found_issues;
                    cumulative_cost += file_cost;

                    // Log cost milestone every $0.50
                    if cumulative_cost > 0.0
                        && (cumulative_cost * 2.0) as i64
                            > ((cumulative_cost - file_cost) * 2.0) as i64
                    {
                        info!(
                            "💰 Scan cost: ${:.4} / ${:.2} budget ({} files analyzed)",
                            cumulative_cost, self.config.scan_cost_budget, files_analyzed
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to analyze {}: {}", file.display(), e);
                }
            }
        }

        info!(
            "Scan summary: analyzed={}, issues={}, actual_cost=${:.4}, budget_halted={}",
            files_analyzed, issues_found, cumulative_cost, budget_halted
        );

        Ok((files_analyzed, issues_found, budget_halted))
    }

    /// Analyze a single file.
    /// Returns (issues_found, actual_cost_usd). Cache hits cost $0.
    async fn analyze_file(
        &self,
        repo_path: &Path,
        file_path: &Path,
        cache: &RepoCacheSql,
    ) -> Result<(i64, f64)> {
        let rel_path = file_path
            .strip_prefix(repo_path)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        // Skip non-existent files (deleted between diff and analysis)
        if !file_path.exists() {
            debug!("Skipping {} - file no longer exists on disk", rel_path);
            return Ok((0, 0.0));
        }

        // Check file size before reading
        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len();

        if file_size > MAX_ANALYSIS_FILE_SIZE {
            info!(
                "Skipping {} - file too large ({} KB > {} KB limit)",
                rel_path,
                file_size / 1024,
                MAX_ANALYSIS_FILE_SIZE / 1024
            );
            return Ok((0, 0.0));
        }

        if file_size == 0 {
            debug!("Skipping {} - empty file", rel_path);
            return Ok((0, 0.0));
        }

        // Read file content
        let content = match tokio::fs::read_to_string(file_path).await {
            Ok(c) => c,
            Err(e) => {
                warn!("Cannot read {}: {} (possibly binary)", rel_path, e);
                return Ok((0, 0.0));
            }
        };

        // Skip if content is suspiciously dense (likely minified/bundled).
        // Heuristic: if average line length > 500 chars and fewer than 50 lines,
        // it's almost certainly generated or minified code.
        let line_count = content.lines().count().max(1);
        let avg_line_len = content.len() / line_count;
        if avg_line_len > 500 && line_count < 50 {
            info!(
                "Skipping {} - likely minified (avg line: {} chars, {} lines)",
                rel_path, avg_line_len, line_count
            );
            return Ok((0, 0.0));
        }

        // Check cache first
        if cache
            .get(
                crate::repo_cache::CacheType::Refactor,
                &rel_path,
                &content,
                "xai",
                "grok-beta",
                None,
                None,
            )
            .await?
            .is_some()
        {
            debug!("Cache hit for {}", rel_path);
            return Ok((0, 0.0));
        }

        info!("Analyzing {}", rel_path);

        // Create RefactorAssistant for analysis
        let db = Database::from_pool(self.pool.clone());
        let assistant = RefactorAssistant::new(db).await?;

        // Analyze with LLM
        let analysis = assistant.analyze_file(file_path).await?;

        // Calculate actual cost from API-reported tokens_used
        // Uses Grok 4.1 Fast pricing with ~70% input / 30% output split
        // (observed from actual API logs)
        let actual_cost = if let Some(tokens) = analysis.tokens_used {
            let t = tokens as f64;
            let input_est = t * 0.7;
            let output_est = t * 0.3;
            (input_est / 1_000_000.0) * COST_PER_MILLION_INPUT
                + (output_est / 1_000_000.0) * COST_PER_MILLION_OUTPUT
        } else {
            0.0
        };

        // Cache the result
        let result_json = serde_json::to_value(&analysis)?;
        cache
            .set(crate::repo_cache_sql::CacheSetParams {
                cache_type: crate::repo_cache::CacheType::Refactor,
                repo_path: &repo_path.to_string_lossy(),
                file_path: &rel_path,
                content: &content,
                provider: "xai",
                model: "grok-beta",
                result: result_json,
                tokens_used: analysis.tokens_used,
                prompt_hash: None,
                schema_version: None,
            })
            .await?;

        debug!(
            "Cached analysis for {} (cost: ${:.4}, tokens: {:?})",
            rel_path, actual_cost, analysis.tokens_used
        );

        // For now, count any analysis as 1 issue found
        // TODO: Parse analysis.suggestions to count actual issues
        Ok((1, actual_cost))
    }

    /// Update last_scan_check timestamp
    async fn update_last_scan_check(&self, repo_id: &str, timestamp: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE repositories
            SET last_scanned_at = ?
            WHERE id = ?
            "#,
        )
        .bind(timestamp)
        .bind(repo_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update last_analyzed timestamp
    async fn update_last_analyzed(&self, repo_id: &str, timestamp: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE repositories
            SET last_scanned_at = ?
            WHERE id = ?
            "#,
        )
        .bind(timestamp)
        .bind(repo_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update last_commit_hash for a repository
    async fn update_last_commit_hash(&self, repo_id: &str, hash: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE repositories
            SET last_commit_hash = ?
            WHERE id = ?
            "#,
        )
        .bind(hash)
        .bind(repo_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clone scanner for async tasks
    fn clone_scanner(&self) -> Self {
        Self {
            config: self.config.clone(),
            pool: self.pool.clone(),
            repos_dir: self.repos_dir.clone(),
            scan_states: self.scan_states.clone(),
            repo_manager: self.repo_manager.clone(),
        }
    }
}

/// Enable auto-scan for a repository
pub async fn enable_auto_scan(
    pool: &sqlx::SqlitePool,
    repo_id: &str,
    interval_minutes: Option<i64>,
) -> Result<()> {
    let interval = interval_minutes.unwrap_or(60);

    sqlx::query(
        r#"
        UPDATE repositories
        SET auto_scan = 1, scan_interval_mins = ?
        WHERE id = ?
        "#,
    )
    .bind(interval)
    .bind(repo_id)
    .execute(pool)
    .await?;

    info!(
        "Enabled auto-scan for repo {} (interval: {} minutes)",
        repo_id, interval
    );

    Ok(())
}

/// Disable auto-scan for a repository
pub async fn disable_auto_scan(pool: &sqlx::SqlitePool, repo_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE repositories
        SET auto_scan = 0
        WHERE id = ?
        "#,
    )
    .bind(repo_id)
    .execute(pool)
    .await?;

    info!("Disabled auto-scan for repo {}", repo_id);

    Ok(())
}

/// Force a full rescan for a repository (reset both timing AND commit hash)
pub async fn force_scan(pool: &sqlx::SqlitePool, repo_id: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE repositories
        SET last_scanned_at = NULL,
            last_commit_hash = NULL
        WHERE id = ?
        "#,
    )
    .bind(repo_id)
    .execute(pool)
    .await?;

    info!(
        "Forced full rescan for repo {} (cleared commit hash + scan time)",
        repo_id
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AutoScannerConfig::default();
        assert!(config.enabled);
        assert_eq!(config.default_interval_minutes, 60);
        assert_eq!(config.max_concurrent_scans, 2);
        assert!((config.scan_cost_budget - 3.00).abs() < f64::EPSILON);
    }

    #[test]
    fn test_file_status() {
        let status = FileStatus::Modified;
        assert_eq!(status, FileStatus::Modified);
        assert_ne!(status, FileStatus::Unmodified);
    }

    #[test]
    fn test_should_skip_path_skip_dirs() {
        assert!(AutoScanner::should_skip_path(
            "src/clients/web/dist/bundle.js"
        ));
        assert!(AutoScanner::should_skip_path("frontend/build/index.js"));
        assert!(AutoScanner::should_skip_path(
            "node_modules/lodash/index.js"
        ));
        assert!(AutoScanner::should_skip_path("target/debug/build/main.rs"));
        assert!(AutoScanner::should_skip_path("vendor/third_party/lib.go"));
        assert!(AutoScanner::should_skip_path("app/.next/server/pages.js"));
        assert!(AutoScanner::should_skip_path("project/__pycache__/mod.py"));
        assert!(AutoScanner::should_skip_path(".cache/some/file.js"));
    }

    #[test]
    fn test_should_skip_path_skip_suffixes() {
        assert!(AutoScanner::should_skip_path("src/app.min.js"));
        assert!(AutoScanner::should_skip_path("styles/main.min.css"));
        assert!(AutoScanner::should_skip_path("src/index.js.map"));
        assert!(AutoScanner::should_skip_path("src/chunk.bundle.js"));
        assert!(AutoScanner::should_skip_path("src/vendor.chunk.js"));
        assert!(AutoScanner::should_skip_path("lib/types.d.ts"));
        assert!(AutoScanner::should_skip_path("package-lock.lock"));
        assert!(AutoScanner::should_skip_path("src/utils.min.mjs"));
    }

    #[test]
    fn test_should_skip_path_the_offending_file() {
        // THE file that cost $0.14 in one API call
        assert!(AutoScanner::should_skip_path("dist/fks-web-kmp.js"));
        assert!(AutoScanner::should_skip_path(
            "src/clients/web/dist/fks-web-kmp.js"
        ));
    }

    #[test]
    fn test_should_not_skip_normal_code() {
        assert!(!AutoScanner::should_skip_path("src/main.rs"));
        assert!(!AutoScanner::should_skip_path("src/auto_scanner.rs"));
        assert!(!AutoScanner::should_skip_path("lib/utils.js"));
        assert!(!AutoScanner::should_skip_path("scripts/build.sh"));
        assert!(!AutoScanner::should_skip_path("src/components/App.tsx"));
        assert!(!AutoScanner::should_skip_path("cmd/server/main.go"));
    }

    #[test]
    fn test_should_not_skip_distribution_source_code() {
        // "distribution" in a path should NOT be caught by "/dist/" pattern
        assert!(!AutoScanner::should_skip_path("src/distribution/calc.py"));
        assert!(!AutoScanner::should_skip_path("lib/distribution/normal.rs"));
    }

    #[test]
    fn test_should_analyze_file_good_files() {
        assert!(AutoScanner::should_analyze_file("src/main.rs"));
        assert!(AutoScanner::should_analyze_file("lib/app.js"));
        assert!(AutoScanner::should_analyze_file("src/utils.ts"));
        assert!(AutoScanner::should_analyze_file("src/App.tsx"));
        assert!(AutoScanner::should_analyze_file("scripts/deploy.sh"));
        assert!(AutoScanner::should_analyze_file("src/Main.kt"));
        assert!(AutoScanner::should_analyze_file("src/Main.java"));
        assert!(AutoScanner::should_analyze_file("cmd/main.go"));
        assert!(AutoScanner::should_analyze_file("app.py"));
        assert!(AutoScanner::should_analyze_file("lib/helpers.rb"));
    }

    #[test]
    fn test_should_analyze_file_non_code() {
        assert!(!AutoScanner::should_analyze_file("README.md"));
        assert!(!AutoScanner::should_analyze_file("Cargo.toml"));
        assert!(!AutoScanner::should_analyze_file("data.json"));
        assert!(!AutoScanner::should_analyze_file("image.png"));
        assert!(!AutoScanner::should_analyze_file("styles.css"));
        assert!(!AutoScanner::should_analyze_file(".gitignore"));
    }

    #[test]
    fn test_should_analyze_file_code_in_skip_paths() {
        assert!(!AutoScanner::should_analyze_file("dist/bundle.js"));
        assert!(!AutoScanner::should_analyze_file(
            "node_modules/pkg/index.js"
        ));
        assert!(!AutoScanner::should_analyze_file("src/app.min.js"));
        assert!(!AutoScanner::should_analyze_file(
            "src/clients/web/dist/fks-web-kmp.js"
        ));
        assert!(!AutoScanner::should_analyze_file("build/output.js"));
        assert!(!AutoScanner::should_analyze_file("vendor/lib/helper.rb"));
    }

    #[test]
    fn test_is_analyzable_file() {
        assert!(AutoScanner::is_analyzable_file("main.rs"));
        assert!(AutoScanner::is_analyzable_file("script.py"));
        assert!(AutoScanner::is_analyzable_file("app.js"));
        assert!(AutoScanner::is_analyzable_file("component.tsx"));
        assert!(AutoScanner::is_analyzable_file("build.sh"));
        assert!(!AutoScanner::is_analyzable_file("readme.md"));
        assert!(!AutoScanner::is_analyzable_file("config.toml"));
        assert!(!AutoScanner::is_analyzable_file("data.csv"));
    }

    #[test]
    fn test_windows_path_normalization() {
        // Backslash paths should be normalized
        assert!(AutoScanner::should_skip_path(
            "src\\clients\\web\\dist\\bundle.js"
        ));
        assert!(AutoScanner::should_skip_path(
            "node_modules\\lodash\\index.js"
        ));
        assert!(!AutoScanner::should_skip_path("src\\main.rs"));
    }
}
