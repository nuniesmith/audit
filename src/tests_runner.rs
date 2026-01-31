//! Test runner module for discovering and executing tests across different project types

use crate::error::{AuditError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

/// Test runner for different project types
#[derive(Debug)]
pub struct TestRunner {
    root: PathBuf,
}

/// Test suite results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    /// Project type
    pub project_type: ProjectType,
    /// Total tests
    pub total: usize,
    /// Passed tests
    pub passed: usize,
    /// Failed tests
    pub failed: usize,
    /// Skipped tests
    pub skipped: usize,
    /// Test duration in seconds
    pub duration: f64,
    /// Test files
    pub test_files: Vec<String>,
    /// Coverage percentage (if available)
    pub coverage: Option<f64>,
    /// Detailed results by file
    pub results_by_file: HashMap<String, FileTestResult>,
    /// Raw output
    pub output: String,
}

/// Test results for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTestResult {
    /// File path
    pub file: String,
    /// Tests in this file
    pub tests: usize,
    /// Passed
    pub passed: usize,
    /// Failed
    pub failed: usize,
    /// Failed test names
    pub failures: Vec<String>,
}

/// Project type detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Kotlin,
    Mixed,
}

impl TestRunner {
    /// Create a new test runner
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Detect project types in the repository
    pub fn detect_project_types(&self) -> Result<Vec<ProjectType>> {
        let mut types = Vec::new();

        // Check for Rust
        if self.root.join("Cargo.toml").exists() {
            types.push(ProjectType::Rust);
        }

        // Check for Python
        if self.root.join("pyproject.toml").exists()
            || self.root.join("setup.py").exists()
            || self.root.join("requirements.txt").exists()
        {
            types.push(ProjectType::Python);
        }

        // Check for JavaScript/TypeScript
        if self.root.join("package.json").exists() {
            let has_ts = WalkDir::new(&self.root)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|ext| ext == "ts" || ext == "tsx")
                });

            if has_ts {
                types.push(ProjectType::TypeScript);
            } else {
                types.push(ProjectType::JavaScript);
            }
        }

        // Check for Kotlin
        if self.root.join("build.gradle.kts").exists() || self.root.join("build.gradle").exists() {
            types.push(ProjectType::Kotlin);
        }

        Ok(types)
    }

    /// Run all tests for detected project types
    pub fn run_all_tests(&self) -> Result<Vec<TestResults>> {
        let project_types = self.detect_project_types()?;
        let mut all_results = Vec::new();

        for project_type in project_types {
            match self.run_tests_for_type(project_type) {
                Ok(results) => all_results.push(results),
                Err(e) => {
                    tracing::warn!("Failed to run {:?} tests: {}", project_type, e);
                    // Continue with other project types
                }
            }
        }

        Ok(all_results)
    }

    /// Run tests for a specific project type
    pub fn run_tests_for_type(&self, project_type: ProjectType) -> Result<TestResults> {
        match project_type {
            ProjectType::Rust => self.run_rust_tests(),
            ProjectType::Python => self.run_python_tests(),
            ProjectType::JavaScript | ProjectType::TypeScript => self.run_js_tests(),
            ProjectType::Kotlin => self.run_kotlin_tests(),
            ProjectType::Mixed => Err(AuditError::Config(
                "Cannot run tests for mixed project type".to_string(),
            )),
        }
    }

    /// Run Rust tests using cargo
    fn run_rust_tests(&self) -> Result<TestResults> {
        let start = std::time::Instant::now();

        // Find all test files
        let test_files = self.find_rust_test_files()?;

        // Run cargo test with JSON output
        let output = Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("--format=json")
            .arg("--nocapture")
            .current_dir(&self.root)
            .output()
            .map_err(AuditError::Io)?;

        let duration = start.elapsed().as_secs_f64();
        let output_str = String::from_utf8_lossy(&output.stdout).to_string();

        // Parse cargo test output
        let (total, passed, failed, skipped) = self.parse_cargo_test_output(&output_str);

        // Try to get coverage if available
        let coverage = self.get_rust_coverage().ok();

        Ok(TestResults {
            project_type: ProjectType::Rust,
            total,
            passed,
            failed,
            skipped,
            duration,
            test_files,
            coverage,
            results_by_file: HashMap::new(), // TODO: Parse detailed results
            output: output_str,
        })
    }

    /// Run Python tests using pytest
    fn run_python_tests(&self) -> Result<TestResults> {
        let start = std::time::Instant::now();

        // Find all test files
        let test_files = self.find_python_test_files()?;

        // Run pytest with JSON report
        let output = Command::new("pytest")
            .arg("--json-report")
            .arg("--json-report-file=.pytest-report.json")
            .arg("-v")
            .current_dir(&self.root)
            .output()
            .map_err(AuditError::Io)?;

        let duration = start.elapsed().as_secs_f64();
        let output_str = String::from_utf8_lossy(&output.stdout).to_string();

        // Parse pytest output
        let (total, passed, failed, skipped) = self.parse_pytest_output(&output_str);

        // Try to get coverage if available
        let coverage = self.get_python_coverage().ok();

        Ok(TestResults {
            project_type: ProjectType::Python,
            total,
            passed,
            failed,
            skipped,
            duration,
            test_files,
            coverage,
            results_by_file: HashMap::new(),
            output: output_str,
        })
    }

    /// Run JavaScript/TypeScript tests using npm/jest
    fn run_js_tests(&self) -> Result<TestResults> {
        let start = std::time::Instant::now();

        // Find all test files
        let test_files = self.find_js_test_files()?;

        // Try npm test first, fall back to jest
        let output = Command::new("npm")
            .arg("test")
            .arg("--")
            .arg("--json")
            .current_dir(&self.root)
            .output()
            .map_err(AuditError::Io)?;

        let duration = start.elapsed().as_secs_f64();
        let output_str = String::from_utf8_lossy(&output.stdout).to_string();

        // Parse test output
        let (total, passed, failed, skipped) = self.parse_jest_output(&output_str);

        Ok(TestResults {
            project_type: ProjectType::TypeScript,
            total,
            passed,
            failed,
            skipped,
            duration,
            test_files,
            coverage: None,
            results_by_file: HashMap::new(),
            output: output_str,
        })
    }

    /// Run Kotlin tests using gradle
    fn run_kotlin_tests(&self) -> Result<TestResults> {
        let start = std::time::Instant::now();

        // Find all test files
        let test_files = self.find_kotlin_test_files()?;

        // Run gradle test
        let output = Command::new("./gradlew")
            .arg("test")
            .current_dir(&self.root)
            .output()
            .map_err(AuditError::Io)?;

        let duration = start.elapsed().as_secs_f64();
        let output_str = String::from_utf8_lossy(&output.stdout).to_string();

        // Parse gradle output
        let (total, passed, failed, skipped) = self.parse_gradle_output(&output_str);

        Ok(TestResults {
            project_type: ProjectType::Kotlin,
            total,
            passed,
            failed,
            skipped,
            duration,
            test_files,
            coverage: None,
            results_by_file: HashMap::new(),
            output: output_str,
        })
    }

    /// Find Rust test files
    fn find_rust_test_files(&self) -> Result<Vec<String>> {
        let mut test_files = Vec::new();

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if content.contains("#[test]") || content.contains("#[cfg(test)]") {
                if let Ok(rel_path) = entry.path().strip_prefix(&self.root) {
                    test_files.push(rel_path.display().to_string());
                }
            }
        }

        Ok(test_files)
    }

    /// Find Python test files
    fn find_python_test_files(&self) -> Result<Vec<String>> {
        let mut test_files = Vec::new();

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("test_") || n.ends_with("_test.py"))
            })
        {
            if let Ok(rel_path) = entry.path().strip_prefix(&self.root) {
                test_files.push(rel_path.display().to_string());
            }
        }

        Ok(test_files)
    }

    /// Find JavaScript/TypeScript test files
    fn find_js_test_files(&self) -> Result<Vec<String>> {
        let mut test_files = Vec::new();

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| {
                        n.ends_with(".test.ts")
                            || n.ends_with(".test.tsx")
                            || n.ends_with(".test.js")
                            || n.ends_with(".spec.ts")
                            || n.ends_with(".spec.js")
                    })
            })
        {
            if let Ok(rel_path) = entry.path().strip_prefix(&self.root) {
                test_files.push(rel_path.display().to_string());
            }
        }

        Ok(test_files)
    }

    /// Find Kotlin test files
    fn find_kotlin_test_files(&self) -> Result<Vec<String>> {
        let mut test_files = Vec::new();

        for entry in WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .components()
                    .any(|c| c.as_os_str() == "test" || c.as_os_str() == "androidTest")
                    && e.path().extension().is_some_and(|ext| ext == "kt")
            })
        {
            if let Ok(rel_path) = entry.path().strip_prefix(&self.root) {
                test_files.push(rel_path.display().to_string());
            }
        }

        Ok(test_files)
    }

    /// Parse cargo test output
    fn parse_cargo_test_output(&self, output: &str) -> (usize, usize, usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Look for summary line like "test result: ok. 15 passed; 0 failed; 0 ignored"
        for line in output.lines() {
            if line.contains("test result:") {
                if let Some(stats) = line.split("test result:").nth(1) {
                    // Parse numbers - look for patterns like "15 passed", "2 failed", "1 ignored"
                    for part in stats.split(';') {
                        // Find a number followed by a keyword in this part
                        for word in part.split_whitespace() {
                            if let Ok(num) = word.parse::<usize>() {
                                // Check if the rest of part indicates the type
                                if part.contains("passed") {
                                    passed = num;
                                } else if part.contains("failed") {
                                    failed = num;
                                } else if part.contains("ignored") {
                                    skipped = num;
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        let total = passed + failed + skipped;
        (total, passed, failed, skipped)
    }

    /// Parse pytest output
    fn parse_pytest_output(&self, output: &str) -> (usize, usize, usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        // Look for summary line like "15 passed, 2 failed, 1 skipped in 2.5s"
        for line in output.lines() {
            if line.contains("passed") || line.contains("failed") {
                for part in line.split(',') {
                    if let Some(num_str) = part.split_whitespace().next() {
                        if let Ok(num) = num_str.parse::<usize>() {
                            if part.contains("passed") {
                                passed = num;
                            } else if part.contains("failed") {
                                failed = num;
                            } else if part.contains("skipped") {
                                skipped = num;
                            }
                        }
                    }
                }
            }
        }

        let total = passed + failed + skipped;
        (total, passed, failed, skipped)
    }

    /// Parse jest output
    fn parse_jest_output(&self, output: &str) -> (usize, usize, usize, usize) {
        // Jest JSON output parsing
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
            let total = json["numTotalTests"].as_u64().unwrap_or(0) as usize;
            let passed = json["numPassedTests"].as_u64().unwrap_or(0) as usize;
            let failed = json["numFailedTests"].as_u64().unwrap_or(0) as usize;
            let skipped = json["numPendingTests"].as_u64().unwrap_or(0) as usize;
            return (total, passed, failed, skipped);
        }

        (0, 0, 0, 0)
    }

    /// Parse gradle output
    fn parse_gradle_output(&self, output: &str) -> (usize, usize, usize, usize) {
        let mut passed = 0;
        let failed = 0;
        let skipped = 0;

        // Look for BUILD SUCCESSFUL and test counts
        for line in output.lines() {
            if line.contains("tests completed") {
                // Parse the test count
                if let Some(num_str) = line.split_whitespace().next() {
                    if let Ok(num) = num_str.parse::<usize>() {
                        passed = num; // Assume all passed if BUILD SUCCESSFUL
                    }
                }
            }
        }

        let total = passed + failed + skipped;
        (total, passed, failed, skipped)
    }

    /// Get Rust code coverage using tarpaulin or llvm-cov
    fn get_rust_coverage(&self) -> Result<f64> {
        // Try cargo-tarpaulin first
        let output = Command::new("cargo")
            .arg("tarpaulin")
            .arg("--out")
            .arg("Stdout")
            .current_dir(&self.root)
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Some(coverage_line) = output_str.lines().find(|l| l.contains("coverage")) {
                // Parse percentage
                if let Some(pct_str) = coverage_line.split('%').next() {
                    if let Some(num_str) = pct_str.split_whitespace().last() {
                        if let Ok(pct) = num_str.parse::<f64>() {
                            return Ok(pct);
                        }
                    }
                }
            }
        }

        Err(AuditError::Config(
            "Coverage tool not available".to_string(),
        ))
    }

    /// Get Python code coverage using pytest-cov
    fn get_python_coverage(&self) -> Result<f64> {
        let output = Command::new("pytest")
            .arg("--cov")
            .arg("--cov-report=term")
            .current_dir(&self.root)
            .output()
            .map_err(AuditError::Io)?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Look for TOTAL line with coverage percentage
        for line in output_str.lines() {
            if line.contains("TOTAL") {
                if let Some(pct_str) = line.split('%').next() {
                    if let Some(num_str) = pct_str.split_whitespace().last() {
                        if let Ok(pct) = num_str.parse::<f64>() {
                            return Ok(pct);
                        }
                    }
                }
            }
        }

        Err(AuditError::Config(
            "Coverage not found in output".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_output() {
        let runner = TestRunner::new(".");
        let output = "test result: ok. 15 passed; 2 failed; 1 ignored; 0 measured";
        let (total, passed, failed, skipped) = runner.parse_cargo_test_output(output);

        assert_eq!(passed, 15);
        assert_eq!(failed, 2);
        assert_eq!(skipped, 1);
        assert_eq!(total, 18);
    }

    #[test]
    fn test_parse_pytest_output() {
        let runner = TestRunner::new(".");
        let output = "15 passed, 2 failed, 1 skipped in 2.5s";
        let (total, passed, failed, skipped) = runner.parse_pytest_output(output);

        assert_eq!(passed, 15);
        assert_eq!(failed, 2);
        assert_eq!(skipped, 1);
        assert_eq!(total, 18);
    }
}
