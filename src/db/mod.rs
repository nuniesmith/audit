//! Database module
//!
//! Provides database operations for notes, repositories, tasks, and queue system.

pub mod config;
pub mod core;
pub mod queue;

// Re-export configuration types and functions
pub use config::{
    backup_database, ensure_data_dir, get_backup_path, get_data_dir, health_check, init_pool,
    print_env_help, DatabaseConfig, DatabaseHealth,
};

// Re-export core database types and functions
pub use core::*;

// Re-export queue types and functions
pub use queue::{
    create_queue_tables, FileAnalysis, QueueItem, QueuePriority, QueueSource, QueueStage,
    RepoCache, TodoItem, GITHUB_USERNAME,
};
