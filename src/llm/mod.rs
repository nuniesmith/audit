//! LLM module
//!
//! Provides LLM integration for code analysis and content processing.

pub mod compat;
pub mod grok;

// Re-export main types
pub use grok::{
    GrokAnalyzer, ProjectPhase, ProjectPlan, StandardizationIssue, StandardizationReport,
    TodoAnalysis,
};

// Re-export compatibility types
pub use compat::{FileAuditResult, LlmAnalysisResult, LlmClient};
