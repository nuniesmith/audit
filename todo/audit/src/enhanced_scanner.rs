//! Enhanced scanner with test running and deep context analysis

use crate::context::{ContextBuilder, GlobalContextBundle};
use crate::error::Result;
use crate::llm::{FileAuditResult, LlmClient};
use crate::scanner::Scanner;
use crate::tests_runner::{TestResults, TestRunner};
use crate::types::{AuditReport, AuditRequest, AuditSummary, Task, TaskPriority};
use std::path::PathBuf;
use tracing::{info, warn};

/// Enhanced scanner with test running and 2M context window analysis
pub struct EnhancedScanner {
    /// Base scanner
    scanner: Scanner,
    /// Root directory
    #[allow(dead_code)]
    root: PathBuf,
    /// Test runner
    test_runner: TestRunner,
    /// Context builder
    context_builder: ContextBuilder,
    /// LLM client (optional)
    llm_client: Option<LlmClient>,
    /// Whether to run tests
    run_tests: bool,
    /// Whether to use deep analysis
    use_deep_analysis: bool,
}

impl EnhancedScanner {
    /// Create a new enhanced scanner
    pub fn new(
        root: PathBuf,
        max_file_size: usize,
        include_tests: bool,
        llm_client: Option<LlmClient>,
    ) -> Result<Self> {
        let scanner = Scanner::new(root.clone(), max_file_size, include_tests)?;
        let test_runner = TestRunner::new(root.clone());
        let context_builder = ContextBuilder::new(root.clone())
            .with_tests(include_tests)
            .with_max_file_size(max_file_size);

        let use_deep_analysis = llm_client.is_some();

        Ok(Self {
            scanner,
            root,
            test_runner,
            context_builder,
            llm_client,
            run_tests: include_tests,
            use_deep_analysis,
        })
    }

    /// Set whether to run tests
    pub fn with_run_tests(mut self, run: bool) -> Self {
        self.run_tests = run;
        self
    }

    /// Set whether to use deep analysis
    pub fn with_deep_analysis(mut self, use_deep: bool) -> Self {
        self.use_deep_analysis = use_deep;
        self
    }

    /// Run complete audit with all features
    pub async fn run_complete_audit(&self, request: &AuditRequest) -> Result<AuditReport> {
        info!("Starting enhanced audit with test running and deep analysis");

        // Step 1: Run base scanner for static analysis
        info!("Step 1: Running static analysis...");
        let mut report = self.scanner.scan(request)?;

        // Step 2: Run tests if enabled
        let mut test_results = None;
        if self.run_tests || request.include_tests {
            info!("Step 2: Running project tests...");
            match self.run_tests() {
                Ok(results) => {
                    self.update_summary_with_tests(&mut report.summary, &results);
                    test_results = Some(results);
                }
                Err(e) => {
                    warn!("Failed to run tests: {}", e);
                }
            }
        }

        // Step 3: Build global context bundle
        info!("Step 3: Building global context bundle...");
        let context_bundle = match self.build_context_bundle(report.system_map.clone()) {
            Ok(bundle) => {
                info!(
                    "Context bundle built: {} symbols, {} files",
                    bundle.signature_map.total_symbols,
                    bundle.source_bundle.files.len()
                );
                Some(bundle)
            }
            Err(e) => {
                warn!("Failed to build context bundle: {}", e);
                None
            }
        };

        // Step 4: Deep analysis with LLM if enabled
        if self.use_deep_analysis && request.enable_llm {
            if let Some(ref bundle) = context_bundle {
                if let Some(ref llm) = self.llm_client {
                    info!("Step 4: Running deep analysis with 2M context window...");
                    match self.run_deep_analysis(llm, bundle).await {
                        Ok(tasks) => {
                            report.tasks.extend(tasks);
                            info!("Generated {} tasks from deep analysis", report.tasks.len());
                        }
                        Err(e) => {
                            warn!("Deep analysis failed: {}", e);
                        }
                    }

                    // Run standard questionnaire for all files
                    info!("Step 4b: Running standard questionnaire...");
                    match self.run_standard_questionnaire(llm, bundle).await {
                        Ok(file_audits) => {
                            let additional_tasks = self.generate_tasks_from_audits(&file_audits);
                            report.tasks.extend(additional_tasks);
                            info!("Generated {} tasks from questionnaire", file_audits.len());
                        }
                        Err(e) => {
                            warn!("Standard questionnaire failed: {}", e);
                        }
                    }
                }
            }
        }

        // Add test results and context bundle to report
        report.test_results = test_results;
        report.context_bundle = context_bundle;

        // Update final summary
        report.summary.total_tasks = report.tasks.len();

        info!(
            "Enhanced audit complete: {} files, {} issues, {} tasks, {} tests",
            report.summary.total_files,
            report.summary.total_issues,
            report.summary.total_tasks,
            report.summary.total_tests.unwrap_or(0)
        );

        Ok(report)
    }

    /// Run all tests in the project
    fn run_tests(&self) -> Result<Vec<TestResults>> {
        info!("Discovering and running tests...");
        let results = self.test_runner.run_all_tests()?;

        for result in &results {
            info!(
                "{:?} tests: {} total, {} passed, {} failed, {} skipped",
                result.project_type, result.total, result.passed, result.failed, result.skipped
            );
            if let Some(coverage) = result.coverage {
                info!("  Coverage: {:.1}%", coverage);
            }
        }

        Ok(results)
    }

    /// Build global context bundle
    fn build_context_bundle(
        &self,
        system_map: crate::types::SystemMap,
    ) -> Result<GlobalContextBundle> {
        self.context_builder.build(system_map)
    }

    /// Run deep analysis with LLM using the 2M context window
    async fn run_deep_analysis(
        &self,
        llm: &LlmClient,
        bundle: &GlobalContextBundle,
    ) -> Result<Vec<Task>> {
        // Format the global context for the LLM
        let formatted_context = ContextBuilder::format_for_llm(bundle);

        info!(
            "Sending {} bytes of context to Grok 4.1",
            formatted_context.len()
        );

        // Run deep analysis
        let analysis = llm.analyze_with_global_context(&formatted_context).await?;

        // Convert analysis results to tasks
        let mut tasks = Vec::new();

        // Logic drift issues
        for issue in &analysis.logic_drift {
            tasks.push(
                Task::new(
                    format!("Logic Drift: {}", issue.category),
                    issue.description.clone(),
                    PathBuf::from(&issue.file),
                    issue.line,
                    TaskPriority::High,
                    crate::types::Category::from_path(&issue.file),
                )
                .with_tag("logic-drift"),
            );
        }

        // Dead code
        for file in &analysis.dead_code {
            tasks.push(
                Task::new(
                    "Dead Code Detected",
                    format!("File appears to be unused: {}", file),
                    PathBuf::from(file),
                    None,
                    TaskPriority::Medium,
                    crate::types::Category::from_path(file),
                )
                .with_tag("dead-code"),
            );
        }

        // Safety issues
        for issue in &analysis.safety_issues {
            let priority = match issue.severity.to_lowercase().as_str() {
                "critical" => TaskPriority::Critical,
                "high" => TaskPriority::High,
                "medium" => TaskPriority::Medium,
                _ => TaskPriority::Low,
            };

            tasks.push(
                Task::new(
                    format!("Safety Issue: {}", issue.category),
                    issue.description.clone(),
                    PathBuf::from(&issue.file),
                    issue.line,
                    priority,
                    crate::types::Category::from_path(&issue.file),
                )
                .with_tag("safety"),
            );
        }

        // Incomplete code
        for issue in &analysis.incomplete_code {
            tasks.push(
                Task::new(
                    "Incomplete Implementation",
                    issue.description.clone(),
                    PathBuf::from(&issue.file),
                    issue.line,
                    TaskPriority::Medium,
                    crate::types::Category::from_path(&issue.file),
                )
                .with_tag("incomplete"),
            );
        }

        // Add LLM-generated tasks
        for generated in &analysis.tasks {
            let priority = match generated.priority.to_lowercase().as_str() {
                "critical" => TaskPriority::Critical,
                "high" => TaskPriority::High,
                "medium" => TaskPriority::Medium,
                _ => TaskPriority::Low,
            };

            let mut task = Task::new(
                generated.category.clone(),
                generated.description.clone(),
                PathBuf::from(&generated.file),
                generated.line,
                priority,
                crate::types::Category::from_path(&generated.file),
            );

            if let Some(ref tag) = generated.suggested_tag {
                task = task.with_tag(tag);
            }

            tasks.push(task);
        }

        Ok(tasks)
    }

    /// Run standard questionnaire for all files
    async fn run_standard_questionnaire(
        &self,
        llm: &LlmClient,
        bundle: &GlobalContextBundle,
    ) -> Result<Vec<FileAuditResult>> {
        let formatted_context = ContextBuilder::format_for_llm(bundle);
        let file_paths: Vec<String> = bundle
            .source_bundle
            .files
            .iter()
            .map(|f| f.path.clone())
            .collect();

        llm.run_standard_questionnaire(&formatted_context, &file_paths)
            .await
    }

    /// Generate tasks from file audit results
    fn generate_tasks_from_audits(&self, audits: &[FileAuditResult]) -> Vec<Task> {
        let mut tasks = Vec::new();

        for audit in audits {
            // Create task for unreachable files
            if !audit.reachable {
                tasks.push(
                    Task::new(
                        "Unreachable Code",
                        format!("File is not imported or used: {}", audit.file),
                        PathBuf::from(&audit.file),
                        None,
                        TaskPriority::Low,
                        crate::types::Category::from_path(&audit.file),
                    )
                    .with_tag("legacy"),
                );
            }

            // Create tasks for compliance issues
            for issue in &audit.compliance_issues {
                tasks.push(
                    Task::new(
                        "Compliance Issue",
                        issue.clone(),
                        PathBuf::from(&audit.file),
                        None,
                        TaskPriority::High,
                        crate::types::Category::from_path(&audit.file),
                    )
                    .with_tag("compliance"),
                );
            }

            // Create task for incomplete code
            if audit.incomplete {
                tasks.push(
                    Task::new(
                        "Incomplete Implementation",
                        format!("File has incomplete code: {}", audit.file),
                        PathBuf::from(&audit.file),
                        None,
                        TaskPriority::Medium,
                        crate::types::Category::from_path(&audit.file),
                    )
                    .with_tag("incomplete"),
                );
            }

            // Create improvement task if suggested
            if !audit.improvement.is_empty() {
                tasks.push(
                    Task::new(
                        "Improvement Suggested",
                        audit.improvement.clone(),
                        PathBuf::from(&audit.file),
                        None,
                        TaskPriority::Low,
                        crate::types::Category::from_path(&audit.file),
                    )
                    .with_tag("improvement"),
                );
            }
        }

        tasks
    }

    /// Update summary with test results
    fn update_summary_with_tests(&self, summary: &mut AuditSummary, results: &[TestResults]) {
        let total_tests: usize = results.iter().map(|r| r.total).sum();
        let total_passed: usize = results.iter().map(|r| r.passed).sum();

        let pass_rate = if total_tests > 0 {
            (total_passed as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        let avg_coverage = results.iter().filter_map(|r| r.coverage).sum::<f64>()
            / results
                .iter()
                .filter(|r| r.coverage.is_some())
                .count()
                .max(1) as f64;

        summary.total_tests = Some(total_tests);
        summary.test_pass_rate = Some(pass_rate);
        summary.code_coverage = if avg_coverage > 0.0 {
            Some(avg_coverage)
        } else {
            None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_scanner_creation() {
        let scanner = EnhancedScanner::new(PathBuf::from("."), 1_000_000, false, None);
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_with_options() {
        let scanner = EnhancedScanner::new(PathBuf::from("."), 1_000_000, false, None)
            .unwrap()
            .with_run_tests(true)
            .with_deep_analysis(false);

        assert!(scanner.run_tests);
        assert!(!scanner.use_deep_analysis);
    }
}
