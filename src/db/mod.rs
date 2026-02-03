//! Database module
//!
//! Provides database operations for notes, repositories, tasks, and queue system.

pub mod core;
pub mod queue;

// Re-export core database types and functions
pub use core::*;

// Re-export queue types and functions
pub use queue::{
    create_queue_tables, FileAnalysis, QueueItem, QueuePriority, QueueSource, QueueStage,
    RepoCache, TodoItem, GITHUB_USERNAME,
};
