# Audit Service - Rust Implementation

> **✨ NEW FEATURES:**
> - 🧠 **Claude Opus 4.5 Support**: Anthropic's most capable model for JANUS whitepaper conformity audits
> - 📋 **JANUS Audit Command**: Deep analysis of neuromorphic trading system against whitepaper specs
> - ✨ **Auto-Format**: Automatically format code across Rust, Kotlin, TypeScript, and Python
> - 📝 **TODO Scanner**: Automatically find and prioritize all TODO/FIXME/HACK comments
> - 🤖 **LLM File Rating**: AI-powered security and quality ratings for files
> - 📋 **LLM Questionnaire**: Comprehensive file-by-file audits with standardized questions
> - 🎯 **Enhanced CI Modes**: Standard and Full audit modes with customizable options
> - 🔬 **Research Pipeline**: Transform research materials into structured implementation plans and tasks
> - 🎨 **Neuromorphic Visualization**: Generate brain-inspired architecture diagrams with Mermaid
>
> See [Quick Reference Card](./QUICK_REFERENCE_CARD.md), [Enhanced Features Guide](./ENHANCED_AUDIT_FEATURES.md), [Research Pipeline Guide](./docs/RESEARCH_PIPELINE.md), or [Visualization Guide](./docs/NEUROMORPHIC_VISUALIZATION.md)

A high-performance code audit service for static analysis and LLM-powered code review. Built with Rust and Axum, designed for CI/CD integration and API-only access.

> **⚠️ Architecture Update:** This service now runs in **API-only mode** with no web UI. Static audits run automatically in CI, and deep LLM audits are triggered manually from GitHub Actions.

## 🎯 Features

### Core Capabilities

- **✨ Auto-Format**: Multi-language code formatting with `cargo fmt`, `ktlint`, `prettier`, and `black`
- **🏷️ Static Tag Detection**: Scan for custom audit annotations (`@audit-tag`, `@audit-todo`, `@audit-freeze`, etc.)
- **📝 TODO Scanner**: Find all TODO/FIXME/HACK/XXX/NOTE comments with automatic priority detection
- **🔍 Static Analysis**: Fast pattern-based analysis for Rust, Python, Kotlin, and infrastructure files
- **🤖 Multi-Provider LLM**: Deep code analysis using **XAI Grok**, **Anthropic Claude**, or **Google Gemini**
- **🧠 Claude Opus 4.5**: Anthropic's best model for high-stakes auditing and whitepaper conformity
- **⭐ LLM File Rating**: Get security and importance ratings for individual files or batches
- **📋 LLM Questionnaire**: Run standardized audits checking reachability, compliance, and completeness
- **🔬 Research Pipeline**: Ingest papers, articles, notes and generate implementation plans with actionable tasks
- **🎨 **Neuromorphic Visualization**: Generate Mermaid flowcharts mapping code to biological brain regions
- **📋 Task Generation**: Automatically generate actionable tasks from findings
- **🌐 REST API**: API-only server (no web UI) for programmatic access
- **💻 CLI Tool**: Command-line interface for local audits and CI/CD integration
- **📊 Multi-Format Output**: JSON, CSV, and human-readable text formats
- **🔄 GitHub Actions**: Manual LLM audit workflows with standard/full modes

### Analysis Types

#### Static Analysis (Fast Path)
- Rust: `unwrap()`, `panic!()`, unsafe blocks, async safety
- Python: bare `except`, `eval()`, SQL injection patterns
- Docker: root user, `latest` tags, hardcoded secrets
- Infrastructure: credential exposure, security misconfigurations

#### LLM Analysis (Deep Path)
- Security vulnerability detection
- Architecture quality assessment
- Code complexity analysis
- Deprecated code identification
- Type safety recommendations

#### JANUS Whitepaper Conformity (Claude Opus 4.5)
- GAF transformation mathematics verification
- LTN Łukasiewicz logic implementation check
- Brain-region component mapping validation
- Memory hierarchy conformity (hippocampus, SWR, neocortex)
- Compliance constraint verification (wash sale, position limits)

### Tag System

Custom annotations for tracking code status:

```rust
// @audit-tag: new | old | experimental | deprecated
// @audit-todo: Task description
// @audit-freeze
// @audit-review: Review notes
// @audit-security: Security concern
```

Example usage:

```rust
// @audit-tag: experimental
// @audit-security: Validate all user input before processing
pub fn process_data(input: &str) -> Result<Data> {
    // @audit-todo: Add input validation
    Ok(Data::new(input))
}

// @audit-freeze
const CRITICAL_CONSTANT: u64 = 42;
```

## 🚀 Quick Start

### 0. Auto-Format Code (FREE, Instant)

```bash
cd src/audit
# Check formatting without making changes
cargo run --release --bin audit-cli -- format ../.. --check

# Apply formatting fixes
cargo run --release --bin audit-cli -- format ../..

# Format specific languages only
cargo run --release --bin audit-cli -- format ../.. -f rust -f kotlin
```

**What you get:**
- Automatic formatting for Rust, Kotlin, TypeScript/JavaScript, and Python
- Check mode to see what needs formatting (CI-friendly)
- Fix mode to apply formatting changes
- Runs in CI/CD to keep code consistent
- JSON output for automation

**Supported formatters:**
- **Rust**: `cargo fmt` (rustfmt)
- **Kotlin**: `ktlint`
- **TypeScript/JavaScript**: `prettier`
- **Python**: `black`

### 1. Find All TODOs (FREE, Instant)

```bash
cd src/audit
cargo run --release --bin audit-cli -- todo ../../src/janus
```

**What you get:**
- All TODO, FIXME, HACK, XXX, NOTE comments
- Automatic priority classification (High/Medium/Low)
- Grouped by file and category
- Export to JSON/CSV for tracking

### 2. Visualize System Architecture (FREE, Instant)

```bash
cd src/audit
cargo run --release --bin audit-cli -- visualize ../..
```

**What you get:**
- Interactive Mermaid diagram of neuromorphic architecture
- Maps code modules to biological brain regions (Sensory, Memory, Executive, Limbic, Action, Output, Meta)
- Shows data flow pathways through the system
- Perfect for documentation and onboarding
- Export to `.mmd` file for embedding in docs

### 2. Rate Files with AI (Paid, Fast)

```bash
# Set API key
export XAI_API_KEY=your_key_here

# Rate a critical file
cargo run --release --bin audit-cli -- rate ../../src/janus/crates/cns/src/brain.rs --provider xai

# Batch rate multiple files (faster)
cargo run --release --bin audit-cli -- rate ../../src/janus/services --batch --provider xai
```

**What you get:**
- Security rating per file
- Importance level assessment
- Detected issues with severity
- Actionable suggestions

### 3. Run Full Audit with Questionnaire (Paid, Comprehensive)

```bash
cargo run --release --bin audit-cli -- question ../../src/janus --provider xai --output audit.json
```

**What you get:**
- File reachability analysis (find dead code)
- Compliance issue detection
- Incomplete code identification
- Suggested audit tags
- Improvement recommendations

### 4. Test xAI Connection (2 minutes)

```bash
cd src/audit

# Set your API key
export XAI_API_KEY="xai-your-key-here"

# Run the test suite
./scripts/test-xai.sh
```

**Expected output:**
```
✅ API responding: OK
✅ Response path correct (.output[0].content[0].text)
✅ Content is valid JSON
✅ Usage statistics present
✅ All tests passed!
```

### Run Local Audit (3 minutes)

```bash
# Build the service
cargo build --release

# Test with a file
export RUST_LOG=debug
export AUDIT_DEBUG_DIR=./debug
./target/release/audit-cli analyze --file src/scanner.rs --provider xai

# Check cost
cat debug/llm-response.json | jq '.usage'
```

### Trigger CI Audit (2 minutes)

```bash
# Quick audit (fast, ~$0.02)
gh workflow run llm-audit.yml -f mode=quick -f llm_provider=xai

# Standard audit (balanced, ~$0.06)
gh workflow run llm-audit.yml -f mode=standard -f llm_provider=xai

# Monitor and download
gh run watch
gh run download
```

### 📊 What You Get

- **Cost Tracking:** Full breakdown of token usage and costs
- **Caching:** 50% savings on repeat runs (automatic)
- **Debug Output:** All responses saved for troubleshooting
- **Multiple Formats:** Works with xAI, Google, and legacy APIs

### 📚 Documentation

- **Implementation Guide:** [docs/IMPLEMENTATION_GUIDE.md](docs/IMPLEMENTATION_GUIDE.md)
- **Detailed Analysis:** [../docs/audit/XAI_INTEGRATION_IMPROVEMENTS.md](../docs/audit/XAI_INTEGRATION_IMPROVEMENTS.md)
- **Troubleshooting:** [../docs/audit/XAI_TROUBLESHOOTING.md](../docs/audit/XAI_TROUBLESHOOTING.md)

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.92.0 or later
- Git
- Optional: API key for LLM analysis
  - **XAI Grok**: Get from [console.x.ai](https://console.x.ai)
  - **Google Gemini**: Get from [aistudio.google.com](https://aistudio.google.com)

### Installation

```bash
cd fks/src/audit
cargo build --release
```

### Environment Setup

Create a `.env` file:

```env
# Server Configuration
AUDIT_HOST=0.0.0.0
AUDIT_PORT=8080

# LLM Configuration
LLM_ENABLED=true
LLM_PROVIDER=xai          # Options: xai | google
LLM_MODEL=grok-4-1-fast-reasoning       # or: gemini-2.0-flash-exp
LLM_MAX_TOKENS=4096
LLM_TEMPERATURE=0.3       # Lower = more focused

# API Keys (use one based on provider)
XAI_API_KEY=xai-xxx       # For XAI Grok
GOOGLE_API_KEY=AIza-xxx   # For Google Gemini

# Git Configuration
GIT_WORKSPACE_DIR=./workspace
GIT_DEFAULT_BRANCH=main
GIT_SHALLOW_CLONE=true

# Scanner Configuration
SCANNER_MAX_FILE_SIZE=1000000
SCANNER_INCLUDE_TESTS=true      # Include tests by default for comprehensive analysis

# Storage Configuration
STORAGE_REPORTS_DIR=./reports
STORAGE_TASKS_DIR=./tasks
```

## 📖 Usage

### 1. Static Audit (Automated in CI)

Every push to `main`/`develop` automatically runs static analysis. Check the **Actions** tab on GitHub for results.

### 2. LLM Audit (Manual Trigger)

**Trigger deep AI-powered analysis from GitHub:**

1. Go to **Actions** → **🤖 LLM Audit**
2. Click **Run workflow**
3. Choose provider: `xai` or `google`
4. Configure depth (standard/deep)
5. Includes all focus areas and test files by default
6. Download artifacts when complete

See [LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md) for detailed instructions.

### 3. API Server (For Custom Integration)

Start the API-only server:

```bash
export XAI_API_KEY="xai-..."  # or GOOGLE_API_KEY
export LLM_ENABLED="true"
cargo run --bin audit-server
```

Server runs on `http://localhost:8080` (API endpoints only, no web UI)

#### API Endpoints

##### Health Check
```bash
curl http://localhost:8080/health
```

##### Create Audit
```bash
curl -X POST http://localhost:8080/api/audit \
  -H "Content-Type: application/json" \
  -d '{
    "repository": "https://github.com/user/repo",
    "branch": "main",
    "enable_llm": true,
    "focus": [],
    "include_tests": false
  }'
```

##### Get Audit Report
```bash
curl http://localhost:8080/api/audit/{id}
```

##### Get Tasks
```bash
curl http://localhost:8080/api/audit/{id}/tasks
```

##### Clone Repository
```bash
curl -X POST http://localhost:8080/api/clone \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://github.com/user/repo",
    "branch": "main"
  }'
```

##### Scan Tags Only
```bash
curl -X POST http://localhost:8080/api/scan/tags \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/path/to/code"
  }'
```

##### Static Analysis Only
```bash
curl -X POST http://localhost:8080/api/scan/static \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/path/to/code"
  }'
```

##### Research Analysis (Content)
```bash
curl -X POST http://localhost:8080/api/research/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "content": "# Research Topic\n\nYour research text here...",
    "title": "research_topic",
    "generate_tasks": true
  }'
```

##### Research Analysis (File)
```bash
curl -X POST http://localhost:8080/api/research/file \
  -H "Content-Type: application/json" \
  -d '{
    "file_path": "research/inbox/paper.md",
    "generate_tasks": true
  }'
```

##### Neuromorphic Visualization
```bash
curl -X POST http://localhost:8080/api/visualize/neuromorphic \
  -H "Content-Type: application/json" \
  -d '{
    "path": "."
  }'
```

##### Component Visualization
```bash
curl -X POST http://localhost:8080/api/visualize/component \
  -H "Content-Type: application/json" \
  -d '{
    "path": ".",
    "component": "src/execution"
  }'
```

### CLI Tool

The CLI provides standalone functionality for local and CI usage.

#### Full Audit

```bash
# Audit a local repository
# Audit current directory (includes tests by default)
cargo run --bin audit-cli -- audit /path/to/repo

# Audit with LLM analysis (includes tests by default)
cargo run --bin audit-cli -- audit /path/to/repo --llm

# Audit a remote repository
cargo run --bin audit-cli -- audit https://github.com/user/repo --branch main

# Exclude tests if needed
cargo run --bin audit-cli -- audit /path/to/repo --exclude-tests

# Save report to file
cargo run --bin audit-cli -- audit . --output report.json --format json
```

#### Scan for Tags

```bash
# Scan current directory
cargo run --bin audit-cli -- tags .

# Filter by tag type
cargo run --bin audit-cli -- tags . --tag-type todo

# Output as CSV
cargo run --bin audit-cli -- tags . --format csv --output tags.csv
```

#### Static Analysis

```bash
# Run fast static analysis
cargo run --bin audit-cli -- static .

# Focus on specific areas
cargo run --bin audit-cli -- static . --focus security --focus performance

# Output as JSON
cargo run --bin audit-cli -- static . --format json --output issues.json
```

#### Generate Tasks

```bash
# Generate tasks from tags and issues
cargo run --bin audit-cli -- tasks .

# Output as CSV for import to project management tools
cargo run --bin audit-cli -- tasks . --format csv --output tasks.csv
```

#### Clone Repository

```bash
# Clone for analysis
cargo run --bin audit-cli -- clone https://github.com/user/repo

# Clone specific branch
cargo run --bin audit-cli -- clone https://github.com/user/repo --branch develop --name my-repo
```

#### Show Statistics

```bash
# Display codebase statistics
cargo run --bin audit-cli -- stats .
```

#### Research Pipeline

Transform research materials into structured implementation plans:

```bash
# Analyze a research file and generate implementation plan
cargo run --bin audit-cli -- research research/inbox/new_strategy.md

# Generate tasks automatically from research
cargo run --bin audit-cli -- research research/inbox/websocket_library.md --generate-tasks

# Custom output directory
cargo run --bin audit-cli -- research paper.md --output docs/strategies/

# JSON output for programmatic processing
cargo run --bin audit-cli -- research paper.md --format json
```

**What you get:**
- Structured implementation plan with architecture integration
- Technical requirements and dependencies
- Step-by-step implementation guide
- Optional: Actionable task list in JSON format
- JANUS-aware analysis (Rust/Python/Kotlin components)

**Workflow:**
1. Save research material to `research/inbox/`
2. Run analysis: `cargo run --bin audit-cli -- research research/inbox/file.md --generate-tasks`
3. Review breakdown: `cat docs/research_breakdowns/file_PLAN.md`
4. Review tasks: `cat docs/research_breakdowns/file_PLAN.tasks.json`

See [Research Pipeline Guide](./docs/RESEARCH_PIPELINE.md) for detailed documentation.

#### Neuromorphic Visualization

Generate brain-inspired architecture diagrams:

```bash
# Generate full neuromorphic architecture map
cargo run --bin audit-cli -- visualize .

# Save to file for documentation
cargo run --bin audit-cli -- visualize . -o brain_map.mmd

# Visualize specific component
cargo run --bin audit-cli -- visualize . --diagram-type component --component src/execution

# JSON output for programmatic use
cargo run --bin audit-cli -- visualize . --format json
```

**What you get:**
- Mermaid flowchart mapping code to brain regions
- Biological metaphor (Thalamus=Gating, Hippocampus=Memory, Basal Ganglia=Action Selection)
- Visual data flow through Sensory → Memory → Executive → Limbic → Action → Output
- Module detection summary with counts by region
- Ready for embedding in GitHub/GitLab markdown or MkDocs

**Workflow:**
1. Run visualization: `cargo run --bin audit-cli -- visualize . -o docs/architecture.mmd`
2. View at [mermaid.live](https://mermaid.live/) or embed in markdown
3. Use for onboarding, presentations, and architecture reviews

See [Neuromorphic Visualization Guide](./docs/NEUROMORPHIC_VISUALIZATION.md) for detailed documentation.

## 🔧 CI/CD Integration

### Automated Static Audits

Static audits run automatically on every push via `.github/workflows/ci.yml`. No setup required!

### Manual LLM Audits

Deep AI-powered audits available via `.github/workflows/llm-audit.yml`:

**Prerequisites:**
1. Add GitHub repository secrets:
   - `XAI_API_KEY` (for XAI Grok)
   - `GOOGLE_API_KEY` (for Google Gemini)

**Trigger:**
1. Go to Actions → 🤖 LLM Audit
2. Click "Run workflow"
3. Select provider and options
   - **Default:** All focus areas (security, logic, performance, compliance, architecture)
   - **Default:** Includes test files (for comprehensive analysis)
4. Download artifacts

See [LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md) for complete documentation.

### Custom CI Integration

```yaml
- name: Run Code Audit
  working-directory: src/audit
  run: |
    # Build CLI
    cargo build --release --bin audit-cli
    
    # Run static analysis (fast, no LLM)
    ./target/release/audit-cli static ../.. --format json --output audit-report.json
    
    # Generate tasks
    ./target/release/audit-cli tasks ../.. --format csv --output tasks.csv
    
    # Optional: Fail on critical issues
    if jq -e '.issues_by_severity.Critical > 0' audit-report.json; then
      echo "❌ Critical issues found"
      exit 1
    fi

- name: Upload Audit Report
  uses: actions/upload-artifact@v4
  with:
    name: audit-report
    path: |
      src/audit/audit-report.json
      src/audit/tasks.csv
```

### Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
echo "Running audit checks..."

# Scan for new TODO tags
audit-cli tags . --tag-type todo --format text

# Run static analysis on changed files
audit-cli static . --format text

echo "✓ Audit checks complete"
```

## 📊 Output Formats

### JSON
Full structured data for programmatic processing:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "repository": "/path/to/repo",
  "branch": "main",
  "created_at": "2025-01-01T00:00:00Z",
  "summary": {
    "total_files": 150,
    "total_lines": 25000,
    "total_issues": 42,
    "total_tasks": 15,
    "critical_files": 3
  },
  "files": [...],
  "tasks": [...],
  "issues_by_severity": {...}
}
```

### CSV
Tabular data for spreadsheets and import tools:

```csv
ID,Title,File,Line,Priority,Category,Tags
TASK-A1B2C3D4,TODO: Add validation,src/main.rs,42,High,Rust,todo;from-tag
```

### Text
Human-readable console output:

```
📋 Audit Report: 550e8400-e29b-41d4-a716-446655440000
═══════════════════════════════════════
Repository:       /path/to/repo
Branch:           main
Created:          2025-01-01 00:00:00 UTC

Summary:
  Total Files:    150
  Total Lines:    25000
  Total Issues:   42
  Total Tasks:    15
  Critical Files: 3
```

## 🏗️ Architecture

```
audit/
├── src/
│   ├── lib.rs              # Library exports
│   ├── config.rs           # Configuration management
│   ├── error.rs            # Error types
│   ├── types.rs            # Core data types
│   ├── tags.rs             # Tag scanner
│   ├── tasks.rs            # Task generator
│   ├── scanner.rs          # File system scanner
│   ├── parser.rs           # Code parser (tree-sitter)
│   ├── llm.rs              # LLM client (Grok)
│   ├── git.rs              # Git operations
│   ├── server.rs           # Axum web server
│   └── bin/
│       ├── server.rs       # Server binary
│       └── cli.rs          # CLI binary
├── Cargo.toml
└── README.md
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_scan_rust_file

# Test with coverage
cargo tarpaulin --out Html
```

## 🐳 Docker

### Build Image

```bash
docker build -t audit-service -f ../../docker/Dockerfile .
```

### Run Container

```bash
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/workspace:/app/workspace \
  -v $(pwd)/reports:/app/reports \
  -e XAI_API_KEY=your-key \
  audit-service
```

### Docker Compose

```yaml

services:
  audit:
    build:
      context: .
      dockerfile: docker/Dockerfile
    ports:
      - "8080:8080"
    environment:
      - AUDIT_PORT=8080
      - XAI_API_KEY=${XAI_API_KEY}
      - LLM_ENABLED=true
    volumes:
      - ./workspace:/app/workspace
      - ./reports:/app/reports
      - ./tasks:/app/tasks
```

## 🔒 Security

### API Key Management

- Never hardcode API keys
- Use environment variables
- For production, use secrets management (HashiCorp Vault, AWS Secrets Manager)

### Permissions

The service needs:
- Read access to repositories
- Write access to workspace/reports/tasks directories
- Network access for LLM API calls

## 📈 Performance

### Benchmarks

On a typical Rust codebase (50k LOC, including tests):

- **Static Analysis**: ~3 seconds
- **Tag Scanning**: ~700ms
- **LLM Analysis** (10 files + tests): ~45 seconds
- **Task Generation**: ~100ms

### Optimization Tips

- Use `--no-llm` for fast CI checks
- Tests are included by default for better analysis (use `--exclude-tests` only if necessary)
- Set `SCANNER_MAX_FILE_SIZE` appropriately
- Use `GIT_SHALLOW_CLONE=true` for faster clones

## 🛠️ Development

### Adding New Static Checks

1. Edit `src/scanner.rs`
2. Add check method (e.g., `check_rust_issues`)
3. Add to `static_analysis()` match statement
4. Write tests

### Adding New Tag Types

1. Edit `src/types.rs` - add to `AuditTagType` enum
2. Edit `src/tags.rs` - add regex pattern
3. Update documentation

### Adding Language Support

1. Edit `src/types.rs` - add to `Category` enum
2. Edit `src/scanner.rs` - add analysis methods
3. Edit `src/llm.rs` - add system prompt
4. Add tree-sitter parser in `src/parser.rs`

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## 📝 License

Part of the FKS Trading Platform. See main repository for license details.

## 🆘 Support

- **Issues**: Use GitHub Issues
- **Docs**: See `/docs/audit/` in main repository
- **Examples**: Check `/examples/audit/`

## 🗺️ Roadmap

- [x] API-only architecture (no web UI)
- [x] Multi-provider LLM support (XAI + Google)
- [x] GitHub Actions manual LLM audits
- [x] Static audits in CI
- [ ] Tree-sitter integration for deeper parsing
- [ ] GitHub App for automatic PR audits
- [ ] Support for more languages (Go, Java, C++)
- [ ] Custom rule configuration
- [ ] Incremental analysis (only changed files)
- [ ] Auto-create GitHub Issues from tasks
- [ ] Integration with Jira/Linear
- [ ] Caching for repeated analyses
- [ ] Distributed scanning for large codebases

## 📚 Related Documentation

### xAI Integration (Latest!)

- **[Implementation Guide](docs/IMPLEMENTATION_GUIDE.md)** - Step-by-step setup and testing
- **[Improvements Analysis](../docs/audit/XAI_INTEGRATION_IMPROVEMENTS.md)** - Full technical analysis
- **[Completed Improvements](../docs/audit/IMPROVEMENTS_COMPLETED.md)** - What was fixed
- **[xAI Notes](../docs/audit/notes_xai.md)** - API documentation and pricing

### JANUS-Specific Documentation

- **[JANUS Context](./JANUS_CONTEXT.md)** - Comprehensive technical reference for LLM audits
  - Project overview and architecture
  - Mathematical specifications (GAF, LTN, PER, UMAP, Circuit Breakers)
  - Brain-region mappings
  - Performance requirements and benchmarks
  - Audit checklists and common issues

- **[Technical Paper Integration](./TECHNICAL_PAPER_INTEGRATION.md)** - Guide for using the JANUS paper
  - Equation-to-code mapping
  - Section-by-section audit guidance
  - Verification patterns and examples
  - Common error patterns with fixes

- **[JANUS LLM System Prompt](./JANUS_LLM_SYSTEM_PROMPT.md)** - Enhanced AI audit prompt
  - Complete JANUS context for LLMs
  - Mathematical correctness guidelines
  - Safety and compliance requirements
  - Issue categorization system

- **[JANUS Audit Quick Reference](./JANUS_AUDIT_QUICK_REF.md)** - One-page developer cheat sheet
  - Critical formulas to verify
  - Quick audit checklist
  - Common issues and fixes
  - Performance targets

- **[Audit Update Summary](./AUDIT_UPDATE_SUMMARY.md)** - Documentation of JANUS integration
  - All changes made in v2.0 update
  - Usage impact and metrics
  - Migration guide and best practices

### General Audit Documentation

- **[LLM Audit Guide](./LLM_AUDIT_GUIDE.md)** - Complete guide for AI-powered audits
- **[Quick Start](./QUICK_START.md)** - Getting started guide
- **[Quick Reference](./QUICK_REFERENCE.md)** - Command cheatsheet
- [CI/CD Workflows](../../.github/workflows/) - GitHub Actions integration
- [Static Audit CI Docs](../../docs/CI_STATIC_AUDIT.md) - Automated audit documentation

### External Resources

- [JANUS Technical Paper](https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex) - Complete mathematical specifications (LaTeX)
- [JANUS Project README](../../janus/README.md) - JANUS service overview
- [CNS Architecture](../../janus/docs/CNS_ARCHITECTURE.md) - Central Nervous System monitoring

## 🔑 Key Differences from Web UI Version

This service is now **API-only**:

✅ **What's Available:**
- REST API endpoints for all audit functions
- CLI tool for local and CI usage
- Automated static audits in CI
- Manual LLM audits via GitHub Actions
- Multi-provider LLM support (XAI + Google)

❌ **What's Removed:**
- Web UI / HTML interface
- Static file serving
- Browser-based audit interface

**Why?** API-only architecture is:
- More secure (smaller attack surface)
- Easier to deploy and scale
- Better for automation and CI/CD
- Cloud-native and containerizable
# audit
