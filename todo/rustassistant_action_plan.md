# RustAssistant Action Plan

**Updated:** February 3, 2026  
**Current State:** Phase 1 Complete, Phase 2 at 75%  
**Deployment:** Running on Raspberry Pi via CI/CD  

---

## 📊 ACTUAL Current Status

### ✅ What's Already Working
- ✅ Core CLI with 50+ commands
- ✅ Note/Task/Repository CRUD
- ✅ Grok LLM integration with 70%+ cache hit rate
- ✅ Cost tracking (<$2/day)
- ✅ **Queue System - FULLY IMPLEMENTED** 
  - `rustassistant queue add/status/list/process/retry`
  - `rustassistant scan repos/todos/tree/analyze/all`
  - `rustassistant report todos/files/health/standardization`
- ✅ Code Review automation
- ✅ Test Generation
- ✅ **Refactoring Assistant - FULLY IMPLEMENTED**
- ✅ Health check endpoint at `/health`
- ✅ CI/CD to Raspberry Pi via Tailscale
- ✅ Docker multi-arch builds

### ❌ What's Actually Missing
- ❌ Documentation Generator (Phase 2 Feature 4)
- ❌ Integration tests for Phase 2 features
- ❌ Database migrations system
- ❌ Prometheus metrics endpoint
- ❌ CLI commands for refactor_assistant (code exists, CLI wiring missing)

---

## 🎯 REVISED Priority Actions

### THIS WEEK (High Impact, Quick Wins)

#### 1. Test Queue System End-to-End ⏱️ 1 hour
The code exists - just verify it works!

```bash
# Create test environment
export DATABASE_URL="sqlite:data/rustassistant.db"
export XAI_API_KEY="your_key_here"
export GITHUB_TOKEN="your_github_token"

# Test queue commands
rustassistant queue add "test thought" --source thought
rustassistant queue add "test note" --source note --project rustassistant
rustassistant queue status
rustassistant queue list inbox
rustassistant queue list pending --limit 10

# Test scan commands
rustassistant scan repos --token $GITHUB_TOKEN
rustassistant scan todos rustassistant
rustassistant scan tree rustassistant --depth 3
rustassistant scan all --token $GITHUB_TOKEN

# Test report commands
rustassistant report todos --priority 2
rustassistant report files --attention-only
rustassistant report health rustassistant
rustassistant report standardization rustassistant
```

**Expected outcome:** All commands work. If any fail, fix and commit.

---

#### 2. Wire Up Refactoring Assistant CLI ⏱️ 2 hours
Code exists in `src/refactor_assistant.rs`, just needs CLI commands.

**Add to `src/bin/cli.rs`:**

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...
    
    /// Refactoring assistant
    Refactor {
        #[command(subcommand)]
        action: RefactorAction,
    },
}

#[derive(Subcommand)]
enum RefactorAction {
    /// Analyze a file for refactoring opportunities
    Analyze {
        /// File path to analyze
        file: String,
    },
    
    /// Generate refactoring plan
    Plan {
        /// File path
        file: String,
        
        /// Specific smell ID to focus on
        #[arg(short, long)]
        smell: Option<String>,
    },
}

async fn handle_refactor_action(pool: &SqlitePool, action: RefactorAction) -> Result<()> {
    use rustassistant::refactor_assistant::RefactorAssistant;
    use rustassistant::db::Database;
    
    let db = Database::new(pool.clone())?;
    let assistant = RefactorAssistant::new(db).await?;
    
    match action {
        RefactorAction::Analyze { file } => {
            println!("🔍 Analyzing {} for refactoring opportunities...\n", file);
            
            let analysis = assistant.analyze_file(&file).await?;
            
            println!("📊 Refactoring Analysis:\n");
            println!("  {} {}", "File:".dimmed(), file);
            println!("  {} {}", "Smells Found:".dimmed(), analysis.code_smells.len());
            println!();
            
            for smell in &analysis.code_smells {
                let severity_icon = match smell.severity {
                    rustassistant::refactor_assistant::SmellSeverity::Critical => "🔴",
                    rustassistant::refactor_assistant::SmellSeverity::High => "🟠",
                    rustassistant::refactor_assistant::SmellSeverity::Medium => "🟡",
                    rustassistant::refactor_assistant::SmellSeverity::Low => "🟢",
                };
                
                println!("  {} {:?} (Line {})", severity_icon, smell.smell_type, smell.location.line);
                println!("     {}", smell.description);
                println!();
            }
            
            if !analysis.suggestions.is_empty() {
                println!("💡 Suggestions:");
                for (i, suggestion) in analysis.suggestions.iter().enumerate() {
                    println!("  {}. {} ({:?})", i + 1, suggestion.title, suggestion.refactoring_type);
                    println!("     Impact: {:?}, Effort: {:?}", suggestion.impact, suggestion.effort);
                    println!();
                }
            }
        }
        
        RefactorAction::Plan { file, smell } => {
            println!("📋 Generating refactoring plan for {}...\n", file);
            
            let analysis = assistant.analyze_file(&file).await?;
            
            let target_smell = if let Some(smell_id) = smell {
                analysis.code_smells.iter()
                    .find(|s| s.id == smell_id)
                    .ok_or_else(|| anyhow::anyhow!("Smell ID not found"))?
            } else {
                analysis.code_smells.first()
                    .ok_or_else(|| anyhow::anyhow!("No code smells found"))?
            };
            
            let plan = assistant.generate_plan(&analysis, &target_smell.id).await?;
            
            println!("📋 Refactoring Plan:\n");
            println!("  {} {}", "Title:".dimmed(), plan.title);
            println!("  {} {:?}", "Priority:".dimmed(), plan.priority);
            println!("  {} {:?}", "Estimated Effort:".dimmed(), plan.estimated_effort);
            println!();
            
            println!("Steps:");
            for (i, step) in plan.steps.iter().enumerate() {
                println!("  {}. {}", i + 1, step.description);
                println!("     Duration: {:?}", step.estimated_duration);
            }
            
            if !plan.risks.is_empty() {
                println!("\n⚠️  Risks:");
                for risk in &plan.risks {
                    println!("  • {} ({})", risk.description, risk.mitigation);
                }
            }
        }
    }
    
    Ok(())
}
```

**Then test:**
```bash
rustassistant refactor analyze src/server.rs
rustassistant refactor plan src/db.rs
```

---

#### 3. Create Documentation Generator ⏱️ 4-6 hours
This is the ONLY Phase 2 feature truly missing.

**Create `src/doc_generator.rs`:**

```rust
//! Documentation Generator
//!
//! Automatically generates documentation using LLM analysis.

use crate::db::Database;
use crate::llm::grok::GrokClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDoc {
    pub module_name: String,
    pub summary: String,
    pub functions: Vec<FunctionDoc>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDoc {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub parameters: Vec<ParameterDoc>,
    pub returns: String,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDoc {
    pub name: String,
    pub param_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmeContent {
    pub title: String,
    pub description: String,
    pub features: Vec<String>,
    pub installation: String,
    pub usage_examples: Vec<String>,
    pub architecture: String,
    pub contributing: String,
}

pub struct DocGenerator {
    grok_client: GrokClient,
}

impl DocGenerator {
    pub async fn new(db: Database) -> Result<Self> {
        let grok_client = GrokClient::from_env(db).await?;
        Ok(Self { grok_client })
    }

    /// Generate documentation for a Rust module/file
    pub async fn generate_module_docs(&self, file_path: impl AsRef<Path>) -> Result<ModuleDoc> {
        let file_path = file_path.as_ref();
        let content = std::fs::read_to_string(file_path)?;
        
        let prompt = format!(
            r#"Analyze this Rust code and generate comprehensive documentation.

File: {}

Code:
```rust
{}
```

Generate a JSON response with:
1. Module summary (2-3 sentences)
2. List of public functions with:
   - Function name
   - Full signature
   - Clear description
   - Parameter documentation
   - Return value description
   - Usage examples
3. Overall module examples

Format as JSON matching this structure:
{{
  "module_name": "string",
  "summary": "string",
  "functions": [
    {{
      "name": "string",
      "signature": "string",
      "description": "string",
      "parameters": [
        {{"name": "string", "param_type": "string", "description": "string"}}
      ],
      "returns": "string",
      "examples": ["string"]
    }}
  ],
  "examples": ["string"]
}}
"#,
            file_path.display(),
            content
        );

        let response = self.grok_client.analyze(&prompt, 4096).await?;
        let doc: ModuleDoc = serde_json::from_str(&response)
            .map_err(|e| anyhow::anyhow!("Failed to parse doc JSON: {}", e))?;
        
        Ok(doc)
    }

    /// Generate README.md content from repository analysis
    pub async fn generate_readme(&self, repo_path: impl AsRef<Path>) -> Result<ReadmeContent> {
        let repo_path = repo_path.as_ref();
        
        // Read key files
        let cargo_toml = repo_path.join("Cargo.toml");
        let src_lib = repo_path.join("src/lib.rs");
        let src_main = repo_path.join("src/main.rs");
        
        let mut context = String::new();
        
        if cargo_toml.exists() {
            context.push_str("Cargo.toml:\n");
            context.push_str(&std::fs::read_to_string(cargo_toml)?);
            context.push_str("\n\n");
        }
        
        if src_lib.exists() {
            context.push_str("src/lib.rs (first 200 lines):\n");
            let content = std::fs::read_to_string(src_lib)?;
            let lines: Vec<&str> = content.lines().take(200).collect();
            context.push_str(&lines.join("\n"));
            context.push_str("\n\n");
        } else if src_main.exists() {
            context.push_str("src/main.rs (first 200 lines):\n");
            let content = std::fs::read_to_string(src_main)?;
            let lines: Vec<&str> = content.lines().take(200).collect();
            context.push_str(&lines.join("\n"));
            context.push_str("\n\n");
        }
        
        let prompt = format!(
            r#"Generate a comprehensive README.md for this Rust project.

{}

Create a professional README with:
1. Project title and tagline
2. Clear description (what it does, why it exists)
3. Key features (bullet points)
4. Installation instructions
5. Usage examples with code
6. Architecture overview
7. Contributing guidelines

Format as JSON:
{{
  "title": "string",
  "description": "string",
  "features": ["string"],
  "installation": "string",
  "usage_examples": ["string"],
  "architecture": "string",
  "contributing": "string"
}}
"#,
            context
        );

        let response = self.grok_client.analyze(&prompt, 4096).await?;
        let readme: ReadmeContent = serde_json::from_str(&response)
            .map_err(|e| anyhow::anyhow!("Failed to parse README JSON: {}", e))?;
        
        Ok(readme)
    }

    /// Format README content as Markdown
    pub fn format_readme(&self, content: &ReadmeContent) -> String {
        let mut md = String::new();
        
        md.push_str(&format!("# {}\n\n", content.title));
        md.push_str(&format!("{}\n\n", content.description));
        
        md.push_str("## Features\n\n");
        for feature in &content.features {
            md.push_str(&format!("- {}\n", feature));
        }
        md.push_str("\n");
        
        md.push_str("## Installation\n\n");
        md.push_str(&content.installation);
        md.push_str("\n\n");
        
        md.push_str("## Usage\n\n");
        for example in &content.usage_examples {
            md.push_str(&format!("```rust\n{}\n```\n\n", example));
        }
        
        md.push_str("## Architecture\n\n");
        md.push_str(&content.architecture);
        md.push_str("\n\n");
        
        md.push_str("## Contributing\n\n");
        md.push_str(&content.contributing);
        md.push_str("\n");
        
        md
    }

    /// Format module documentation as Markdown
    pub fn format_module_doc(&self, doc: &ModuleDoc) -> String {
        let mut md = String::new();
        
        md.push_str(&format!("# Module: {}\n\n", doc.module_name));
        md.push_str(&format!("{}\n\n", doc.summary));
        
        if !doc.functions.is_empty() {
            md.push_str("## Functions\n\n");
            
            for func in &doc.functions {
                md.push_str(&format!("### `{}`\n\n", func.name));
                md.push_str(&format!("```rust\n{}\n```\n\n", func.signature));
                md.push_str(&format!("{}\n\n", func.description));
                
                if !func.parameters.is_empty() {
                    md.push_str("**Parameters:**\n\n");
                    for param in &func.parameters {
                        md.push_str(&format!(
                            "- `{}` (`{}`): {}\n",
                            param.name, param.param_type, param.description
                        ));
                    }
                    md.push_str("\n");
                }
                
                md.push_str(&format!("**Returns:** {}\n\n", func.returns));
                
                if !func.examples.is_empty() {
                    md.push_str("**Examples:**\n\n");
                    for example in &func.examples {
                        md.push_str(&format!("```rust\n{}\n```\n\n", example));
                    }
                }
            }
        }
        
        if !doc.examples.is_empty() {
            md.push_str("## Module Examples\n\n");
            for example in &doc.examples {
                md.push_str(&format!("```rust\n{}\n```\n\n", example));
            }
        }
        
        md
    }
}
```

**Add to `src/lib.rs`:**
```rust
pub mod doc_generator;
pub use doc_generator::{DocGenerator, ModuleDoc, ReadmeContent, FunctionDoc, ParameterDoc};
```

**Add CLI commands** to `src/bin/cli.rs`:
```rust
/// Documentation generator
Docs {
    #[command(subcommand)]
    action: DocsAction,
}

#[derive(Subcommand)]
enum DocsAction {
    /// Generate documentation for a module
    Module {
        /// File path
        file: String,
        
        /// Output file (optional, prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Generate README for repository
    Readme {
        /// Repository path (defaults to current directory)
        #[arg(default_value = ".")]
        repo: String,
        
        /// Output file (optional, prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },
}
```

**Test:**
```bash
rustassistant docs module src/db.rs
rustassistant docs readme . --output NEW_README.md
```

---

### NEXT WEEK (Testing & Stability)

#### 4. Add Integration Tests ⏱️ 4 hours

Create `tests/integration/phase2_features.rs`:

```rust
//! Integration tests for Phase 2 features

use rustassistant::db::Database;
use rustassistant::test_generator::TestGenerator;
use rustassistant::code_review::CodeReviewer;
use rustassistant::refactor_assistant::RefactorAssistant;
use rustassistant::doc_generator::DocGenerator;
use sqlx::SqlitePool;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_generator_works() {
    let pool = setup_test_db().await;
    let db = Database::new(pool).unwrap();
    let generator = TestGenerator::new(db).await.unwrap();
    
    // This will call the actual API - use a small test file
    let result = generator.generate_tests_for_file("tests/fixtures/sample.rs").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_code_reviewer_works() {
    let pool = setup_test_db().await;
    let db = Database::new(pool).unwrap();
    let reviewer = CodeReviewer::new(db).await.unwrap();
    
    let result = reviewer.review_file("tests/fixtures/sample.rs").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_refactor_assistant_works() {
    let pool = setup_test_db().await;
    let db = Database::new(pool).unwrap();
    let assistant = RefactorAssistant::new(db).await.unwrap();
    
    let result = assistant.analyze_file("tests/fixtures/sample.rs").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_doc_generator_works() {
    let pool = setup_test_db().await;
    let db = Database::new(pool).unwrap();
    let generator = DocGenerator::new(db).await.unwrap();
    
    let result = generator.generate_module_docs("tests/fixtures/sample.rs").await;
    assert!(result.is_ok());
}
```

Create `tests/fixtures/sample.rs` for testing.

---

#### 5. Database Migrations ⏱️ 2 hours

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features sqlite

# Create migrations
mkdir -p migrations
sqlx migrate add initial_schema
sqlx migrate add queue_tables
sqlx migrate add analysis_tables

# Copy current schema into migrations
# Then update CI to run migrations
```

---

#### 6. Prometheus Metrics ⏱️ 3 hours

Add to `Cargo.toml`:
```toml
prometheus = "0.13"
lazy_static = "1.4"
```

Create `src/metrics.rs`:
```rust
use prometheus::{Counter, Histogram, Registry, TextEncoder, Encoder};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REQUESTS_TOTAL: Counter = 
        Counter::new("http_requests_total", "Total HTTP requests").unwrap();
    
    pub static ref LLM_CALLS_TOTAL: Counter = 
        Counter::new("llm_calls_total", "Total LLM API calls").unwrap();
    
    pub static ref CACHE_HITS_TOTAL: Counter = 
        Counter::new("cache_hits_total", "Total cache hits").unwrap();
    
    pub static ref LLM_LATENCY: Histogram = 
        Histogram::new("llm_latency_seconds", "LLM call latency").unwrap();
}

pub fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

Add route to server:
```rust
.route("/metrics", get(|| async { metrics::metrics_handler() }))
```

---

## 📅 Realistic Timeline

```
Week 1 (NOW)
├── Day 1: Test all queue/scan/report commands (1 hour)
├── Day 2: Wire up refactor CLI commands (2 hours)
├── Day 3-4: Implement doc_generator (6 hours)
├── Day 5: Test all Phase 2 features
└── Weekend: PHASE 2 COMPLETE! 🎉

Week 2 (Testing)
├── Day 1-2: Integration tests
├── Day 3: Database migrations
├── Day 4-5: Prometheus metrics
└── Weekend: Performance testing

Week 3 (Polish)
├── Documentation updates
├── Bug fixes
├── Version 0.2.0 release
└── Plan Phase 3
```

---

## ✅ Updated Success Metrics

### Phase 2 Completion Checklist
- [x] Code Review ✅
- [x] Test Generator ✅
- [x] Refactoring Assistant ✅ (needs CLI wiring)
- [ ] Documentation Generator ❌ (in progress)

### By End of Week 1
- [ ] All queue commands verified working
- [ ] Refactor CLI commands working
- [ ] Doc generator implemented and tested
- [ ] Phase 2 at 100%

### By End of Week 2
- [ ] Integration tests passing
- [ ] Database migrations configured
- [ ] Metrics endpoint deployed

---

## 🚀 Phase 3 Preview

Once Phase 2 is done:

1. **LanceDB Integration** - Vector embeddings for notes/code
2. **Semantic Search** - Find similar code patterns
3. **Context Stuffing** - Use Grok's 2M token window
4. **Project Planning** - Generate plans from notes cluster

---

## 💡 Daily Workflow (Start Using It!)

```bash
# Morning
rustassistant queue status
rustassistant next

# During day
rustassistant queue add "your idea"
rustassistant note add "meeting notes" --tags meeting

# Evening
rustassistant queue process --batch-size 5
rustassistant report todos --priority 2
```

---

**You're 75% done with Phase 2. One more week and you're shipping! 🚀**