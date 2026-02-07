//! Research Pipeline Module
//!
//! Extends the audit service to handle content ingestion and synthesis.
//! Processes raw research materials (papers, articles, notes) into structured
//! implementation plans and actionable tasks.

use crate::config::Config;
use crate::error::{AuditError, Result};
use crate::llm::LlmClient;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Represents a structured task extracted from research material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTask {
    pub title: String,
    pub description: String,
    pub complexity: TaskComplexity,
    pub target_component: TargetComponent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskComplexity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetComponent {
    Rust,
    Python,
    Kotlin,
    Infrastructure,
    Documentation,
}

/// Research breakdown result containing the full analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchBreakdown {
    pub title: String,
    pub executive_summary: String,
    pub technical_requirements: String,
    pub architecture_integration: String,
    pub implementation_steps: Vec<String>,
    pub markdown_content: String,
}

/// Analyzes a research file and generates a comprehensive breakdown
pub async fn analyze_file(
    file_path: &Path,
    llm_client: &LlmClient,
    config: &Config,
) -> Result<ResearchBreakdown> {
    info!("📖 Reading research file: {:?}", file_path);

    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))
        .map_err(|e| AuditError::other(e.to_string()))?;

    let file_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("research");

    info!("🤖 Sending to LLM for breakdown (file: {})", file_name);

    let _system_prompt = get_breakdown_prompt(config);
    let _user_prompt = format!("Research Material: {}\n\n{}", file_name, content);

    let response = llm_client
        .analyze_file(file_path, &content, crate::types::Category::Janus)
        .await
        .map_err(|e| AuditError::llm_api(format!("LLM query failed: {}", e)))?;

    // Extract the text response from LlmAnalysisResult
    let response_text = response.summary;

    // Parse the response into structured data
    let breakdown = parse_breakdown_response(&response_text, file_name)?;

    Ok(breakdown)
}

/// Analyzes raw content (not from a file) and generates a breakdown
pub async fn analyze_content(
    content: &str,
    title: &str,
    llm_client: &LlmClient,
    config: &Config,
) -> Result<ResearchBreakdown> {
    info!("🤖 Analyzing research content: {}", title);

    let _system_prompt = get_breakdown_prompt(config);
    let _user_prompt = format!("Research Material: {}\n\n{}", title, content);

    // For content analysis, we create a temporary path
    let temp_path = std::path::PathBuf::from(format!("research/{}.md", title));

    let response = llm_client
        .analyze_file(&temp_path, content, crate::types::Category::Janus)
        .await
        .map_err(|e| AuditError::llm_api(format!("LLM query failed: {}", e)))?;

    let response_text = response.summary;

    let breakdown = parse_breakdown_response(&response_text, title)?;

    Ok(breakdown)
}

/// Extracts structured tasks from research content
pub async fn extract_tasks(
    content: &str,
    llm_client: &LlmClient,
    config: &Config,
) -> Result<Vec<ResearchTask>> {
    info!("📋 Extracting actionable tasks from research material");

    let _system_prompt = get_task_extraction_prompt(config);

    // For task extraction, we use a temporary path and pass the content
    let temp_path = std::path::PathBuf::from("research/tasks.md");

    let response = llm_client
        .analyze_file(&temp_path, content, crate::types::Category::Janus)
        .await
        .map_err(|e| AuditError::llm_api(format!("Task extraction failed: {}", e)))?;

    let response_text = response.summary;

    // Clean potential markdown fencing from LLM response
    let clean_json = response_text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let tasks: Vec<ResearchTask> = serde_json::from_str(clean_json).map_err(|e| {
        warn!("Failed to parse task JSON. Response: {}", clean_json);
        AuditError::other(format!("Failed to parse LLM JSON response: {}", e))
    })?;

    info!("✅ Extracted {} tasks", tasks.len());
    Ok(tasks)
}

/// Saves a research breakdown to a markdown file
pub fn save_breakdown(
    breakdown: &ResearchBreakdown,
    output_dir: &Path,
    original_filename: Option<&str>,
) -> Result<PathBuf> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {:?}", output_dir))
        .map_err(|e| AuditError::other(e.to_string()))?;

    let filename = if let Some(orig) = original_filename {
        format!("{}_PLAN.md", orig)
    } else {
        format!("{}_PLAN.md", sanitize_filename(&breakdown.title))
    };

    let output_path = output_dir.join(filename);

    fs::write(&output_path, &breakdown.markdown_content)
        .with_context(|| format!("Failed to write breakdown to: {:?}", output_path))
        .map_err(|e| AuditError::other(e.to_string()))?;

    info!("✅ Saved research breakdown to: {:?}", output_path);
    Ok(output_path)
}

/// Saves extracted tasks to a JSON file
pub fn save_tasks(tasks: &[ResearchTask], output_path: &Path) -> Result<PathBuf> {
    // Create parent directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))
            .map_err(|e| AuditError::other(e.to_string()))?;
    }

    let json = serde_json::to_string_pretty(tasks)
        .map_err(|e| AuditError::other(format!("Failed to serialize tasks: {}", e)))?;

    fs::write(output_path, json)
        .with_context(|| format!("Failed to write tasks to: {:?}", output_path))
        .map_err(|e| AuditError::other(e.to_string()))?;

    info!("✅ Saved {} tasks to: {:?}", tasks.len(), output_path);
    Ok(output_path.to_path_buf())
}

/// Gets the breakdown system prompt from config or uses default
fn get_breakdown_prompt(config: &Config) -> String {
    config
        .research
        .as_ref()
        .and_then(|r| r.prompts.get("breakdown"))
        .cloned()
        .unwrap_or_else(default_breakdown_prompt)
}

/// Gets the task extraction prompt from config or uses default
fn get_task_extraction_prompt(config: &Config) -> String {
    config
        .research
        .as_ref()
        .and_then(|r| r.prompts.get("tasks"))
        .cloned()
        .unwrap_or_else(default_task_extraction_prompt)
}

/// Default breakdown system prompt
fn default_breakdown_prompt() -> String {
    r#"You are the Lead Architect for Project JANUS. You are reviewing raw research notes or a technical paper.
Your goal is to synthesize this information into a concrete implementation plan for the JANUS system.

Context:
- Core 'Muscle' (Execution): Rust (high-performance trading execution, data processing)
- Core 'Brain' (AI/Logic): Python (strategy development, machine learning)
- Frontend: Kotlin Multiplatform (mobile and desktop interfaces)
- Infrastructure: Docker, PostgreSQL, Redis, message queues

Output a Markdown response with the following sections:

# [Topic Name] - Implementation Plan

## 1. Executive Summary
Briefly explain what this is and why it adds value to JANUS.

## 2. Technical Requirements
- Data requirements (feeds, storage, schemas)
- Performance constraints (latency, throughput)
- Dependencies (libraries, external systems)
- Security considerations

## 3. Architecture Integration
### Rust Components
Specific structs, traits, modules, or services needed in the Rust execution layer.

### Python Components
Specific scripts, models, dataframes, or algorithms needed in the Python brain layer.

### Infrastructure Changes
Database schemas, message queue topics, caching strategies, etc.

## 4. Implementation Steps
Provide a numbered list of concrete implementation steps, ordered by dependencies.

## 5. Risks and Mitigations
Potential technical challenges and how to address them.

Be specific and technical. Reference actual JANUS architecture patterns when possible."#.to_string()
}

/// Default task extraction prompt
fn default_task_extraction_prompt() -> String {
    r#"Analyze the provided text and extract a list of actionable coding tasks.
Return ONLY a valid JSON array of objects. Do not include markdown code fences.

Each task should have:
- title: Short, clear title (max 60 chars)
- description: Detailed technical description with specific implementation details
- complexity: One of "low", "medium", or "high"
- target_component: One of "Rust", "Python", "Kotlin", "Infrastructure", or "Documentation"
- estimated_hours: Optional estimated hours to complete (integer)
- dependencies: Optional array of task titles this depends on

Example format:
[
  {
    "title": "Implement WebSocket price feed handler",
    "description": "Create a Rust async WebSocket client to consume real-time price data from Binance. Use tokio-tungstenite. Parse JSON messages and publish to internal message queue.",
    "complexity": "medium",
    "target_component": "Rust",
    "estimated_hours": 8,
    "dependencies": []
  }
]

Focus on tasks that are:
- Specific and actionable
- Have clear technical requirements
- Can be completed independently (or with minimal dependencies)
- Align with the JANUS architecture (Rust for execution, Python for intelligence)"#.to_string()
}

/// Parses the LLM breakdown response into structured data
fn parse_breakdown_response(response: &str, title: &str) -> Result<ResearchBreakdown> {
    // For now, we'll do a simple section-based parse
    // In a production system, you might use more sophisticated NLP or ask the LLM
    // to return structured JSON

    let sections = extract_sections(response);

    Ok(ResearchBreakdown {
        title: title.to_string(),
        executive_summary: sections
            .get("executive_summary")
            .or_else(|| sections.get("executive summary"))
            .unwrap_or(&"See full breakdown".to_string())
            .clone(),
        technical_requirements: sections
            .get("technical_requirements")
            .or_else(|| sections.get("technical requirements"))
            .unwrap_or(&String::new())
            .clone(),
        architecture_integration: sections
            .get("architecture_integration")
            .or_else(|| sections.get("architecture integration"))
            .unwrap_or(&String::new())
            .clone(),
        implementation_steps: extract_implementation_steps(&sections),
        markdown_content: response.to_string(),
    })
}

/// Extracts markdown sections from response
fn extract_sections(markdown: &str) -> std::collections::HashMap<String, String> {
    let mut sections = std::collections::HashMap::new();
    let mut current_section = String::new();
    let mut current_content = Vec::new();

    for line in markdown.lines() {
        if line.starts_with("##") {
            // Save previous section
            if !current_section.is_empty() {
                let key = current_section
                    .trim_start_matches('#')
                    .trim()
                    .to_lowercase()
                    .replace(' ', "_");
                sections.insert(key, current_content.join("\n"));
            }

            current_section = line.to_string();
            current_content.clear();
        } else if !current_section.is_empty() {
            current_content.push(line.to_string());
        }
    }

    // Save last section
    if !current_section.is_empty() {
        let key = current_section
            .trim_start_matches('#')
            .trim()
            .to_lowercase()
            .replace(' ', "_");
        sections.insert(key, current_content.join("\n"));
    }

    sections
}

/// Extracts implementation steps from parsed sections
fn extract_implementation_steps(
    sections: &std::collections::HashMap<String, String>,
) -> Vec<String> {
    let steps_text = sections
        .get("implementation_steps")
        .or_else(|| sections.get("implementation steps"))
        .cloned()
        .unwrap_or_default();

    steps_text
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("1.")
                || trimmed.starts_with("2.")
                || trimmed.starts_with('-')
                || trimmed.starts_with('*')
        })
        .map(|s| s.trim().to_string())
        .collect()
}

/// Sanitizes a filename by removing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test/file.md"), "test_file.md");
        assert_eq!(sanitize_filename("valid_name"), "valid_name");
        assert_eq!(sanitize_filename("test: value"), "test_ value");
    }

    #[test]
    fn test_extract_sections() {
        let markdown = r#"# Title
## Section 1
Content 1
## Section 2
Content 2
More content"#;

        let sections = extract_sections(markdown);
        assert!(sections.contains_key("section_1"));
        assert!(sections.contains_key("section_2"));
    }

    #[test]
    fn test_parse_breakdown_response() {
        let response = r#"# Research Topic
## Executive Summary
This is a test summary.
## Technical Requirements
- Requirement 1
- Requirement 2"#;

        let breakdown = parse_breakdown_response(response, "test").unwrap();
        assert_eq!(breakdown.title, "test");
        assert!(breakdown.executive_summary.contains("test summary"));
    }
}
