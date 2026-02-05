//! Database Configuration
//!
//! Handles database paths, permissions, and initialization across environments.
//! Solves the jordan/actions user permission issues.

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{info, warn};

// ============================================================================
// Configuration
// ============================================================================

/// Database configuration loaded from environment
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file
    pub path: PathBuf,
    /// Whether to run migrations on startup
    pub auto_migrate: bool,
    /// Maximum connections in pool
    pub max_connections: u32,
    /// Whether this is a development environment
    pub is_dev: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: get_default_db_path(),
            auto_migrate: true,
            max_connections: 5,
            is_dev: cfg!(debug_assertions),
        }
    }
}

impl DatabaseConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let path = std::env::var("RUSTASSISTANT_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| get_default_db_path());

        let auto_migrate = std::env::var("RUSTASSISTANT_AUTO_MIGRATE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);

        let max_connections = std::env::var("RUSTASSISTANT_DB_MAX_CONN")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        let is_dev = std::env::var("RUSTASSISTANT_ENV")
            .map(|v| v == "development" || v == "dev")
            .unwrap_or_else(|_| cfg!(debug_assertions));

        Self {
            path,
            auto_migrate,
            max_connections,
            is_dev,
        }
    }
}

// ============================================================================
// Path Resolution
// ============================================================================

/// Get the default database path based on environment
fn get_default_db_path() -> PathBuf {
    // Priority:
    // 1. RUSTASSISTANT_DB_PATH env var (handled in from_env)
    // 2. XDG_DATA_HOME/rustassistant/rustassistant.db
    // 3. ~/.local/share/rustassistant/rustassistant.db
    // 4. ./data/rustassistant.db (development fallback)

    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(data_home)
            .join("rustassistant")
            .join("rustassistant.db");
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("rustassistant")
            .join("rustassistant.db");
    }

    // Development fallback
    PathBuf::from("./data/rustassistant.db")
}

/// Get the data directory (parent of db file)
pub fn get_data_dir(config: &DatabaseConfig) -> PathBuf {
    config
        .path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("./data"))
}

// ============================================================================
// Directory Setup
// ============================================================================

/// Ensure the data directory exists with correct permissions
pub fn ensure_data_dir(config: &DatabaseConfig) -> Result<()> {
    let dir = get_data_dir(config);

    if !dir.exists() {
        info!("Creating data directory: {}", dir.display());
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create data directory: {}", dir.display()))?;
    }

    // On Unix, ensure directory is writable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&dir)?;
        let mut perms = metadata.permissions();

        // Ensure owner can read/write/execute (0o700 minimum)
        let current_mode = perms.mode();
        if current_mode & 0o700 != 0o700 {
            perms.set_mode(current_mode | 0o700);
            std::fs::set_permissions(&dir, perms)?;
            info!("Updated directory permissions for: {}", dir.display());
        }
    }

    Ok(())
}

// ============================================================================
// Pool Creation
// ============================================================================

/// Initialize the database connection pool
pub async fn init_pool(config: &DatabaseConfig) -> Result<SqlitePool> {
    // Ensure directory exists
    ensure_data_dir(config)?;

    let db_url = format!("sqlite:{}?mode=rwc", config.path.display());
    info!("Connecting to database: {}", config.path.display());

    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .synchronous(SqliteSynchronous::Normal)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(30));

    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .connect_with(options)
        .await
        .with_context(|| format!("Failed to connect to database: {}", config.path.display()))?;

    // Run migrations if enabled
    if config.auto_migrate {
        run_migrations(&pool).await?;
    }

    // Fix permissions on the database file itself
    #[cfg(unix)]
    fix_db_permissions(&config.path)?;

    Ok(pool)
}

/// Run database migrations
async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    info!("Running database migrations...");

    // Using sqlx-cli migrations
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("Failed to run migrations")?;

    info!("Migrations complete");
    Ok(())
}

/// Fix database file permissions on Unix
#[cfg(unix)]
fn fix_db_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    if path.exists() {
        let metadata = std::fs::metadata(path)?;
        let mut perms = metadata.permissions();

        // Set to 0o644 (owner read/write, group/others read)
        // This allows both jordan and actions users to read
        let desired_mode = 0o664;
        if perms.mode() & 0o777 != desired_mode {
            perms.set_mode(desired_mode);
            std::fs::set_permissions(path, perms)?;
            info!("Updated database file permissions: {}", path.display());
        }
    }

    // Also fix WAL and SHM files if they exist
    let wal_path = path.with_extension("db-wal");
    let shm_path = path.with_extension("db-shm");

    for extra_path in [wal_path, shm_path] {
        if extra_path.exists() {
            let metadata = std::fs::metadata(&extra_path)?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o664);
            std::fs::set_permissions(&extra_path, perms).ok(); // Ignore errors
        }
    }

    Ok(())
}

#[cfg(not(unix))]
fn fix_db_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

// ============================================================================
// Environment Variables Documentation
// ============================================================================

/// Print help for database-related environment variables
pub fn print_env_help() {
    println!(
        r#"
Database Environment Variables:
===============================

RUSTASSISTANT_DB_PATH
    Path to the SQLite database file.
    Default (Linux): ~/.local/share/rustassistant/rustassistant.db
    Default (dev):   ./data/rustassistant.db
    
RUSTASSISTANT_AUTO_MIGRATE
    Run migrations on startup. Values: true, false, 1, 0
    Default: true
    
RUSTASSISTANT_DB_MAX_CONN
    Maximum database connections in pool.
    Default: 5
    
RUSTASSISTANT_ENV
    Environment mode. Values: development, dev, production, prod
    Default: development (debug builds), production (release builds)

Example Usage:
--------------

Development:
    export RUSTASSISTANT_DB_PATH="./data/dev.db"
    export RUSTASSISTANT_ENV="development"
    cargo run

Production (systemd):
    [Service]
    Environment="RUSTASSISTANT_DB_PATH=/var/lib/rustassistant/rustassistant.db"
    Environment="RUSTASSISTANT_ENV=production"
    
GitHub Actions:
    - name: Setup database
      run: |
        mkdir -p /var/lib/rustassistant
        chmod 775 /var/lib/rustassistant
      env:
        RUSTASSISTANT_DB_PATH: /var/lib/rustassistant/rustassistant.db
"#
    );
}

// ============================================================================
// Backup Utilities
// ============================================================================

/// Create a backup of the database
pub async fn backup_database(pool: &SqlitePool, backup_path: &Path) -> Result<()> {
    // Ensure backup directory exists
    if let Some(parent) = backup_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Use SQLite's backup API via VACUUM INTO
    let backup_url = backup_path.display().to_string();
    sqlx::query(&format!("VACUUM INTO '{}'", backup_url))
        .execute(pool)
        .await
        .context("Failed to create database backup")?;

    info!("Database backed up to: {}", backup_path.display());
    Ok(())
}

/// Get backup path with timestamp
pub fn get_backup_path(config: &DatabaseConfig) -> PathBuf {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let data_dir = get_data_dir(config);
    data_dir.join("backups").join(format!("rustassistant_{}.db", timestamp))
}

// ============================================================================
// Health Check
// ============================================================================

/// Check database health
pub async fn health_check(pool: &SqlitePool) -> Result<DatabaseHealth> {
    let start = std::time::Instant::now();

    // Simple query to verify connection
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await
        .context("Database health check failed")?;

    let latency = start.elapsed();

    // Get some stats
    let task_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tasks")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    Ok(DatabaseHealth {
        connected: result.0 == 1,
        latency_ms: latency.as_millis() as u64,
        task_count: task_count.0,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub latency_ms: u64,
    pub task_count: i64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert!(config.auto_migrate);
        assert_eq!(config.max_connections, 5);
    }

    #[test]
    fn test_config_from_env() {
        std::env::set_var("RUSTASSISTANT_DB_PATH", "/tmp/test.db");
        std::env::set_var("RUSTASSISTANT_AUTO_MIGRATE", "false");

        let config = DatabaseConfig::from_env();
        assert_eq!(config.path, PathBuf::from("/tmp/test.db"));
        assert!(!config.auto_migrate);

        // Cleanup
        std::env::remove_var("RUSTASSISTANT_DB_PATH");
        std::env::remove_var("RUSTASSISTANT_AUTO_MIGRATE");
    }
}
