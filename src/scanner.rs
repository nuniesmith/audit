//! Scanner module for file system scanning and static analysis

use crate::error::{AuditError, Result};
use crate::tags::TagScanner;
use crate::types::{
    AuditReport, AuditRequest, AuditSummary, Category, FileAnalysis, FilePriority, Issue,
    IssueCategory, IssueSeverity, SystemMap,
};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Scanner for analyzing codebases
pub struct Scanner {
    /// Root directory to scan
    root: PathBuf,
    /// Tag scanner
    tag_scanner: TagScanner,
    /// Maximum file size to scan (bytes)
    max_file_size: usize,
    /// Whether to include tests
    include_tests: bool,
}

impl Scanner {
    /// Create a new scanner
    pub fn new(root: PathBuf, max_file_size: usize, include_tests: bool) -> Result<Self> {
        let tag_scanner = TagScanner::new()?;

        Ok(Self {
            root,
            tag_scanner,
            max_file_size,
            include_tests,
        })
    }

    /// Scan the codebase and generate a report
    pub fn scan(&self, request: &AuditRequest) -> Result<AuditReport> {
        info!("Starting codebase scan at {}", self.root.display());

        // Build system map
        let system_map = self.build_system_map()?;

        // Scan all files
        let files = self.scan_files()?;

        // Calculate summary
        let summary = self.calculate_summary(&files);

        // Generate tasks from analyses (placeholder - will be filled by TaskGenerator)
        let tasks = Vec::new();

        // Count issues by severity
        let mut issues_by_severity = HashMap::new();
        for file in &files {
            for issue in &file.issues {
                *issues_by_severity.entry(issue.severity).or_insert(0) += 1;
            }
        }

        let report = AuditReport {
            id: uuid::Uuid::new_v4().to_string(),
            repository: request.repository.clone(),
            branch: request.branch.clone().unwrap_or_else(|| "main".to_string()),
            created_at: chrono::Utc::now(),
            system_map,
            files,
            tasks,
            issues_by_severity,
            summary,
            test_results: None,
            context_bundle: None,
        };

        info!(
            "Scan complete: {} files, {} issues",
            report.summary.total_files, report.summary.total_issues
        );

        Ok(report)
    }

    /// Build system architecture map
    fn build_system_map(&self) -> Result<SystemMap> {
        let mut total_files = 0;
        let mut files_by_category = HashMap::new();
        let mut lines_by_category = HashMap::new();

        for entry in WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .build()
        {
            let entry = entry.map_err(|e| AuditError::other(format!("Walk error: {}", e)))?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if !self.should_analyze(path) {
                continue;
            }

            total_files += 1;
            let category = Category::from_path(&path.to_string_lossy());

            *files_by_category.entry(category).or_insert(0) += 1;

            // Count lines
            if let Ok(content) = fs::read_to_string(path) {
                let line_count = content.lines().count();
                *lines_by_category.entry(category).or_insert(0) += line_count;
            }
        }

        // Extract dependencies from imports and use statements
        let dependencies = self.extract_dependencies()?;

        // Generate mermaid diagram
        let mermaid_diagram =
            Some(self.generate_mermaid_diagram(&files_by_category, &dependencies));

        Ok(SystemMap {
            total_files,
            files_by_category,
            lines_by_category,
            dependencies,
            mermaid_diagram,
        })
    }

    /// Extract service dependencies from source files
    fn extract_dependencies(&self) -> Result<Vec<crate::types::ServiceDependency>> {
        use crate::types::{DependencyType, ServiceDependency};
        use std::collections::HashSet;

        let mut dependencies = HashSet::new();

        for entry in WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .build()
        {
            let entry = entry.map_err(|e| AuditError::other(format!("Walk error: {}", e)))?;
            let path = entry.path();

            if !path.is_file() || !self.should_analyze(path) {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let from_service = self.path_to_service(path);

            // Check for gRPC client connections
            for line in content.lines() {
                // gRPC connections (Rust tonic)
                if line.contains("::connect(") || line.contains("Channel::from_") {
                    if let Some(to_service) = self.extract_grpc_target(line) {
                        dependencies.insert((
                            from_service.clone(),
                            to_service,
                            DependencyType::Grpc,
                        ));
                    }
                }

                // HTTP client calls
                if line.contains("reqwest::") || line.contains("hyper::") {
                    if let Some(to_service) = self.extract_http_target(line) {
                        dependencies.insert((
                            from_service.clone(),
                            to_service,
                            DependencyType::Http,
                        ));
                    }
                }

                // Internal crate dependencies (use statements)
                if line.starts_with("use ") || line.contains("use crate::") {
                    if let Some(to_service) = self.extract_internal_dep(line) {
                        if to_service != from_service {
                            dependencies.insert((
                                from_service.clone(),
                                to_service,
                                DependencyType::Internal,
                            ));
                        }
                    }
                }
            }
        }

        Ok(dependencies
            .into_iter()
            .map(|(from, to, dep_type)| ServiceDependency { from, to, dep_type })
            .collect())
    }

    /// Extract service name from file path
    fn path_to_service(&self, path: &Path) -> String {
        let path_str = path.to_string_lossy();

        if path_str.contains("/janus/") {
            if path_str.contains("/neuromorphic/") {
                "neuromorphic".to_string()
            } else if path_str.contains("/crates/ml/") {
                "ml".to_string()
            } else if path_str.contains("/crates/exchanges/") {
                "exchanges".to_string()
            } else if path_str.contains("/crates/vision/") {
                "vision".to_string()
            } else if path_str.contains("/crates/cns/") {
                "cns".to_string()
            } else {
                "janus".to_string()
            }
        } else if path_str.contains("/execution/") {
            "execution".to_string()
        } else if path_str.contains("/audit/") {
            "audit".to_string()
        } else if path_str.contains("/clients/") {
            "clients".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Extract gRPC target service from a connect call
    fn extract_grpc_target(&self, line: &str) -> Option<String> {
        // Look for port patterns to identify services
        if line.contains(":50051") || line.contains("execution") {
            Some("execution".to_string())
        } else if line.contains(":50052") || line.contains("janus") {
            Some("janus".to_string())
        } else if line.contains(":9090") || line.contains("prometheus") {
            Some("prometheus".to_string())
        } else {
            None
        }
    }

    /// Extract HTTP target service from client calls
    fn extract_http_target(&self, line: &str) -> Option<String> {
        if line.contains("localhost:") || line.contains("127.0.0.1:") {
            // Try to identify by port
            if line.contains(":8080") {
                Some("api-gateway".to_string())
            } else if line.contains(":3000") {
                Some("dashboard".to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Extract internal crate dependency from use statement
    fn extract_internal_dep(&self, line: &str) -> Option<String> {
        let crates = [
            "janus",
            "execution",
            "common",
            "ml",
            "vision",
            "exchanges",
            "cns",
            "audit",
        ];

        for crate_name in &crates {
            if line.contains(&format!("use {}::", crate_name))
                || line.contains(&format!("use crate::{}", crate_name))
                || line.contains(&format!("{}::", crate_name))
            {
                return Some(crate_name.to_string());
            }
        }

        None
    }

    /// Generate mermaid diagram from dependencies
    fn generate_mermaid_diagram(
        &self,
        files_by_category: &HashMap<Category, usize>,
        dependencies: &[crate::types::ServiceDependency],
    ) -> String {
        use crate::types::DependencyType;

        let mut diagram = String::from("graph TB\n");

        // Add subgraphs for each category with files
        diagram.push_str("    subgraph Core[\"Core Services\"]\n");
        if files_by_category.contains_key(&Category::Janus) {
            diagram.push_str("        janus[\"JANUS\\nNeuromorphic Engine\"]\n");
        }
        if files_by_category.contains_key(&Category::Execution) {
            diagram.push_str("        execution[\"Execution\\nOrder Management\"]\n");
        }
        diagram.push_str("    end\n\n");

        diagram.push_str("    subgraph Support[\"Support Services\"]\n");
        if files_by_category.contains_key(&Category::Audit) {
            diagram.push_str("        audit[\"Audit\\nCode Analysis\"]\n");
        }
        if files_by_category.contains_key(&Category::Clients) {
            diagram.push_str("        clients[\"Clients\\nAPI Clients\"]\n");
        }
        diagram.push_str("    end\n\n");

        diagram.push_str("    subgraph Infrastructure[\"Infrastructure\"]\n");
        if files_by_category.contains_key(&Category::Infra) {
            diagram.push_str("        infra[\"Infra\\nDeployment\"]\n");
        }
        diagram.push_str("    end\n\n");

        // Add dependency arrows
        for dep in dependencies {
            let arrow = match dep.dep_type {
                DependencyType::Grpc => "-->|gRPC|",
                DependencyType::Http => "-.->|HTTP|",
                DependencyType::Internal => "-->",
            };
            diagram.push_str(&format!("    {} {} {}\n", dep.from, arrow, dep.to));
        }

        diagram
    }

    /// Scan all files in the codebase
    fn scan_files(&self) -> Result<Vec<FileAnalysis>> {
        let mut analyses = Vec::new();

        for entry in WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .build()
        {
            let entry = entry.map_err(|e| AuditError::other(format!("Walk error: {}", e)))?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if !self.should_analyze(path) {
                continue;
            }

            match self.analyze_file(path) {
                Ok(analysis) => analyses.push(analysis),
                Err(e) => {
                    warn!("Failed to analyze {}: {}", path.display(), e);
                }
            }
        }

        Ok(analyses)
    }

    /// Analyze a single file
    fn analyze_file(&self, path: &Path) -> Result<FileAnalysis> {
        debug!("Analyzing file: {}", path.display());

        let category = Category::from_path(&path.to_string_lossy());
        let priority = FilePriority::from_path(&path.to_string_lossy());

        // Read file content
        let content = fs::read_to_string(path).map_err(|e| {
            AuditError::other(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        let lines = content.lines().count();

        // Count documentation blocks
        let doc_blocks = self.count_doc_blocks(&content, &category);

        // Scan for tags
        let tags = self.tag_scanner.scan_file(path)?;

        // Perform static analysis
        let issues = self.static_analysis(path, &content, category)?;

        Ok(FileAnalysis {
            path: path.to_path_buf(),
            category,
            priority,
            lines,
            doc_blocks,
            security_rating: None, // Will be filled by LLM
            issues,
            llm_analysis: None, // Will be filled by LLM
            tags,
        })
    }

    /// Count documentation blocks in a file
    fn count_doc_blocks(&self, content: &str, category: &Category) -> usize {
        let mut count = 0;

        match category {
            Category::Janus | Category::Execution | Category::Clients | Category::Audit => {
                // Count /// and //! comments (all are Rust code)
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                        count += 1;
                    }
                }
            }
            _ => {}
        }

        count
    }

    /// Perform static analysis on a file
    fn static_analysis(
        &self,
        path: &Path,
        content: &str,
        category: Category,
    ) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();

        match category {
            Category::Janus | Category::Execution | Category::Clients | Category::Audit => {
                // All are Rust code
                issues.extend(self.check_rust_issues(path, content)?);
            }
            Category::Infra => {
                issues.extend(self.check_infra_issues(path, content)?);
            }
            _ => {}
        }

        Ok(issues)
    }

    /// Check Rust-specific issues
    fn check_rust_issues(&self, path: &Path, content: &str) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1;

            // Check for unwrap()
            if line.contains(".unwrap()") && !line.trim_start().starts_with("//") {
                issues.push(Issue {
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::CodeQuality,
                    file: path.to_path_buf(),
                    line: line_number,
                    message: "Use of .unwrap() - consider proper error handling".to_string(),
                    suggestion: Some("Use ? operator or match/if let".to_string()),
                });
            }

            // Check for expect()
            if line.contains(".expect(") && !line.trim_start().starts_with("//") {
                issues.push(Issue {
                    severity: IssueSeverity::Low,
                    category: IssueCategory::CodeQuality,
                    file: path.to_path_buf(),
                    line: line_number,
                    message: "Use of .expect() - ensure error message is descriptive".to_string(),
                    suggestion: None,
                });
            }

            // Check for panic!()
            if line.contains("panic!(") && !line.trim_start().starts_with("//") {
                issues.push(Issue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::CodeQuality,
                    file: path.to_path_buf(),
                    line: line_number,
                    message: "Explicit panic - avoid in library code".to_string(),
                    suggestion: Some("Return a Result instead".to_string()),
                });
            }

            // Check for unsafe blocks
            if line.contains("unsafe") && !line.trim_start().starts_with("//") {
                issues.push(Issue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::Security,
                    file: path.to_path_buf(),
                    line: line_number,
                    message: "Unsafe code block - ensure proper documentation and justification"
                        .to_string(),
                    suggestion: Some("Add SAFETY comment explaining invariants".to_string()),
                });
            }

            // Check for blocking operations in async context
            if (line.contains("std::thread::sleep") || line.contains("std::fs::read"))
                && (content.contains("async fn") || content.contains("#[tokio::main]"))
            {
                issues.push(Issue {
                    severity: IssueSeverity::High,
                    category: IssueCategory::AsyncSafety,
                    file: path.to_path_buf(),
                    line: line_number,
                    message: "Blocking operation in async context".to_string(),
                    suggestion: Some(
                        "Use async equivalent (tokio::time::sleep, tokio::fs::read)".to_string(),
                    ),
                });
            }
        }

        Ok(issues)
    }

    /// Check infrastructure/Docker issues
    fn check_infra_issues(&self, path: &Path, content: &str) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Check Dockerfiles
        if filename == "Dockerfile" || path.to_string_lossy().contains("Dockerfile") {
            for (line_num, line) in content.lines().enumerate() {
                let line_number = line_num + 1;

                // Check for running as root
                if line.contains("USER root") {
                    issues.push(Issue {
                        severity: IssueSeverity::High,
                        category: IssueCategory::Security,
                        file: path.to_path_buf(),
                        line: line_number,
                        message: "Running container as root - security risk".to_string(),
                        suggestion: Some("Create and use a non-root user".to_string()),
                    });
                }

                // Check for latest tag
                if line.contains(":latest") {
                    issues.push(Issue {
                        severity: IssueSeverity::Medium,
                        category: IssueCategory::CodeQuality,
                        file: path.to_path_buf(),
                        line: line_number,
                        message: "Using :latest tag - not reproducible".to_string(),
                        suggestion: Some("Pin to specific version".to_string()),
                    });
                }
            }
        }

        // Check Docker Compose files
        if filename.starts_with("compose")
            || filename == "docker-compose.yml"
            || filename == "docker-compose.yaml"
        {
            for (line_num, line) in content.lines().enumerate() {
                let line_number = line_num + 1;

                // Check for latest tag in compose files
                if line.contains(":latest") {
                    issues.push(Issue {
                        severity: IssueSeverity::Medium,
                        category: IssueCategory::CodeQuality,
                        file: path.to_path_buf(),
                        line: line_number,
                        message: "Using :latest tag in compose - not reproducible".to_string(),
                        suggestion: Some("Pin to specific version".to_string()),
                    });
                }

                // Check for privileged mode
                if line.trim().starts_with("privileged:") && line.contains("true") {
                    issues.push(Issue {
                        severity: IssueSeverity::High,
                        category: IssueCategory::Security,
                        file: path.to_path_buf(),
                        line: line_number,
                        message: "Container running in privileged mode - security risk".to_string(),
                        suggestion: Some(
                            "Remove privileged mode or use specific capabilities".to_string(),
                        ),
                    });
                }
            }
        }

        // Check shell scripts
        if filename.ends_with(".sh") {
            for (line_num, line) in content.lines().enumerate() {
                let line_number = line_num + 1;
                let trimmed = line.trim();

                // Check for set -e (error handling)
                if line_number <= 5
                    && !content.contains("set -e")
                    && !content.contains("set -euo")
                    && line_number == 2
                {
                    // Only warn once
                    issues.push(Issue {
                        severity: IssueSeverity::Medium,
                        category: IssueCategory::CodeQuality,
                        file: path.to_path_buf(),
                        line: 1,
                        message: "Shell script missing 'set -e' for error handling".to_string(),
                        suggestion: Some(
                            "Add 'set -e' or 'set -euo pipefail' near the top".to_string(),
                        ),
                    });
                }

                // Check for unsafe commands
                if trimmed.contains("rm -rf") && !trimmed.starts_with("#") {
                    issues.push(Issue {
                        severity: IssueSeverity::High,
                        category: IssueCategory::Security,
                        file: path.to_path_buf(),
                        line: line_number,
                        message: "Potentially dangerous 'rm -rf' command".to_string(),
                        suggestion: Some(
                            "Ensure path is properly validated and quoted".to_string(),
                        ),
                    });
                }
            }
        }

        // Check for hardcoded secrets in all infrastructure files
        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1;
            let lower = line.to_lowercase();

            if (lower.contains("password") || lower.contains("api_key") || lower.contains("secret"))
                && (line.contains("=") || line.contains(":"))
                && !line.trim_start().starts_with("#")
                && !line.trim_start().starts_with("//")
            {
                issues.push(Issue {
                    severity: IssueSeverity::Critical,
                    category: IssueCategory::Security,
                    file: path.to_path_buf(),
                    line: line_number,
                    message: "Possible hardcoded secret - use environment variables".to_string(),
                    suggestion: Some("Move to .env file or secrets manager".to_string()),
                });
            }
        }

        Ok(issues)
    }

    /// Check if a file should be analyzed
    fn should_analyze(&self, path: &Path) -> bool {
        // Check file size
        if let Ok(metadata) = path.metadata() {
            if metadata.len() > self.max_file_size as u64 {
                return false;
            }
        }

        // Check if it's a test file
        if !self.include_tests {
            let path_str = path.to_string_lossy();
            if path_str.contains("test") || path_str.contains("tests/") {
                return false;
            }
        }

        // Check for specific infrastructure files without extensions or with special names
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename == "Dockerfile"
                || filename == "compose"
                || filename == "README"
                || filename == ".dockerignore"
                || filename == ".gitignore"
                || filename == ".gitattributes"
                || filename.starts_with("compose.")
                || filename.ends_with(".sh")
            {
                return true;
            }
        }

        // Only analyze source files by extension
        let extension = path.extension().and_then(|e| e.to_str());
        matches!(
            extension,
            Some("rs")
                | Some("py")
                | Some("kt")
                | Some("kts")
                | Some("swift")
                | Some("ts")
                | Some("tsx")
                | Some("js")
                | Some("toml")
                | Some("yaml")
                | Some("yml")
                | Some("md")
                | Some("sh")
        )
    }

    /// Calculate audit summary
    fn calculate_summary(&self, files: &[FileAnalysis]) -> AuditSummary {
        let total_files = files.len();
        let total_lines = files.iter().map(|f| f.lines).sum();
        let total_issues = files.iter().map(|f| f.issues.len()).sum();

        let critical_files = files
            .iter()
            .filter(|f| {
                f.issues
                    .iter()
                    .any(|i| i.severity == IssueSeverity::Critical)
            })
            .count();

        AuditSummary {
            total_files,
            total_lines,
            total_issues,
            total_tasks: 0, // Will be filled by TaskGenerator
            critical_files,
            avg_security_rating: None, // Will be filled by LLM
            total_tests: None,
            test_pass_rate: None,
            code_coverage: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    #[test]
    fn test_scanner_new() {
        let temp = TempDir::new().unwrap();
        let scanner = Scanner::new(temp.path().to_path_buf(), 1_000_000, false).unwrap();
        assert_eq!(scanner.max_file_size, 1_000_000);
        assert!(!scanner.include_tests);
    }

    #[test]
    fn test_should_analyze() {
        let temp = TempDir::new().unwrap();
        let scanner = Scanner::new(temp.path().to_path_buf(), 1_000_000, false).unwrap();

        assert!(scanner.should_analyze(Path::new("main.rs")));
        assert!(scanner.should_analyze(Path::new("app.py")));
        assert!(!scanner.should_analyze(Path::new("data.txt")));
    }

    #[test]
    fn test_check_rust_unwrap() {
        let temp = TempDir::new().unwrap();
        let scanner = Scanner::new(temp.path().to_path_buf(), 1_000_000, false).unwrap();

        let content = r#"
fn test() {
    let value = some_option.unwrap();
}
"#;

        let issues = scanner
            .check_rust_issues(Path::new("test.rs"), content)
            .unwrap();
        assert!(issues.iter().any(|i| i.message.contains("unwrap")));
    }

    #[test]
    fn test_infra_category_detection() {
        // Test various infrastructure file patterns
        assert_eq!(Category::from_path("docker/Dockerfile"), Category::Infra);
        assert_eq!(
            Category::from_path(".github/workflows/ci.yml"),
            Category::Infra
        );
        assert_eq!(Category::from_path(".githooks/pre-commit"), Category::Infra);
        // config/ directory with .yml extension -> Config (not Infra)
        assert_eq!(Category::from_path("config/app.yml"), Category::Config);
        // docs/ -> Documentation
        assert_eq!(
            Category::from_path("docs/README.md"),
            Category::Documentation
        );
        assert_eq!(Category::from_path("scripts/deploy.sh"), Category::Infra);
        assert_eq!(Category::from_path(".dockerignore"), Category::Infra);
        assert_eq!(Category::from_path(".gitignore"), Category::Infra);
        assert_eq!(Category::from_path(".gitattributes"), Category::Infra);
        assert_eq!(Category::from_path("compose"), Category::Infra);
        assert_eq!(Category::from_path("compose.yml"), Category::Infra);
        assert_eq!(Category::from_path("compose.prod.yml"), Category::Infra);
        // .toml files -> Config
        assert_eq!(Category::from_path("pyproject.toml"), Category::Config);
        // .md files -> Documentation
        assert_eq!(Category::from_path("README.md"), Category::Documentation);
        // .sh files without scripts/ prefix -> Other
        assert_eq!(Category::from_path("run.sh"), Category::Other);
    }

    #[test]
    fn test_should_analyze_infrastructure_files() {
        let temp = TempDir::new().unwrap();
        let scanner = Scanner::new(temp.path().to_path_buf(), 1_000_000, false).unwrap();

        // Test infrastructure files without standard extensions
        assert!(scanner.should_analyze(Path::new("Dockerfile")));
        assert!(scanner.should_analyze(Path::new("compose")));
        assert!(scanner.should_analyze(Path::new("compose.yml")));
        assert!(scanner.should_analyze(Path::new("README")));
        assert!(scanner.should_analyze(Path::new(".dockerignore")));
        assert!(scanner.should_analyze(Path::new(".gitignore")));
        assert!(scanner.should_analyze(Path::new(".gitattributes")));
        assert!(scanner.should_analyze(Path::new("deploy.sh")));
    }

    #[test]
    fn test_check_compose_latest_tag() {
        let temp = TempDir::new().unwrap();
        let scanner = Scanner::new(temp.path().to_path_buf(), 1_000_000, false).unwrap();

        let content = r#"
services:
  web:
    image: nginx:latest
"#;

        let issues = scanner
            .check_infra_issues(Path::new("compose.yml"), content)
            .unwrap();
        assert!(issues.iter().any(|i| i.message.contains("latest")));
    }

    #[test]
    fn test_check_compose_privileged_mode() {
        let temp = TempDir::new().unwrap();
        let scanner = Scanner::new(temp.path().to_path_buf(), 1_000_000, false).unwrap();

        let content = r#"
services:
  app:
    image: myapp:1.0
    privileged: true
"#;

        let issues = scanner
            .check_infra_issues(Path::new("compose.yml"), content)
            .unwrap();
        assert!(issues.iter().any(|i| i.message.contains("privileged")));
        assert!(issues.iter().any(|i| i.severity == IssueSeverity::High));
    }

    #[test]
    fn test_check_shell_script_set_e() {
        let temp = TempDir::new().unwrap();
        let scanner = Scanner::new(temp.path().to_path_buf(), 1_000_000, false).unwrap();

        let content = r#"#!/bin/bash
echo "Running script"
command1
command2
"#;

        let issues = scanner
            .check_infra_issues(Path::new("deploy.sh"), content)
            .unwrap();
        assert!(issues.iter().any(|i| i.message.contains("set -e")));
    }

    #[test]
    fn test_check_shell_script_rm_rf() {
        let temp = TempDir::new().unwrap();
        let scanner = Scanner::new(temp.path().to_path_buf(), 1_000_000, false).unwrap();

        let content = r#"#!/bin/bash
set -e
rm -rf /tmp/build
"#;

        let issues = scanner
            .check_infra_issues(Path::new("clean.sh"), content)
            .unwrap();
        assert!(issues.iter().any(|i| i.message.contains("rm -rf")));
        assert!(issues.iter().any(|i| i.severity == IssueSeverity::High));
    }

    #[test]
    fn test_check_hardcoded_secrets() {
        let temp = TempDir::new().unwrap();
        let scanner = Scanner::new(temp.path().to_path_buf(), 1_000_000, false).unwrap();

        let content = r#"
API_KEY = "sk-12345678"
PASSWORD = "secret123"
"#;

        let issues = scanner
            .check_infra_issues(Path::new("config.py"), content)
            .unwrap();
        assert!(issues
            .iter()
            .any(|i| i.message.contains("hardcoded secret")));
        assert!(issues.iter().any(|i| i.severity == IssueSeverity::Critical));
    }
}
