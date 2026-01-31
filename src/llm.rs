//! LLM client for code analysis using Grok 4.1

use crate::error::{AuditError, Result};
use crate::types::Category;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, warn};

/// LLM client for code analysis
pub struct LlmClient {
    /// HTTP client
    client: Client,
    /// API key
    api_key: String,
    /// Model name
    model: String,
    /// LLM provider (xai, google)
    provider: String,
    /// Base URL
    base_url: String,
    /// Max tokens
    max_tokens: usize,
    /// Temperature
    temperature: f64,
}

impl LlmClient {
    /// Create a new LLM client with provider detection
    pub fn new(
        api_key: String,
        model: String,
        max_tokens: usize,
        temperature: f64,
    ) -> Result<Self> {
        // Auto-detect provider from model name
        let provider = if model.starts_with("gemini") {
            "google".to_string()
        } else if model.starts_with("grok") {
            "xai".to_string()
        } else if model.starts_with("claude") {
            "anthropic".to_string()
        } else {
            // Default to XAI
            "xai".to_string()
        };

        Self::new_with_provider(api_key, provider, model, max_tokens, temperature)
    }

    /// Create a new LLM client with explicit provider
    pub fn new_with_provider(
        api_key: String,
        provider: String,
        model: String,
        max_tokens: usize,
        temperature: f64,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // Increased timeout for reasoning models (5 min for Claude Opus)
            .build()
            .map_err(|e| AuditError::other(format!("Failed to create HTTP client: {}", e)))?;

        // Set base URL based on provider
        let base_url = match provider.as_str() {
            "google" | "gemini" => "https://generativelanguage.googleapis.com/v1beta".to_string(),
            "xai" | "grok" => "https://api.x.ai/v1".to_string(),
            "anthropic" | "claude" => "https://api.anthropic.com/v1".to_string(),
            _ => {
                warn!("Unknown provider '{}', defaulting to XAI", provider);
                "https://api.x.ai/v1".to_string()
            }
        };

        info!(
            "LLM client initialized: provider={}, model={}, base_url={}",
            provider, model, base_url
        );

        Ok(Self {
            client,
            api_key,
            model,
            provider,
            base_url,
            max_tokens,
            temperature,
        })
    }

    /// Analyze a file with LLM
    pub async fn analyze_file(
        &self,
        file_path: &Path,
        content: &str,
        category: Category,
    ) -> Result<LlmAnalysisResult> {
        let system_prompt = self.build_system_prompt(category);
        let user_prompt = self.build_file_prompt(file_path, content);

        info!("Analyzing file with LLM: {}", file_path.display());

        let response = self
            .complete(&system_prompt, &user_prompt)
            .await
            .map_err(|e| {
                warn!("LLM analysis failed for {}: {}", file_path.display(), e);
                e
            })?;

        self.parse_analysis_response(&response)
    }

    /// Analyze codebase for tags and architecture
    pub async fn analyze_codebase(
        &self,
        codebase_context: &str,
        focus_areas: &[String],
    ) -> Result<CodebaseAnalysisResult> {
        let system_prompt = "You are a senior software architect and security expert auditing Project JANUS, \
            a neuromorphic trading system inspired by biological neural architecture.\n\n\
            JANUS combines visual pattern recognition (GAF + ViViT) with symbolic logic (LTN) for financial trading.\n\
            Technical Paper: https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex\n\n\
            Analyze the codebase and identify:\n\
            1. Dead or deprecated code (suggest @audit-tag: legacy)\n\
            2. Mathematical formula errors (compare against technical paper)\n\
            3. Safety vulnerabilities (hardcoded secrets, missing circuit breakers)\n\
            4. Performance issues (blocking I/O in forward service, p99 > 10ms)\n\
            5. Architecture violations (incorrect brain-region mapping, service boundary breaches)\n\
            6. Compliance issues (wash sale rules, position limits)\n\n\
            For each issue, categorize as: ARCHITECTURE, MATHEMATICS, PERFORMANCE, SAFETY, COMPLIANCE, or CODE_QUALITY.\n\
            Include paper references (Part X, Section Y) when applicable.\n\n\
            RESPOND ONLY WITH VALID JSON. Do not include any text before or after the JSON object. Use this exact schema:\n\
            {\n  \
              \"deprecated_files\": [\"path/to/file.rs\"],\n  \
              \"missing_types\": [{\"file\": \"path\", \"issue\": \"description\"}],\n  \
              \"security_concerns\": [{\"severity\": \"critical|high|medium|low\", \"file\": \"path\", \"message\": \"description\"}],\n  \
              \"architecture_issues\": [{\"file\": \"path\", \"issue\": \"description\"}],\n  \
              \"recommended_tags\": [{\"file\": \"path\", \"tag\": \"@audit-tag: value\"}]\n\
            }";

        let user_prompt = format!(
            "Codebase Context:\n{}\n\nFocus Areas: {}\n\n\
            Provide analysis with recommended tags for each file.",
            codebase_context,
            focus_areas.join(", ")
        );

        info!(
            "Analyzing codebase with LLM (context size: {} bytes)",
            codebase_context.len()
        );

        let response = self.complete(system_prompt, &user_prompt).await?;
        self.parse_codebase_response(&response)
    }

    /// Batch analyze multiple files
    pub async fn analyze_batch(
        &self,
        files: Vec<(String, String, Category)>, // (path, content, category)
    ) -> Result<Vec<LlmAnalysisResult>> {
        let mut results = Vec::new();

        for (path, content, category) in files {
            let path_buf = Path::new(&path);
            match self.analyze_file(path_buf, &content, category).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Failed to analyze {}: {}", path, e);
                    // Continue with other files
                }
            }
        }

        Ok(results)
    }

    /// Deep analysis using 2M context window with global context bundle
    pub async fn analyze_with_global_context(
        &self,
        global_context: &str,
    ) -> Result<DeepAnalysisResult> {
        let system_prompt = self.build_deep_analysis_system_prompt();
        let user_prompt = format!(
            "{}\n\nPlease analyze this entire codebase using the standard questionnaire.",
            global_context
        );

        info!(
            "Running deep analysis with 2M context window (size: {} bytes)",
            global_context.len()
        );

        let response = self.complete(&system_prompt, &user_prompt).await?;
        self.parse_deep_analysis_response(&response)
    }

    /// Run standard questionnaire for every file in the codebase
    pub async fn run_standard_questionnaire(
        &self,
        global_context: &str,
        files: &[String],
    ) -> Result<Vec<FileAuditResult>> {
        let system_prompt = self.build_questionnaire_system_prompt();

        let file_list = files.join("\n- ");
        let user_prompt = format!(
            "GLOBAL CONTEXT:\n{}\n\n\
            FILES TO AUDIT:\n- {}\n\n\
            For EACH file, answer the standard questionnaire.",
            global_context, file_list
        );

        info!("Running standard questionnaire for {} files", files.len());

        let response = self.complete(&system_prompt, &user_prompt).await?;
        self.parse_questionnaire_response(&response)
    }

    /// Build system prompt for deep analysis with 2M context
    fn build_deep_analysis_system_prompt(&self) -> String {
        r#"You are a senior software architect analyzing Project JANUS, a neuromorphic trading system.

## PROJECT CONTEXT

JANUS is a brain-inspired algorithmic trading platform combining:
- Visual pattern recognition (GAF + ViViT)
- Symbolic logic (Logic Tensor Networks with Łukasiewicz t-norms)
- Multi-timescale memory (Hippocampus → Sharp-Wave Ripple → Neocortex)
- Neuromorphic architecture (brain-region component mapping)

**Technical Paper**: https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex

## ARCHITECTURE
- **Forward Service**: Real-time execution (<10ms latency), located in services/forward/
- **Backward Service**: Memory consolidation (market close only), located in services/backward/
- **CNS**: Central Nervous System monitoring, located in services/cns/

## CRITICAL CHECKS

### Mathematical Correctness
- GAF normalization MUST use learnable parameters (γ, β) with tanh
- LTN MUST use Łukasiewicz logic for inference, Product logic for training
- PER priorities MUST include epsilon (ε = 10⁻⁶) to prevent division by zero
- Qdrant vectors MUST be L2-normalized before insertion

### Safety & Compliance
- Circuit breakers using Mahalanobis distance (τ = 5 for p < 0.001)
- Wash sale rules: no buy within 30 days of loss-realizing sell
- Position limits: MAX_POSITION_SIZE = 10%, DAILY_LOSS_LIMIT = 2%
- API keys NEVER hardcoded (must use environment variables)

### Performance Requirements
- Forward service: p99 latency < 10ms, throughput > 10K req/s
- No blocking I/O in async contexts, especially in forward service
- Memory: Forward < 2GB RSS, Backward < 8GB RSS

## ANALYSIS TASKS

You have been provided with:
1. **Signature Map**: All function/struct/trait definitions
2. **Dependency Graph**: Cross-file imports and relationships
3. **Architectural Rules**: Project constraints and invariants
4. **Diff Context**: Recent changes
5. **Test Coverage**: Test results and metrics
6. **Full Source Code**: Complete codebase

Identify:

### LOGIC DRIFT
- Code violating neuromorphic architectural principles
- Functions duplicating logic from shared libraries
- Inconsistent patterns across brain-region modules

### DEAD CODE
- Files with no incoming imports (using dependency graph)
- Unreferenced functions and types
- Deprecated code without @audit-tag: legacy

### SAFETY ISSUES (CRITICAL)
- In Rust: unwrap() in execution/hot-path code, unsafe blocks without justification
- In Python: missing type annotations, eval/exec usage
- Blocking I/O in async functions (especially forward service)
- Hardcoded API keys or secrets
- Missing circuit breakers or unbounded operations

### MATHEMATICAL ERRORS
- Formula implementations not matching technical paper specifications
- Domain violations (e.g., arccos on values outside [-1, 1])
- Incorrect t-norm usage (Łukasiewicz vs. Product)
- Missing numerical stability terms (epsilon in divisions)

### INCOMPLETENESS
- TODO comments without task IDs or paper references
- Empty function bodies in non-stub files
- Missing error handling

## ISSUE CATEGORIZATION

Use these categories:
- **ARCHITECTURE**: Neuromorphic design violations, service boundary issues
- **MATHEMATICS**: Formula errors, domain violations, numerical instability
- **PERFORMANCE**: Latency violations, blocking operations, memory leaks
- **SAFETY**: Missing circuit breakers, hardcoded secrets, unbounded operations
- **COMPLIANCE**: Regulatory violations (wash sale, position limits, PDT)
- **LOGIC**: Business logic errors, incorrect constraint encoding
- **CODE_QUALITY**: Missing docs, poor naming, test coverage gaps

## OUTPUT FORMAT

RESPOND ONLY WITH VALID JSON. Do not include any explanatory text before or after the JSON object.
The JSON must match this exact schema:
{
  "logic_drift": ["description of drift 1", "description of drift 2"],
  "dead_code": ["path/to/dead/file.rs", "path/to/unused/module.rs"],
  "safety_issues": [
    {
      "file": "path/to/file.rs",
      "line": 123,
      "severity": "critical|high|medium|low",
      "message": "description of safety issue"
    }
  ],
  "mathematical_errors": [
    {
      "file": "path/to/file.rs",
      "line": 456,
      "paper_reference": "Part X, Section Y, Eq. Z",
      "error": "description of mathematical error"
    }
  ],
  "incomplete_code": ["path/to/incomplete.rs"],
  "tasks": [
    {
      "file": "path/to/file.rs",
      "line": 123,
      "priority": "Critical|High|Medium|Low",
      "category": "ARCHITECTURE|MATHEMATICS|PERFORMANCE|SAFETY|COMPLIANCE|LOGIC|CODE_QUALITY",
      "paper_reference": "Part X, Section Y, Eq. Z (if applicable)",
      "description": "detailed description",
      "suggested_fix": "suggested solution"
    }
  ]
}"#
        .to_string()
    }

    /// Build system prompt for standard questionnaire
    fn build_questionnaire_system_prompt(&self) -> String {
        r#"You are an expert code auditor for Project JANUS, a neuromorphic trading system.

## PROJECT CONTEXT

JANUS combines brain-inspired architecture with financial trading:
- **Forward Service**: Real-time execution (<10ms latency) - Visual cortex, Basal ganglia, Cerebellum
- **Backward Service**: Memory consolidation (market close) - Hippocampus, Sharp-Wave Ripple, Neocortex
- **CNS**: Health monitoring (Prometheus + Grafana)

**Technical Specification**: https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex

## CRITICAL REQUIREMENTS

### Mathematical Correctness
- GAF: Learnable normalization with tanh, ensures |x| < 1 for arccos
- LTN: Łukasiewicz t-norms for inference, Product t-norms for training
- PER: Include epsilon (10⁻⁶) in priority calculations
- Qdrant: L2-normalize vectors before insertion

### Safety & Compliance
- No hardcoded API keys (use environment variables)
- Circuit breakers with Mahalanobis distance thresholds
- Wash sale constraint: no buy within 30 days of loss-realizing sell
- Position limits: 10% max, 2% daily loss limit

### Performance
- Forward service: No blocking I/O, pre-allocated buffers, SIMD where possible
- Backward service: Runs only during market close
- Memory: Forward < 2GB, Backward < 8GB

## STANDARD QUESTIONNAIRE

For EVERY file in the list, answer these questions:

1. **REACHABILITY**: Is this file imported/used by any other module?
   - If NO: Suggest @audit-tag: legacy

2. **JANUS COMPLIANCE**: Does this file violate project-specific rules?
   - Rust hot-path: Check for unwrap(), blocking I/O in async
   - Mathematical: Verify formulas match technical paper specifications
   - Safety: Check for hardcoded secrets, missing circuit breakers
   - Architecture: Verify brain-region mapping is clear and documented

3. **MATHEMATICAL CORRECTNESS**: If implementing paper formulas:
   - Cite paper reference (Part X, Section Y, Equation Z)
   - Verify implementation matches specification exactly
   - Check for domain violations (arccos, division by zero)
   - Ensure numerical stability (epsilon terms)

4. **COMPLETENESS**: Are there TODO comments or incomplete implementations?
   - If YES: Generate task with priority and paper reference if applicable

5. **NEUROMORPHIC ALIGNMENT**: Does the component align with brain-inspired design?
   - Check for proper brain-region mapping (Visual cortex, Hippocampus, etc.)
   - Verify service boundary (Forward vs. Backward)
   - Ensure hot/cold path separation

6. **IMPROVEMENT**: Suggest ONE high-impact refactor
   - Focus on: mathematical correctness, safety, performance, or architectural alignment

## OUTPUT FORMAT

RESPOND ONLY WITH VALID JSON. Do not include any explanatory text before or after the JSON object.
The JSON must match this exact schema:
{
  "file_audits": [
    {
      "file": "path/to/file.rs",
      "reachable": true,
      "brain_region": "Visual Cortex|Hippocampus|Basal Ganglia|etc. (if applicable)",
      "service_boundary": "Forward|Backward|CNS|Shared",
      "compliance_issues": [
        {
          "type": "MATHEMATICS|SAFETY|PERFORMANCE|ARCHITECTURE",
          "description": "detailed description of the issue",
          "paper_reference": "Part X, Section Y (if applicable)"
        }
      ],
      "incomplete": false,
      "suggested_tags": ["neuromorphic", "forward-path", "mathematical"],
      "improvement": "detailed improvement suggestion",
      "priority": "Critical|High|Medium|Low"
    }
  ]
}"#
        .to_string()
    }

    /// Parse deep analysis response
    fn parse_deep_analysis_response(&self, response: &str) -> Result<DeepAnalysisResult> {
        // Try to parse as JSON first
        if let Ok(result) = serde_json::from_str::<DeepAnalysisResult>(response) {
            return Ok(result);
        }

        // Try to extract JSON from markdown code blocks
        if let Some(json_str) = self.extract_json_from_markdown(response) {
            if let Ok(result) = serde_json::from_str::<DeepAnalysisResult>(&json_str) {
                return Ok(result);
            }
        }

        // Try to find JSON object anywhere in the response
        if let Some(json_str) = self.extract_json_object(response) {
            if let Ok(result) = serde_json::from_str::<DeepAnalysisResult>(&json_str) {
                return Ok(result);
            }
        }

        // Log the first 500 chars of the response for debugging
        let preview = if response.len() > 500 {
            format!("{}...", &response[..500])
        } else {
            response.to_string()
        };
        warn!(
            "Failed to parse deep analysis response as JSON, using fallback. Response preview: {}",
            preview
        );

        // Fallback
        Ok(DeepAnalysisResult {
            logic_drift: vec![],
            dead_code: vec![],
            safety_issues: vec![],
            incomplete_code: vec![],
            tasks: vec![],
        })
    }

    /// Parse questionnaire response
    fn parse_questionnaire_response(&self, response: &str) -> Result<Vec<FileAuditResult>> {
        #[derive(Deserialize)]
        struct QuestionnaireResponse {
            file_audits: Vec<FileAuditResult>,
        }

        // Try to parse as wrapped JSON first
        if let Ok(result) = serde_json::from_str::<QuestionnaireResponse>(response) {
            return Ok(result.file_audits);
        }

        // Try to parse as direct array
        if let Ok(results) = serde_json::from_str::<Vec<FileAuditResult>>(response) {
            return Ok(results);
        }

        // Try to extract JSON from markdown code blocks (wrapped format)
        if let Some(json_str) = self.extract_json_from_markdown(response) {
            if let Ok(result) = serde_json::from_str::<QuestionnaireResponse>(&json_str) {
                return Ok(result.file_audits);
            }
            // Try direct array in markdown
            if let Ok(results) = serde_json::from_str::<Vec<FileAuditResult>>(&json_str) {
                return Ok(results);
            }
        }

        // Try to find JSON object anywhere in the response
        if let Some(json_str) = self.extract_json_object(response) {
            if let Ok(result) = serde_json::from_str::<QuestionnaireResponse>(&json_str) {
                return Ok(result.file_audits);
            }
            if let Ok(results) = serde_json::from_str::<Vec<FileAuditResult>>(&json_str) {
                return Ok(results);
            }
        }

        // Log the first 500 chars of the response for debugging
        let preview = if response.len() > 500 {
            format!("{}...", &response[..500])
        } else {
            response.to_string()
        };
        warn!(
            "Failed to parse questionnaire response as JSON. Response preview: {}",
            preview
        );

        // Save failed response to debug file for analysis
        if let Ok(debug_dir) = std::env::var("AUDIT_DEBUG_DIR") {
            let debug_path =
                std::path::Path::new(&debug_dir).join("questionnaire-failed-response.txt");
            if let Err(e) = std::fs::write(&debug_path, response) {
                warn!("Failed to save debug response to {:?}: {}", debug_path, e);
            } else {
                warn!("Saved failed questionnaire response to {:?}", debug_path);
            }
        }

        // Log the response for debugging
        warn!("Failed to parse questionnaire response - logging preview");
        debug!(
            "Response preview (first 1000 chars): {}",
            &response.chars().take(1000).collect::<String>()
        );
        debug!(
            "Response length: {} bytes, {} chars",
            response.len(),
            response.chars().count()
        );

        // Fallback
        warn!("Failed to parse questionnaire response in any known format");
        Ok(vec![])
    }

    /// Build system prompt based on category
    fn build_system_prompt(&self, category: Category) -> String {
        let base = "You are an expert code auditor. Analyze the provided code and respond ONLY with valid JSON. \
            Do not include any text before or after the JSON. The JSON must have these exact fields:\n\
            {\n  \
              \"security_rating\": \"A-F letter grade\",\n  \
              \"importance\": 0.0-1.0 decimal number,\n  \
              \"issues\": [{\"severity\": \"critical|high|medium|low\", \"message\": \"description\", \"line\": optional_number}],\n  \
              \"summary\": \"brief analysis summary\"\n\
            }";

        let category_specific = match category {
            Category::Janus => {
                "\n\nFocus on:\n\
                - Neuromorphic component correctness\n\
                - Decision-making logic and strategy implementation\n\
                - Risk management logic correctness (circuit breakers, kill switches)\n\
                - State management in trading systems\n\
                - Real-time performance constraints\n\
                - Thread safety in concurrent contexts\n\
                - Memory safety and ownership patterns\n\
                - Unsafe code blocks and their justification\n\
                - Order generation and signal creation\n\
                - Exchange connector implementations\n\
                - Async safety (Send + Sync bounds)"
            }
            Category::Execution => {
                "\n\nFocus on:\n\
                - Lightweight service architecture\n\
                - External communication reliability\n\
                - Message passing and signal handling from Janus\n\
                - Network I/O and connection management\n\
                - Error handling in communication paths\n\
                - Minimal latency in signal transmission\n\
                - Proper error propagation to Janus\n\
                - Service isolation and failure handling\n\
                - No business logic - only signal relay"
            }
            Category::Clients => {
                "\n\nFocus on:\n\
                - KMP (Kotlin Multiplatform) best practices\n\
                - Cross-platform code sharing (Android, iOS, Web, Desktop)\n\
                - Platform-specific implementations (expect/actual)\n\
                - UI/UX state management\n\
                - API client implementations\n\
                - Data serialization/deserialization\n\
                - Authentication and session management\n\
                - Offline-first architecture\n\
                - Error handling and user feedback\n\
                - Platform-specific resource management"
            }
            Category::Audit => {
                "\n\nFocus on:\n\
                - Static analysis accuracy\n\
                - LLM integration security\n\
                - File parsing robustness\n\
                - Report generation correctness\n\
                - Proper error handling\n\
                - Test coverage"
            }
            Category::Infra => {
                "\n\nFocus on:\n\
                - Hardcoded secrets or credentials\n\
                - Insecure defaults\n\
                - Missing security headers\n\
                - Exposed sensitive ports\n\
                - Container security best practices"
            }
            _ => "",
        };

        format!("{}{}", base, category_specific)
    }

    /// Build prompt for file analysis
    fn build_file_prompt(&self, file_path: &Path, content: &str) -> String {
        format!(
            "File: {}\n\n```\n{}\n```\n\nProvide a detailed security and quality analysis.",
            file_path.display(),
            content
        )
    }

    /// Call LLM completion API (supports XAI and Google Gemini)
    async fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        match self.provider.as_str() {
            "google" | "gemini" => self.complete_google(system_prompt, user_prompt).await,
            "xai" | "grok" => self.complete_xai(system_prompt, user_prompt).await,
            "anthropic" | "claude" => self.complete_anthropic(system_prompt, user_prompt).await,
            _ => self.complete_xai(system_prompt, user_prompt).await,
        }
    }

    /// Call XAI Grok API using Chat Completions endpoint
    async fn complete_xai(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        use serde_json::Value;

        // Use OpenAI-compatible Chat Completions API format with JSON mode
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            max_tokens: Some(self.max_tokens),
            temperature: Some(self.temperature),
            stream: Some(false),
            response_format: Some(ResponseFormat {
                format_type: "json_object".to_string(),
            }),
        };

        debug!(
            "Sending request to XAI Chat Completions API: model={}",
            self.model
        );

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AuditError::llm_api(format!("XAI API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Save error response for debugging if debug dir is set
            if let Ok(debug_dir) = std::env::var("AUDIT_DEBUG_DIR") {
                let debug_path = std::path::Path::new(&debug_dir).join("llm-error-response.json");
                let _ = std::fs::write(&debug_path, &error_text);
                warn!("Saved error response to {:?}", debug_path);
            }

            return Err(AuditError::llm_api(format!(
                "XAI API returned {}: {}",
                status, error_text
            )));
        }

        // Parse OpenAI-compatible response format
        let response_json: Value = response
            .json()
            .await
            .map_err(|e| AuditError::llm_api(format!("Failed to parse XAI response: {}", e)))?;

        // Extract content from OpenAI-compatible response
        let text = response_json
            .pointer("/choices/0/message/content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                // Log response structure for debugging
                let keys = response_json
                    .as_object()
                    .map(|o| o.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                warn!("Response keys: {:?}", keys);

                // Save full response for debugging
                if let Ok(debug_dir) = std::env::var("AUDIT_DEBUG_DIR") {
                    let debug_path =
                        std::path::Path::new(&debug_dir).join("llm-response-structure.json");
                    let _ = std::fs::write(
                        &debug_path,
                        serde_json::to_string_pretty(&response_json).unwrap_or_default(),
                    );
                    warn!("Saved response structure to {:?}", debug_path);
                }

                AuditError::llm_api(format!(
                    "No completion text in XAI response. Available keys: {:?}",
                    keys
                ))
            })?;

        // Log usage statistics if available (OpenAI-compatible format)
        if let Some(usage_obj) = response_json.get("usage") {
            if let Ok(usage) = serde_json::from_value::<UsageStats>(usage_obj.clone()) {
                let cached = usage
                    .prompt_tokens_details
                    .as_ref()
                    .map(|d| d.cached_tokens)
                    .unwrap_or(0);
                let reasoning = usage
                    .completion_tokens_details
                    .as_ref()
                    .map(|d| d.reasoning_tokens)
                    .unwrap_or(0);

                debug!(
                    "XAI usage: prompt={} (cached={}), completion={} (reasoning={}), total={}",
                    usage.prompt_tokens,
                    cached,
                    usage.completion_tokens,
                    reasoning,
                    usage.total_tokens
                );

                // Calculate and log cost for grok-4-1-fast-reasoning
                let input_cost = ((usage.prompt_tokens - cached) as f64 / 1_000_000.0) * 0.20;
                let cached_cost = (cached as f64 / 1_000_000.0) * 0.05;
                let output_cost = (usage.completion_tokens as f64 / 1_000_000.0) * 0.50;
                let total_cost = input_cost + cached_cost + output_cost;

                debug!(
                    "XAI cost: input=${:.4}, cached=${:.4}, output=${:.4}, total=${:.4}",
                    input_cost, cached_cost, output_cost, total_cost
                );
            }
        }

        Ok(text)
    }

    /// Call Anthropic Claude API (supports Claude Opus 4.5 for high-level analysis)
    ///
    /// Claude Opus 4.5 is Anthropic's most capable model, ideal for:
    /// - Deep codebase auditing and analysis
    /// - Whitepaper conformity verification (JANUS theory)
    /// - Complex reasoning about system architecture
    /// - High-stakes code review and security analysis
    async fn complete_anthropic(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        use serde_json::json;

        // Anthropic Messages API format
        let request_body = json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "system": system_prompt,
            "messages": [{
                "role": "user",
                "content": user_prompt
            }],
            "temperature": self.temperature
        });

        debug!(
            "Sending request to Anthropic Claude API: model={}",
            self.model
        );

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AuditError::llm_api(format!("Anthropic API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Save error response for debugging
            if let Ok(debug_dir) = std::env::var("AUDIT_DEBUG_DIR") {
                let debug_path =
                    std::path::Path::new(&debug_dir).join("anthropic-error-response.json");
                let _ = std::fs::write(&debug_path, &error_text);
                warn!("Saved Anthropic error response to {:?}", debug_path);
            }

            return Err(AuditError::llm_api(format!(
                "Anthropic API returned {}: {}",
                status, error_text
            )));
        }

        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            AuditError::llm_api(format!("Failed to parse Anthropic response: {}", e))
        })?;

        // Extract text from Claude response format
        // Response format: { "content": [{ "type": "text", "text": "..." }], "usage": {...} }
        let text = response_json
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                let keys = response_json
                    .as_object()
                    .map(|o| o.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                warn!("Anthropic response keys: {:?}", keys);

                // Save full response for debugging
                if let Ok(debug_dir) = std::env::var("AUDIT_DEBUG_DIR") {
                    let debug_path =
                        std::path::Path::new(&debug_dir).join("anthropic-response-structure.json");
                    let _ = std::fs::write(
                        &debug_path,
                        serde_json::to_string_pretty(&response_json).unwrap_or_default(),
                    );
                    warn!("Saved Anthropic response structure to {:?}", debug_path);
                }

                AuditError::llm_api(format!(
                    "No completion text in Anthropic response. Available keys: {:?}",
                    keys
                ))
            })?;

        // Log usage statistics if available
        if let Some(usage) = response_json.get("usage") {
            let input_tokens = usage
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let output_tokens = usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_read = usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_creation = usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            debug!(
                "Anthropic usage: input={} (cache_read={}, cache_creation={}), output={}",
                input_tokens, cache_read, cache_creation, output_tokens
            );

            // Calculate cost for Claude Opus 4.5 (as of 2025)
            // Pricing: $15/1M input tokens, $75/1M output tokens
            // Cached input: $1.50/1M, Cache write: $18.75/1M
            let input_cost = (input_tokens as f64 / 1_000_000.0) * 15.0;
            let output_cost = (output_tokens as f64 / 1_000_000.0) * 75.0;
            let cache_read_cost = (cache_read as f64 / 1_000_000.0) * 1.50;
            let cache_write_cost = (cache_creation as f64 / 1_000_000.0) * 18.75;
            let total_cost = input_cost + output_cost + cache_read_cost + cache_write_cost;

            debug!(
                "Anthropic cost (Claude Opus 4.5): input=${:.4}, output=${:.4}, cache_read=${:.4}, cache_write=${:.4}, total=${:.4}",
                input_cost, output_cost, cache_read_cost, cache_write_cost, total_cost
            );
        }

        Ok(text)
    }

    /// Call Google Gemini API
    async fn complete_google(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        use serde_json::json;

        // Combine system and user prompts for Gemini
        let combined_prompt = format!("{}\n\n{}", system_prompt, user_prompt);

        let request_body = json!({
            "contents": [{
                "parts": [{
                    "text": combined_prompt
                }]
            }],
            "generationConfig": {
                "temperature": self.temperature,
                "maxOutputTokens": self.max_tokens,
            }
        });

        debug!("Sending request to Google Gemini API: model={}", self.model);

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AuditError::llm_api(format!("Google API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(AuditError::llm_api(format!(
                "Google API returned {}: {}",
                status, error_text
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AuditError::llm_api(format!("Failed to parse Google response: {}", e)))?;

        // Extract text from Gemini response format
        response_json
            .get("candidates")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|candidate| candidate.get("content"))
            .and_then(|content| content.get("parts"))
            .and_then(|parts| parts.as_array())
            .and_then(|arr| arr.first())
            .and_then(|part| part.get("text"))
            .and_then(|text| text.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                AuditError::llm_api(format!(
                    "No completion text in Google response: {}",
                    response_json
                ))
            })
    }

    /// Parse analysis response from LLM
    fn parse_analysis_response(&self, response: &str) -> Result<LlmAnalysisResult> {
        // Try to parse as JSON first (most common with json_object mode)
        if let Ok(result) = serde_json::from_str::<LlmAnalysisResult>(response) {
            return Ok(result);
        }

        // Try to extract JSON from markdown code blocks
        if let Some(json_str) = self.extract_json_from_markdown(response) {
            if let Ok(result) = serde_json::from_str::<LlmAnalysisResult>(&json_str) {
                return Ok(result);
            }
        }

        // Try to find JSON object anywhere in the response
        if let Some(json_str) = self.extract_json_object(response) {
            if let Ok(result) = serde_json::from_str::<LlmAnalysisResult>(&json_str) {
                return Ok(result);
            }
        }

        // Log the first 500 chars of the response for debugging
        let preview = if response.len() > 500 {
            format!("{}...", &response[..500])
        } else {
            response.to_string()
        };
        warn!(
            "Failed to parse LLM response as JSON, using fallback. Response preview: {}",
            preview
        );

        // Fallback: create a basic analysis with the raw response
        Ok(LlmAnalysisResult {
            security_rating: "C".to_string(),
            importance: 0.5,
            issues: vec![],
            summary: response.to_string(),
        })
    }

    /// Parse codebase analysis response
    fn parse_codebase_response(&self, response: &str) -> Result<CodebaseAnalysisResult> {
        // Try to parse as JSON first
        if let Ok(result) = serde_json::from_str::<CodebaseAnalysisResult>(response) {
            return Ok(result);
        }

        // Try to extract JSON from markdown code blocks
        if let Some(json_str) = self.extract_json_from_markdown(response) {
            if let Ok(result) = serde_json::from_str::<CodebaseAnalysisResult>(&json_str) {
                return Ok(result);
            }
        }

        // Try to find JSON object anywhere in the response
        if let Some(json_str) = self.extract_json_object(response) {
            if let Ok(result) = serde_json::from_str::<CodebaseAnalysisResult>(&json_str) {
                return Ok(result);
            }
        }

        // Log the first 500 chars of the response for debugging
        let preview = if response.len() > 500 {
            format!("{}...", &response[..500])
        } else {
            response.to_string()
        };
        warn!(
            "Failed to parse codebase analysis response as JSON, using fallback. Response preview: {}",
            preview
        );

        // Fallback
        Ok(CodebaseAnalysisResult {
            deprecated_files: vec![],
            missing_types: vec![],
            security_concerns: vec![],
            architecture_issues: vec![],
            recommended_tags: vec![],
        })
    }

    /// Extract JSON from markdown code blocks
    fn extract_json_from_markdown(&self, text: &str) -> Option<String> {
        // Look for ```json ... ``` or ``` ... ```
        let patterns = [
            (r"```json\s*\n([\s\S]*?)\n```", 1),
            (r"```\s*\n([\s\S]*?)\n```", 1),
        ];

        for (pattern, group) in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(text) {
                    if let Some(json) = captures.get(group) {
                        return Some(json.as_str().trim().to_string());
                    }
                }
            }
        }

        None
    }

    /// Extract JSON object from anywhere in text (finds first { ... } block)
    fn extract_json_object(&self, text: &str) -> Option<String> {
        let text = text.trim();

        // Find the first '{' and match to its closing '}'
        let start = text.find('{')?;
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, ch) in text[start..].char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => depth += 1,
                '}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(text[start..start + i + 1].to_string());
                    }
                }
                _ => {}
            }
        }

        None
    }
}

/// Completion request for legacy endpoints
#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct CompletionRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: usize,
    temperature: f64,
}

/// Chat Completions API request format (OpenAI-compatible)
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

/// Response format specification for structured output
#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

/// Chat message
#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Legacy completion response (OpenAI-style)
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct CompletionResponse {
    choices: Vec<Choice>,
}

/// Choice in completion response
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

/// Legacy xAI Responses API response format (deprecated - use Chat Completions API)
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct XaiResponse {
    #[allow(dead_code)]
    output: Vec<XaiOutput>,
    #[serde(default)]
    usage: Option<UsageStats>,
    #[serde(default)]
    #[allow(dead_code)]
    status: Option<String>,
}

/// Legacy xAI output item (deprecated)
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct XaiOutput {
    #[allow(dead_code)]
    content: Vec<XaiContent>,
}

/// Legacy xAI content item (deprecated)
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct XaiContent {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    content_type: String,
    #[allow(dead_code)]
    text: String,
}

/// Usage statistics from API response
#[derive(Debug, Deserialize, Clone)]
struct UsageStats {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
    #[serde(default)]
    prompt_tokens_details: Option<PromptTokensDetails>,
    #[serde(default)]
    completion_tokens_details: Option<CompletionTokensDetails>,
}

/// Prompt token details
#[derive(Debug, Deserialize, Clone)]
struct PromptTokensDetails {
    #[serde(default)]
    cached_tokens: u32,
    #[serde(default)]
    #[allow(dead_code)]
    text_tokens: u32,
    #[serde(default)]
    #[allow(dead_code)]
    image_tokens: u32,
}

/// Completion token details
#[derive(Debug, Deserialize, Clone)]
struct CompletionTokensDetails {
    #[serde(default)]
    reasoning_tokens: u32,
}

/// LLM analysis result for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAnalysisResult {
    /// Security rating (A-F)
    pub security_rating: String,
    /// Importance score (0.0-1.0)
    pub importance: f64,
    /// Issues found
    pub issues: Vec<LlmIssue>,
    /// Summary of analysis
    pub summary: String,
}

/// Issue found by LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmIssue {
    /// Issue severity
    pub severity: String,
    /// Issue category
    #[serde(default = "default_category")]
    pub category: String,
    /// Line number
    pub line: Option<usize>,
    /// Issue description (accepts both 'description' and 'message' from LLM)
    #[serde(alias = "message")]
    pub description: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

fn default_category() -> String {
    "CODE_QUALITY".to_string()
}

/// Codebase analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseAnalysisResult {
    /// Files that should be marked as deprecated
    pub deprecated_files: Vec<String>,
    /// Files missing type annotations
    pub missing_types: Vec<String>,
    /// Security concerns found
    pub security_concerns: Vec<String>,
    /// Architecture issues
    pub architecture_issues: Vec<String>,
    /// Recommended tags for files
    pub recommended_tags: Vec<FileTag>,
}

/// Recommended tag for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTag {
    /// File path
    pub file: String,
    /// Recommended tag type
    pub tag_type: String,
    /// Tag value
    pub value: String,
    /// Reason for recommendation
    pub reason: String,
}

/// Deep analysis result using 2M context window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepAnalysisResult {
    /// Logic drift issues
    pub logic_drift: Vec<AnalysisIssue>,
    /// Dead code findings
    pub dead_code: Vec<String>,
    /// Safety issues
    pub safety_issues: Vec<AnalysisIssue>,
    /// Incomplete code
    pub incomplete_code: Vec<AnalysisIssue>,
    /// Generated tasks
    pub tasks: Vec<GeneratedTask>,
}

/// Analysis issue from deep scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisIssue {
    /// File path
    pub file: String,
    /// Line number
    pub line: Option<usize>,
    /// Issue category
    #[serde(default = "default_category")]
    pub category: String,
    /// Description (accepts both 'description' and 'message' from LLM)
    #[serde(alias = "message")]
    pub description: String,
    /// Severity
    pub severity: String,
    /// Suggested fix (accepts both 'fix' and 'suggested_fix')
    #[serde(alias = "suggested_fix")]
    pub fix: Option<String>,
}

/// Task generated by LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTask {
    /// File path
    pub file: String,
    /// Line number
    pub line: Option<usize>,
    /// Priority
    pub priority: String,
    /// Category
    pub category: String,
    /// Description
    pub description: String,
    /// Suggested tag to inject
    pub suggested_tag: Option<String>,
}

/// File audit result from standard questionnaire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAuditResult {
    /// File path
    pub file: String,
    /// Is the file reachable/used?
    pub reachable: bool,
    /// Compliance issues found
    pub compliance_issues: Vec<String>,
    /// Is code incomplete?
    pub incomplete: bool,
    /// Suggested audit tags
    pub suggested_tags: Vec<String>,
    /// Improvement suggestion
    pub improvement: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_markdown() {
        let client = LlmClient::new(
            "test-key".to_string(),
            "grok-4-1-fast-reasoning".to_string(),
            4096,
            0.7,
        )
        .unwrap();

        let markdown = r#"
Here is the analysis:

```json
{
    "security_rating": "A",
    "importance": 0.9,
    "issues": [],
    "summary": "Good code"
}
```

That's the result.
"#;

        let json = client.extract_json_from_markdown(markdown).unwrap();
        assert!(json.contains("security_rating"));
    }

    #[test]
    fn test_build_system_prompt_janus() {
        let client = LlmClient::new(
            "test-key".to_string(),
            "grok-4-1-fast-reasoning".to_string(),
            4096,
            0.7,
        )
        .unwrap();

        let prompt = client.build_system_prompt(Category::Janus);
        assert!(prompt.contains("Neuromorphic"));
        assert!(prompt.contains("Decision-making"));
    }

    #[test]
    fn test_build_system_prompt_execution() {
        let client = LlmClient::new(
            "test-key".to_string(),
            "grok-4-1-fast-reasoning".to_string(),
            4096,
            0.7,
        )
        .unwrap();

        let prompt = client.build_system_prompt(Category::Execution);
        assert!(prompt.contains("Lightweight"));
        assert!(prompt.contains("signal"));
    }
}
