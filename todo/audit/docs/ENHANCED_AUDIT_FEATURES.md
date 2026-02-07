# Enhanced Audit Features

This document describes the new features added to the audit service for comprehensive code analysis.

## Table of Contents

- [Overview](#overview)
- [New Features](#new-features)
- [CLI Commands](#cli-commands)
- [CI/CD Integration](#cicd-integration)
- [Usage Examples](#usage-examples)
- [Configuration](#configuration)

## Overview

The audit service now supports enhanced analysis capabilities including:

1. **TODO Comment Scanning** - Automatically find and prioritize TODO items in your codebase
2. **LLM-Powered File Rating** - Get AI-driven quality and security ratings for your code
3. **Standard Questionnaire** - Run comprehensive audits using standardized questions
4. **Enhanced CI Modes** - Standard and Full audit modes with customizable options

## New Features

### 1. TODO Scanner

Scans your codebase for TODO comments with automatic priority detection.

**Features:**
- Detects `TODO:`, `FIXME:`, `HACK:`, `XXX:`, `NOTE:` comments
- Automatic priority inference (High/Medium/Low)
- Support for multiple comment styles (line comments, block comments)
- Multi-language support (Rust, Python, TypeScript, Kotlin, etc.)
- Grouping by file, category, and priority

**Priority Levels:**
- **High**: FIXME, XXX, security-related, urgent items
- **Medium**: Standard TODO items
- **Low**: NOTE, optional improvements, future considerations

### 2. LLM File Rating

Uses AI to analyze and rate individual files or batches of files.

**Ratings Include:**
- Security rating
- Importance level
- Issue detection with severity
- Actionable suggestions
- Summary and recommendations

**Modes:**
- **Individual**: Analyze files one by one (good for detailed analysis)
- **Batch**: Analyze multiple files together (faster, good for overview)

### 3. LLM Questionnaire

Runs a comprehensive audit using standardized questions for every file.

**Questions Cover:**
- Is this code reachable/used?
- Are there compliance issues?
- Is the implementation incomplete?
- What tags should be applied?
- What improvements are recommended?

### 4. Enhanced CI Workflow

The GitHub Actions workflow now supports:

**Audit Modes:**
- **Standard**: Quick analysis focusing on critical issues (12K tokens)
- **Full**: Comprehensive analysis with questionnaire (16K tokens)

**Manual Workflow Inputs:**
- `mode`: Choose standard or full audit
- `llm_provider`: Select xAI (Grok) or Google (Gemini)
- `scan_todos`: Enable/disable TODO scanning (default: enabled)
- `run_questionnaire`: Run LLM questionnaire in full mode (default: disabled)
- `max_files`: Limit number of files to analyze (0 = all)
- `focus_area`: Target specific areas (security, performance, reliability, code_quality)

## CLI Commands

### TODO Scanning

```bash
# Scan for all TODOs
cargo run --bin audit-cli -- todo .

# Filter by priority
cargo run --bin audit-cli -- todo . --priority high

# Output to JSON
cargo run --bin audit-cli -- todo . --output todos.json --format json

# Output to CSV
cargo run --bin audit-cli -- todo . --format csv
```

### LLM File Rating

```bash
# Rate a single file
cargo run --bin audit-cli -- rate src/main.rs --provider xai

# Rate all files in a directory (individual mode)
cargo run --bin audit-cli -- rate src/ --provider xai

# Batch mode (faster for multiple files)
cargo run --bin audit-cli -- rate src/ --provider xai --batch

# Output to file
cargo run --bin audit-cli -- rate src/ --output ratings.json
```

### LLM Questionnaire

```bash
# Run questionnaire on all files
cargo run --bin audit-cli -- question src/ --provider xai

# Use Google Gemini instead
cargo run --bin audit-cli -- question src/ --provider google

# Save results
cargo run --bin audit-cli -- question src/ --output questionnaire.json
```

### Existing Commands

```bash
# Full audit with LLM
cargo run --bin audit-cli -- audit . --llm

# Scan for audit tags
cargo run --bin audit-cli -- tags .

# Static analysis only (fast, no LLM)
cargo run --bin audit-cli -- static .

# Generate tasks
cargo run --bin audit-cli -- tasks .

# Show statistics
cargo run --bin audit-cli -- stats .
```

## CI/CD Integration

### GitHub Actions Workflow

The `.github/workflows/llm-audit.yml` workflow can be triggered manually with custom options:

1. Go to Actions → LLM Audit → Run workflow
2. Select options:
   - **Mode**: standard (quick) or full (comprehensive)
   - **LLM Provider**: xai or google
   - **Scan TODOs**: true/false
   - **Run Questionnaire**: true/false (full mode only)
   - **Max Files**: 0 for all, or a specific limit
   - **Focus Area**: empty for all, or specific category

### Standard Mode

Best for: Regular CI checks, pull request reviews

**What it does:**
- Static code analysis
- TODO scanning
- Audit tag detection
- LLM analysis of critical security/safety issues
- ~12K tokens per run

**Example:**
```yaml
mode: standard
llm_provider: xai
scan_todos: true
run_questionnaire: false
max_files: 100
focus_area: security
```

### Full Mode

Best for: Weekly deep dives, major releases, security audits

**What it does:**
- Everything in Standard mode
- Extended LLM analysis (~16K tokens)
- Optional file-by-file questionnaire
- Comprehensive reporting
- Dead code detection
- Architecture review

**Example:**
```yaml
mode: full
llm_provider: xai
scan_todos: true
run_questionnaire: true
max_files: 0  # All files
focus_area: ""  # All areas
```

## Usage Examples

### Example 1: Find High-Priority TODOs

```bash
# Scan and show high-priority items
cargo run --bin audit-cli -- todo ./src/janus --priority high

# Output to JSON for processing
cargo run --bin audit-cli -- todo ./src/janus --priority high --output high-priority-todos.json
```

**Output:**
```
📝 TODO Items Found: 15
═══════════════════════════════════════
🔴 High Priority:   5
🟡 Medium Priority: 8
🟢 Low Priority:    2
📁 Files with TODOs: 12

🔴 High Priority TODOs:
  1. [src/brain.rs:278] Integrate with alerting system (PagerDuty, Slack, etc.)
  2. [src/probes.rs:176] Implement gRPC health check protocol
  3. [gateway/janus_client.py:7] Generate Python gRPC stubs from proto file
  ...
```

### Example 2: Rate Security-Critical Files

```bash
# Set API key
export XAI_API_KEY=your_key_here

# Rate specific files
cargo run --bin audit-cli -- rate src/audit/src/llm.rs --provider xai

# Batch rate all Rust files in a module
cargo run --bin audit-cli -- rate src/janus/crates/cns --batch --provider xai
```

**Output:**
```
🤖 LLM File Ratings
═══════════════════════════════════════
Files Analyzed: 3

📄 src/janus/crates/cns/src/brain.rs
  Security Rating: Medium
  Importance: High
  Summary: Core brain orchestration logic with reflex actions
  Issues: 5
    • [Medium] TODO comments indicate incomplete features (line 278)
    • [Low] Consider adding error handling for component restart (line 283)
    • [Info] Throttling logic not yet implemented (line 293)
```

### Example 3: Run Full Questionnaire

```bash
# Run questionnaire on entire codebase
export XAI_API_KEY=your_key_here
cargo run --bin audit-cli -- question ./src/janus --provider xai --output audit-results.json
```

**Output:**
```
📋 LLM Questionnaire Results
═══════════════════════════════════════
Files Audited: 47

📄 src/janus/services/gateway/src/clients/janus_client.py
  Reachable: ✅ Yes
  Compliance Issues: 0
  Incomplete: ⚠️ Yes
  Suggested Tags: @audit-todo, @audit-review
  Improvement: Implement gRPC client stubs - currently commented out

📄 src/janus/crates/cns/src/probes.rs
  Reachable: ✅ Yes
  Compliance Issues: 0
  Incomplete: ⚠️ Yes
  Suggested Tags: @audit-todo
  Improvement: Implement proper gRPC health check instead of TCP connection check
```

### Example 4: CI/CD Integration

Add to your CI pipeline:

```yaml
- name: Scan for High-Priority TODOs
  run: |
    cd src/audit
    cargo run --release --bin audit-cli -- todo ../../src --priority high --output todos.json
    HIGH_COUNT=$(cat todos.json | jq 'length')
    
    if [ "$HIGH_COUNT" -gt 10 ]; then
      echo "::warning::Found $HIGH_COUNT high-priority TODOs"
    fi

- name: Rate Changed Files
  run: |
    cd src/audit
    git diff --name-only origin/main...HEAD | grep '\.rs$' > changed_files.txt
    while read file; do
      cargo run --release --bin audit-cli -- rate "../../$file" --provider xai
    done < changed_files.txt
```

## Configuration

### Environment Variables

```bash
# Required for LLM features
export XAI_API_KEY=xai-your-api-key        # For Grok
export GOOGLE_API_KEY=your-google-api-key  # For Gemini

# Optional
export RUST_LOG=info                        # Set log level
```

### Output Formats

All commands support three output formats:

- **Text** (default): Human-readable console output
- **JSON**: Machine-readable structured data
- **CSV**: Spreadsheet-compatible tabular data

Use `--format json|csv|text` flag or global `-f` flag.

## Artifact Outputs

When running in CI, the workflow generates:

1. **AUDIT_REPORT.md** - Complete audit report with all findings
2. **tasks.csv** - Importable task list for project management
3. **todos.json** - All TODO items with metadata
4. **questionnaire-results.json** - Detailed file-by-file audit (full mode)
5. **static-analysis.json** - Static code analysis results
6. **audit-tags.json** - All audit tags found in codebase
7. **cost-history.jsonl** - Token usage and cost tracking

All artifacts are:
- Uploaded to GitHub Actions (90-day retention)
- Committed to `docs/audit/` directory
- Indexed in `docs/audit/README.md`

## Best Practices

### TODO Management

1. **Use consistent formats**: Stick to `TODO:`, `FIXME:`, etc.
2. **Add context**: Include description and severity indicators
3. **Regular cleanup**: Review and address high-priority TODOs weekly
4. **Link to issues**: Reference GitHub issues when possible

```rust
// Good TODOs:
// TODO: Implement rate limiting for API endpoints (see #123)
// FIXME: Security vulnerability in authentication - urgent!
// NOTE: Consider caching this result in future optimization

// Avoid:
// TODO
// fix this
```

### LLM Rating Usage

- **Standard mode**: Use for PR reviews and regular checks
- **Full mode**: Use for releases, security audits, architecture reviews
- **Batch mode**: When you need overview of many files quickly
- **Individual mode**: For detailed analysis of critical components

### Questionnaire Strategy

- Run questionnaire monthly or before major releases
- Focus on unreachable code - candidates for removal
- Track compliance issues for regulatory requirements
- Use suggested tags to improve codebase organization

## Troubleshooting

### Common Issues

**No TODOs found:**
- Check that files have supported extensions (.rs, .py, .kt, etc.)
- Verify TODO comment format (e.g., `// TODO:` not `//TODO`)
- Ensure directories aren't being skipped (target/, node_modules/, etc.)

**LLM API errors:**
- Verify API key is set correctly
- Check API rate limits
- Try switching providers (`--provider google` instead of `--provider xai`)

**Out of context window:**
- Reduce `max_files` parameter
- Use batch mode for large codebases
- Focus on specific directories or files

## Cost Optimization

### Token Usage

- **Standard mode**: ~8-12K tokens ($0.10-0.15 per run with Grok)
- **Full mode**: ~12-16K tokens ($0.15-0.20 per run with Grok)
- **Questionnaire**: ~100-200 tokens per file

### Tips

1. Use static analysis first (free, fast)
2. Run TODO scanner locally before CI (no cost)
3. Use `max_files` to limit scope
4. Leverage prompt caching (xAI supports this)
5. Run full audits weekly, standard daily

## Future Enhancements

Planned features:

- [ ] Integration with issue trackers (GitHub Issues, Jira)
- [ ] Auto-generate pull requests for simple fixes
- [ ] Custom questionnaire templates
- [ ] Historical trend analysis
- [ ] Multi-language LLM support for non-English codebases
- [ ] Incremental analysis (only changed files)
- [ ] VSCode extension for inline TODO highlighting

## Support

For questions or issues:

1. Check existing [documentation](./README.md)
2. Review [examples](./examples/)
3. Open an issue on GitHub
4. Check the [audit service logs](./docs/audit/)

---

**Last Updated**: 2024
**Version**: 0.1.0