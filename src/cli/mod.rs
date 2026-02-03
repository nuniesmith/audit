//! CLI module
//!
//! Provides command-line interface functionality for queue, scan, and report operations.

pub mod queue_commands;

// Re-export command types
pub use queue_commands::{
    handle_queue_command, handle_report_command, handle_scan_command, QueueCommands,
    ReportCommands, ScanCommands,
};
