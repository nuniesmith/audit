# Implementation Guide: Documentation Generator

**Estimated Time:** 4-6 hours  
**Priority:** HIGH (last missing Phase 2 feature)  
**Status:** Not Started

---

## Overview

The Documentation Generator will use Grok to automatically create:
1. **Module documentation** - Analyze Rust files and generate comprehensive docs
2. **README generation** - Create project README from codebase analysis
3. **Markdown formatting** - Convert analysis into readable Markdown

---

## Step 1: Create the Module (30 min)

### Create `src/doc_generator.rs`

```rust
//! Documentation Generator
//!
//! Automatically generates documentation using LLM analysis.
//!
//! # Examples
//!
//! ```no_run
//! use rustassistant::doc_generator::DocGenerator;
//! use rustassistant::db::Database;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let pool = sqlx::SqlitePool::connect("sqlite:data/rustassistant.db").await?;
//!     let db = Database::new(pool)?;
//!     let generator = DocGenerator::new(db).await?;
//!     
//!     let docs = generator.generate_module_docs("src/db.rs").await?;
//!     println!("{}", generator.format_module_doc(&docs));
//!     
//!     Ok(())
//! }
//! ```

use crate::db::Database;
use crate::llm::grok::GrokClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

// [Add types from action plan here]
```

### Add to `src/lib.rs`

```rust
pub mod doc_generator;

pub use doc_generator::{
    DocGenerator, 
    ModuleDoc, 
    ReadmeContent, 
    FunctionDoc, 
    ParameterDoc
};
```

**Test:** `cargo check`

---

## Step 2: Define Data Types (30 min)

Add these types to `src/doc_generator.rs`:

```rust
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
```

**Test:** `cargo check`

---

## Step 3: Implement DocGenerator Struct (1 hour)

```rust
pub struct DocGenerator {
    grok_client: GrokClient,
}

impl DocGenerator {
    /// Create a new documentation generator
    pub async fn new(db: Database) -> Result<Self> {
        let grok_client = GrokClient::from_env(db).await?;
        Ok(Self { grok_client })
    }

    /// Generate documentation for a Rust module/file
    pub async fn generate_module_docs(&self, file_path: impl AsRef<Path>) -> Result<ModuleDoc> {
        let file_path = file_path.as_ref();
        let content = std::fs::read_to_string(file_path)?;
        
        let module_name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let prompt = self.build_module_doc_prompt(&module_name, &content);
        
        let response = self.grok_client.analyze(&prompt, 4096).await?;
        
        // Parse JSON response
        let doc: ModuleDoc = serde_json::from_str(&response)
            .map_err(|e| anyhow::anyhow!("Failed to parse module doc JSON: {}. Response: {}", e, response))?;
        
        Ok(doc)
    }

    /// Generate README content from repository analysis
    pub async fn generate_readme(&self, repo_path: impl AsRef<Path>) -> Result<ReadmeContent> {
        let repo_path = repo_path.as_ref();
        
        let context = self.build_repo_context(repo_path)?;
        let prompt = self.build_readme_prompt(&context);
        
        let response = self.grok_client.analyze(&prompt, 4096).await?;
        
        let readme: ReadmeContent = serde_json::from_str(&response)
            .map_err(|e| anyhow::anyhow!("Failed to parse README JSON: {}", e))?;
        
        Ok(readme)
    }

    // Helper methods (implement next)
    fn build_module_doc_prompt(&self, module_name: &str, content: &str) -> String {
        // TODO: Implement
        String::new()
    }

    fn build_repo_context(&self, repo_path: &Path) -> Result<String> {
        // TODO: Implement
        Ok(String::new())
    }

    fn build_readme_prompt(&self, context: &str) -> String {
        // TODO: Implement
        String::new()
    }
}
```

**Test:** `cargo check`

---

## Step 4: Implement Prompt Builders (1 hour)

```rust
impl DocGenerator {
    fn build_module_doc_prompt(&self, module_name: &str, content: &str) -> String {
        format!(
            r#"You are a documentation expert. Analyze this Rust code and generate comprehensive documentation.

File: {module_name}.rs

Code:
```rust
{content}
```

Generate a JSON response with:
1. Module summary (2-3 clear sentences explaining what this module does)
2. List of public functions with detailed documentation:
   - Function name
   - Full signature
   - Clear description of what it does
   - Parameter documentation (name, type, description)
   - Return value description
   - Usage examples
3. Overall module usage examples

Focus on public APIs. Be accurate and concise.

Respond ONLY with valid JSON matching this structure:
{{
  "module_name": "{module_name}",
  "summary": "Brief summary of the module",
  "functions": [
    {{
      "name": "function_name",
      "signature": "pub async fn name(param: Type) -> Result<T>",
      "description": "What this function does",
      "parameters": [
        {{"name": "param", "param_type": "Type", "description": "What it means"}}
      ],
      "returns": "Description of return value",
      "examples": ["example_code()"]
    }}
  ],
  "examples": ["module_level_example()"]
}}"#,
            module_name = module_name,
            content = content
        )
    }

    fn build_repo_context(&self, repo_path: &Path) -> Result<String> {
        let mut context = String::new();
        
        // Read Cargo.toml
        let cargo_toml = repo_path.join("Cargo.toml");
        if cargo_toml.exists() {
            context.push_str("=== Cargo.toml ===\n");
            context.push_str(&std::fs::read_to_string(cargo_toml)?);
            context.push_str("\n\n");
        }
        
        // Read src/lib.rs or src/main.rs (first 200 lines)
        let lib_rs = repo_path.join("src/lib.rs");
        let main_rs = repo_path.join("src/main.rs");
        
        if lib_rs.exists() {
            context.push_str("=== src/lib.rs (first 200 lines) ===\n");
            let content = std::fs::read_to_string(lib_rs)?;
            let lines: Vec<&str> = content.lines().take(200).collect();
            context.push_str(&lines.join("\n"));
            context.push_str("\n\n");
        } else if main_rs.exists() {
            context.push_str("=== src/main.rs (first 200 lines) ===\n");
            let content = std::fs::read_to_string(main_rs)?;
            let lines: Vec<&str> = content.lines().take(200).collect();
            context.push_str(&lines.join("\n"));
            context.push_str("\n\n");
        }
        
        // Read existing README if it exists
        let readme = repo_path.join("README.md");
        if readme.exists() {
            context.push_str("=== Existing README.md ===\n");
            let content = std::fs::read_to_string(readme)?;
            let lines: Vec<&str> = content.lines().take(50).collect();
            context.push_str(&lines.join("\n"));
            context.push_str("\n\n");
        }
        
        Ok(context)
    }

    fn build_readme_prompt(&self, context: &str) -> String {
        format!(
            r#"You are a technical writer. Generate a professional README.md for this Rust project.

Project Context:
{context}

Create a comprehensive README with:
1. Project title and one-line tagline
2. Clear description (2-3 paragraphs: what it does, why it exists, who it's for)
3. Key features (5-8 bullet points)
4. Installation instructions (cargo install, git clone, etc.)
5. Usage examples with actual code
6. Architecture overview
7. Contributing guidelines

Be professional, clear, and accurate.

Respond ONLY with valid JSON:
{{
  "title": "Project Name",
  "description": "Detailed description...",
  "features": [
    "Feature 1",
    "Feature 2"
  ],
  "installation": "Installation instructions as markdown",
  "usage_examples": [
    "code_example_1",
    "code_example_2"
  ],
  "architecture": "Architecture overview as markdown",
  "contributing": "Contributing guidelines as markdown"
}}"#,
            context = context
        )
    }
}
```

**Test:** `cargo check`

---

## Step 5: Implement Markdown Formatters (30 min)

```rust
impl DocGenerator {
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
}
```

**Test:** `cargo check && cargo build`

---

## Step 6: Add CLI Commands (1 hour)

### Add to `src/bin/cli.rs`

```rust
/// Documentation generator
Docs {
    #[command(subcommand)]
    action: DocsAction,
}
```

Add enum:

```rust
#[derive(Subcommand)]
enum DocsAction {
    /// Generate documentation for a module/file
    Module {
        /// File path
        file: String,
        
        /// Output file (prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Generate README for repository
    Readme {
        /// Repository path
        #[arg(default_value = ".")]
        repo: String,
        
        /// Output file (prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,
    },
}
```

Add handler:

```rust
async fn handle_docs_action(pool: &SqlitePool, action: DocsAction) -> anyhow::Result<()> {
    use rustassistant::doc_generator::DocGenerator;
    use rustassistant::db::Database;
    
    let db = Database::new(pool.clone())?;
    let generator = DocGenerator::new(db).await?;
    
    match action {
        DocsAction::Module { file, output } => {
            println!("📝 Generating documentation for {}...\n", file);
            
            let doc = generator.generate_module_docs(&file).await?;
            let markdown = generator.format_module_doc(&doc);
            
            if let Some(output_path) = output {
                std::fs::write(&output_path, &markdown)?;
                println!("{} Documentation written to {}", "✓".green(), output_path);
            } else {
                println!("{}", markdown);
            }
        }
        
        DocsAction::Readme { repo, output } => {
            println!("📖 Generating README for {}...\n", repo);
            
            let content = generator.generate_readme(&repo).await?;
            let markdown = generator.format_readme(&content);
            
            if let Some(output_path) = output {
                std::fs::write(&output_path, &markdown)?;
                println!("{} README written to {}", "✓".green(), output_path);
            } else {
                println!("{}", markdown);
            }
        }
    }
    
    Ok(())
}
```

Wire it up in main:

```rust
Commands::Docs { action } => handle_docs_action(&pool, action).await?,
```

**Test:** `cargo build --release`

---

## Step 7: Test End-to-End (30 min)

```bash
# Set API key
export XAI_API_KEY="your_key_here"

# Test module documentation
./target/release/rustassistant docs module src/db.rs

# Test README generation
./target/release/rustassistant docs readme .

# Test with output file
./target/release/rustassistant docs module src/db.rs --output DB_DOCS.md
./target/release/rustassistant docs readme . --output NEW_README.md

# Verify output
cat DB_DOCS.md
cat NEW_README.md
```

---

## Step 8: Add Tests (30 min)

Create `tests/integration/doc_generator_test.rs`:

```rust
use rustassistant::db::Database;
use rustassistant::doc_generator::DocGenerator;
use sqlx::SqlitePool;

async fn setup() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_doc_generator_creates() {
    let pool = setup().await;
    let db = Database::new(pool).unwrap();
    let _generator = DocGenerator::new(db).await.unwrap();
}

#[tokio::test]
#[ignore] // Requires API key
async fn test_generate_module_docs() {
    let pool = setup().await;
    let db = Database::new(pool).unwrap();
    let generator = DocGenerator::new(db).await.unwrap();
    
    let doc = generator.generate_module_docs("src/db.rs").await.unwrap();
    
    assert!(!doc.module_name.is_empty());
    assert!(!doc.summary.is_empty());
}

#[tokio::test]
#[ignore] // Requires API key
async fn test_generate_readme() {
    let pool = setup().await;
    let db = Database::new(pool).unwrap();
    let generator = DocGenerator::new(db).await.unwrap();
    
    let content = generator.generate_readme(".").await.unwrap();
    
    assert!(!content.title.is_empty());
    assert!(!content.description.is_empty());
}

#[test]
fn test_format_module_doc() {
    use rustassistant::doc_generator::*;
    
    let doc = ModuleDoc {
        module_name: "test".to_string(),
        summary: "Test module".to_string(),
        functions: vec![],
        examples: vec![],
    };
    
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    let db = Database::new(pool).unwrap();
    let generator = DocGenerator::new(db).await.unwrap();
    
    let markdown = generator.format_module_doc(&doc);
    assert!(markdown.contains("# Module: test"));
}
```

**Run:** `cargo test doc_generator`

---

## Step 9: Documentation (30 min)

Update `README.md`:

```markdown
### Documentation Generator

Generate comprehensive documentation using AI:

```bash
# Generate module documentation
rustassistant docs module src/db.rs

# Generate README
rustassistant docs readme .

# Save to file
rustassistant docs module src/db.rs --output docs/DB.md
rustassistant docs readme . --output NEW_README.md
```
```

Update `CLI_CHEATSHEET.md`:

```markdown
## Documentation Commands

| Command | Description |
|---------|-------------|
| `rustassistant docs module <file>` | Generate documentation for a module |
| `rustassistant docs readme <repo>` | Generate README for repository |
| `rustassistant docs module <file> -o output.md` | Save docs to file |
```

---

## Step 10: Commit & Deploy (15 min)

```bash
# Stage changes
git add src/doc_generator.rs
git add src/lib.rs
git add src/bin/cli.rs
git add tests/integration/doc_generator_test.rs
git add README.md
git add CLI_CHEATSHEET.md

# Commit
git commit -m "feat: implement documentation generator (Phase 2 Feature 4)

- Add DocGenerator with module and README generation
- Implement Grok-based analysis for comprehensive docs
- Add CLI commands: docs module, docs readme
- Add Markdown formatters
- Add integration tests
- Update documentation

Phase 2 is now 100% complete!"

# Push (CI/CD will deploy)
git push origin main
```

---

## Verification Checklist

- [ ] `cargo check` passes
- [ ] `cargo build --release` succeeds
- [ ] `cargo test doc_generator` passes
- [ ] `rustassistant docs module src/db.rs` works
- [ ] `rustassistant docs readme .` works
- [ ] Output is valid Markdown
- [ ] Documentation is accurate
- [ ] CLI help text is clear
- [ ] Changes committed and pushed

---

## Troubleshooting

### JSON Parsing Errors
- Grok might not return valid JSON
- Add error handling to retry or fall back
- Log the raw response for debugging

### File Not Found
- Check file paths are relative to repo root
- Handle missing files gracefully

### API Rate Limits
- Add caching for documentation requests
- Implement retry logic with backoff

---

## Success Criteria

✅ **Phase 2 Feature 4 Complete** when:
1. Module documentation generation works
2. README generation works
3. CLI commands are intuitive
4. Output is high-quality Markdown
5. Tests pass
6. Documentation updated

**Estimated completion: 4-6 hours**

---

## Next Steps After Completion

1. Test on real repos
2. Generate docs for rustassistant itself
3. Add to daily workflow
4. Update STATUS.md to "Phase 2 Complete"
5. Tag version v0.2.0-beta
6. Plan Phase 3! 🎉