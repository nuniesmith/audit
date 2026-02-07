# Audit Tag System Documentation

## Overview

The FKS Audit Tag System provides a structured way to annotate, categorize, and track code across the entire codebase. It enables automated detection of technical debt, security concerns, old code, and other issues through a schema-based tagging approach.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Tag Types](#tag-types)
3. [Tag Schema](#tag-schema)
4. [Usage Examples](#usage-examples)
5. [CLI Commands](#cli-commands)
6. [GitHub Actions Integration](#github-actions-integration)
7. [Directory Tree Visualization](#directory-tree-visualization)
8. [Simple Issue Detection](#simple-issue-detection)
9. [Best Practices](#best-practices)
10. [Advanced Topics](#advanced-topics)

---

## Quick Start

### Adding Tags to Your Code

```rust
// @audit-tag: new,security
fn validate_api_key(key: &str) -> Result<bool> {
    // Implementation
}

// @audit-todo: Add proper error handling and logging
fn risky_operation() {
    // @audit-security: Input validation required
    let data = read_untrusted_input();
}

// @audit-freeze
const TRADING_LIMITS: f64 = 1000.0;

// @audit-review: Performance optimization needed
fn expensive_computation() {
    // Implementation
}
```

### Running a Tag Scan

```bash
# Scan current directory for all tags
cargo run --bin audit-cli -- tags .

# Scan with JSON output
cargo run --bin audit-cli -- tags . --output tags.json

# Filter by tag type
cargo run --bin audit-cli -- tags . --tag-type todo
```

### Generating Directory Tree

```bash
# Basic tree (5 levels deep)
cargo run --bin audit-cli -- tree .

# Tree with audit tags
cargo run --bin audit-cli -- tree . --with-tags --depth 10

# Save tree to JSON
cargo run --bin audit-cli -- tree . --output tree.json
```

---

## Tag Types

### 1. `@audit-tag:`

**Purpose:** General code categorization and status tracking

**Format:** `@audit-tag: status[,category][,priority]`

**Examples:**
```rust
// @audit-tag: new
// New code (< 3 months)

// @audit-tag: old,debt
// Old code that needs refactoring

// @audit-tag: experimental,perf
// Experimental performance optimization

// @audit-tag: deprecated,security,high
// Deprecated security-sensitive code (high priority)
```

**Valid Statuses:**
- `new` - New code (< 3 months)
- `active` - Under active development
- `stable` - Production-ready, stable code
- `deprecated` - Marked for removal
- `old` - Old code (> 1 year)
- `very-old` - Very old code (> 2 years)
- `needs-review` - Requires code review
- `frozen` - Do not modify
- `experimental` - Prototype/experimental

**Valid Categories:**
- `security` / `sec` - Security concerns
- `performance` / `perf` - Performance optimization
- `risk` - Risk management
- `debt` / `tech-debt` - Technical debt
- `docs` - Documentation
- `testing` - Tests
- `legacy` / `old` - Legacy code
- `experimental` / `exp` - Experimental
- `config` - Configuration

### 2. `@audit-todo:`

**Purpose:** Track TODO items and tasks

**Format:** `@audit-todo: description`

**Examples:**
```rust
// @audit-todo: Implement retry logic with exponential backoff
fn send_request() { }

// @audit-todo: Add comprehensive unit tests
fn calculate_risk_score() { }

// @audit-todo: Extract into separate module
fn complex_function() { }
```

### 3. `@audit-freeze`

**Purpose:** Mark code that should never be modified

**Format:** `@audit-freeze`

**Examples:**
```rust
// @audit-freeze
// Critical trading constants - DO NOT MODIFY
pub const MAX_POSITION_SIZE: f64 = 10000.0;
pub const RISK_LIMIT: f64 = 0.02;

// @audit-freeze
// Proven algorithm - any changes require full regression testing
fn calculate_portfolio_risk() -> f64 {
    // Implementation validated over 2 years
}
```

### 4. `@audit-review:`

**Purpose:** Request code review or flag areas needing attention

**Format:** `@audit-review: review notes`

**Examples:**
```rust
// @audit-review: Check thread safety in concurrent scenarios
async fn process_orders() { }

// @audit-review: Verify edge cases are handled
fn parse_market_data(data: &str) -> Result<MarketData> { }
```

### 5. `@audit-security:`

**Purpose:** Mark security-sensitive code or concerns

**Format:** `@audit-security: security concern description`

**Examples:**
```rust
// @audit-security: Validate all API inputs to prevent injection
fn execute_query(query: &str) { }

// @audit-security: Ensure proper authentication before processing
fn handle_withdrawal_request() { }

// @audit-security: Rate limiting required
fn api_endpoint() { }
```

---

## Tag Schema

### CodeStatus Enum

```rust
pub enum CodeStatus {
    New,           // < 3 months
    Active,        // Under development
    Stable,        // Production-ready
    Deprecated,    // Marked for removal
    Old,           // > 1 year
    VeryOld,       // > 2 years
    NeedsReview,   // Requires review
    Frozen,        // Do not modify
    Experimental,  // Prototype
    Unknown,       // Status unclear
}
```

### TagCategory Enum

```rust
pub enum TagCategory {
    Organization,    // Code structure
    Security,        // Security concerns
    Performance,     // Performance optimization
    Risk,           // Risk management
    TechnicalDebt,  // Technical debt
    Documentation,  // Documentation
    Testing,        // Tests
    Legacy,         // Old/deprecated code
    Experimental,   // New/experimental
    Configuration,  // Config files
}
```

### CodeAge Classification

```rust
pub enum CodeAge {
    Fresh,      // < 1 month
    Recent,     // 1-3 months
    Moderate,   // 3-6 months
    Mature,     // 6-12 months
    Old,        // 1-2 years
    VeryOld,    // 2+ years
}
```

### Complexity Level

```rust
pub enum Complexity {
    Simple,     // Straightforward code
    Moderate,   // Moderate complexity
    Complex,    // Complex logic
    Critical,   // Very complex/critical
}
```

---

## Usage Examples

### Example 1: Marking Old Code for Refactoring

```rust
// @audit-tag: old,debt,high
// @audit-review: This module hasn't been updated in 18 months
// @audit-todo: Refactor to use new async architecture
pub mod legacy_data_processor {
    // Old synchronous code
    pub fn process_data(data: Vec<u8>) -> Result<Output> {
        // TODO: Replace with async stream processing
    }
}
```

### Example 2: Security-Sensitive Code

```rust
// @audit-tag: stable,security
// @audit-security: Critical authentication logic - any changes require security review
// @audit-freeze
pub fn verify_user_credentials(username: &str, password: &str) -> Result<Session> {
    // Validated security implementation
    // DO NOT MODIFY without security team review
}
```

### Example 3: Experimental Feature

```rust
// @audit-tag: experimental,perf
// @audit-review: Monitor performance in production
// @audit-todo: Add benchmarks and profiling
pub fn experimental_order_matching() {
    // New high-performance matching algorithm
    // Still being evaluated
}
```

### Example 4: Deprecated API

```rust
// @audit-tag: deprecated,legacy,high
// @audit-todo: Migrate all callers to new_api_v2
#[deprecated(since = "2.0.0", note = "Use new_api_v2 instead")]
pub fn old_api() {
    // Kept for backward compatibility
    // Remove in version 3.0
}
```

---

## CLI Commands

### Scan for Tags

```bash
# Basic scan
cargo run --bin audit-cli -- tags /path/to/project

# With JSON output
cargo run --bin audit-cli -- tags . --output audit-tags.json

# Filter by specific tag type
cargo run --bin audit-cli -- tags . --tag-type security
```

### Scan for TODOs

```bash
# All TODOs
cargo run --bin audit-cli -- todo .

# High priority TODOs only
cargo run --bin audit-cli -- todo . --priority high

# Export to JSON
cargo run --bin audit-cli -- todo . --output todos.json
```

### Generate Directory Tree

```bash
# Basic tree visualization
cargo run --bin audit-cli -- tree .

# With audit tags included
cargo run --bin audit-cli -- tree . --with-tags

# Custom depth and JSON output
cargo run --bin audit-cli -- tree . --depth 10 --with-tags --output tree.json
```

### Static Analysis

```bash
# Full static analysis
cargo run --bin audit-cli -- static .

# Focus on specific area
cargo run --bin audit-cli -- static . --focus "security"

# With JSON report
cargo run --bin audit-cli -- static . --output static-analysis.json
```

---

## GitHub Actions Integration

### Adding Focus Point to Workflow

The workflow now accepts an optional focus point to guide the LLM analysis:

```yaml
name: 🤖 LLM Audit
on:
  workflow_dispatch:
    inputs:
      llm_provider:
        description: "LLM Provider"
        type: choice
        default: "xai"
        options:
          - xai
          - google
      focus_point:
        description: "Optional focus area"
        required: false
        type: string
        default: ""
```

### Example Focus Points

When manually triggering the workflow, you can specify:

- `"security"` - Focus on security vulnerabilities
- `"risk management"` - Focus on trading risk and safety
- `"JANUS ML"` - Focus on machine learning components
- `"old code cleanup"` - Focus on finding and refactoring old code
- `"performance"` - Focus on performance optimizations
- `"technical debt"` - Focus on identifying technical debt

### Workflow Steps

The enhanced workflow includes:

1. **Tag Scanning** - Scans for all audit tags
2. **TODO Scanning** - Finds and categorizes TODO comments
3. **Static Analysis** - Detects simple issues
4. **LLM Analysis** - Deep analysis with optional focus point
5. **Report Generation** - Creates comprehensive reports

---

## Directory Tree Visualization

### Tree Structure

The directory tree provides:

- **Hierarchical view** of your codebase
- **Issue counts** per file/directory (🔴 critical, 🟠 high)
- **Tag indicators** (🏷️ count)
- **Line counts** for each file
- **Hotspot detection** (files/dirs with most issues)

### Example Output

```
📁 src
├── 📁 janus [1250 LOC] 🟠2 🏷️5
│   ├── 📁 brain [800 LOC] 🟠1 🏷️3
│   │   ├── 📄 amygdala.rs [200 LOC] 🔴1 🏷️1
│   │   ├── 📄 hippocampus.rs [350 LOC] 🏷️1
│   │   └── 📄 prefrontal.rs [250 LOC] 🟠1 🏷️1
│   ├── 📄 forward.rs [300 LOC] 🏷️1
│   └── 📄 backward.rs [150 LOC] 🏷️1
└── 📁 common [450 LOC]
    └── 📄 types.rs [450 LOC]
```

### Tree Statistics

Each tree scan provides:

- Total files analyzed
- Total lines of code
- Total TODOs and FIXMEs
- Total audit tags
- Total issues by severity
- Number of directories analyzed
- Top 10 issue hotspots

---

## Simple Issue Detection

The system automatically detects common code issues:

### Detected Patterns

1. **`.unwrap()`** - Medium severity
   - May cause panics
   - Suggests proper error handling

2. **`.expect("")`** - Low severity
   - Empty expect messages
   - Suggests adding context

3. **`TODO` comments** - Info
   - Tracks as technical debt

4. **`FIXME` comments** - Medium severity
   - Requires attention

5. **`XXX` comments** - High severity
   - Critical issue markers

6. **`unsafe` blocks** - High severity
   - Requires careful review

7. **`println!`** - Low severity
   - Should use proper logging

8. **`thread::sleep`** - Medium severity
   - Blocking in async contexts

9. **`#[deprecated]`** - Info
   - Plans for removal needed

10. **Clone in loops** - Medium severity
    - Performance impact

---

## Best Practices

### 1. Tag Consistently

```rust
// ✅ GOOD: Clear, structured tags
// @audit-tag: stable,security
// @audit-security: Input validation performed
fn process_input(data: &str) -> Result<Output> { }

// ❌ BAD: No tags on security-sensitive code
fn process_input(data: &str) -> Result<Output> { }
```

### 2. Keep Tags Updated

```rust
// ❌ BAD: Outdated tags
// @audit-tag: new
// (Code written 2 years ago)

// ✅ GOOD: Accurate status
// @audit-tag: old,debt
// @audit-todo: Needs modernization
```

### 3. Use Freeze Wisely

```rust
// ✅ GOOD: Freeze critical constants
// @audit-freeze
pub const MAX_RISK: f64 = 0.02;

// ❌ BAD: Freezing everything
// @audit-freeze
fn helper_function() { } // No need to freeze
```

### 4. Provide Context

```rust
// ✅ GOOD: Detailed TODO
// @audit-todo: Replace synchronous HTTP client with async reqwest
//               to prevent blocking the executor. Expected in v2.0.

// ❌ BAD: Vague TODO
// @audit-todo: Fix this
```

### 5. Review Tags Regularly

Schedule periodic reviews of:
- All `old` and `very-old` tags
- High-priority security tags
- Frozen code (is it still needed?)
- Deprecated code (ready to remove?)

---

## Advanced Topics

### Custom Tag Validation

```rust
use audit::tag_schema::validate_tag;

let result = validate_tag("new,security,high");
if result.is_valid {
    println!("Tag is valid");
} else {
    for error in result.errors {
        eprintln!("Error: {}", error);
    }
    for suggestion in result.suggestions {
        println!("Suggestion: {}", suggestion);
    }
}
```

### Programmatic Tree Building

```rust
use audit::directory_tree::DirectoryTreeBuilder;
use audit::tags::TagScanner;

let builder = DirectoryTreeBuilder::new("./src");
let scanner = TagScanner::new()?;
let tags = scanner.scan_directory("./src")?;

let tree = builder.build_with_tags(&tags)?;
let summary = builder.generate_summary(&tree);

println!("Total files: {}", summary.total_files);
println!("Total issues: {}", summary.total_issues);

// Find hotspots
let hotspots = builder.find_hotspots(&tree, 10);
for hotspot in hotspots {
    println!("{}: {} issues", hotspot.name, hotspot.total_issues);
}
```

### Custom Issue Detection

```rust
use audit::tag_schema::SimpleIssueDetector;

let detector = SimpleIssueDetector::new();

for pattern in detector.patterns() {
    println!("{}: {}", pattern.name, pattern.description);
}
```

### Filtering by Age

```rust
use audit::tag_schema::{CodeAge, CodeStatus};

let status = CodeStatus::Old;
if status.is_technical_debt() {
    println!("This code needs attention!");
}

let age = CodeAge::from_months(18);
match age {
    CodeAge::Old | CodeAge::VeryOld => {
        println!("Consider refactoring");
    }
    _ => {}
}
```

---

## Integration Examples

### CI/CD Pipeline

```yaml
- name: Scan Audit Tags
  run: |
    cargo run --bin audit-cli -- tags . --output tags.json
    TAG_COUNT=$(jq 'length' tags.json)
    echo "Found $TAG_COUNT audit tags"

- name: Check for High Priority Issues
  run: |
    cargo run --bin audit-cli -- tree . --with-tags --output tree.json
    CRITICAL=$(jq '.issues.critical' tree.json)
    if [ "$CRITICAL" -gt 0 ]; then
      echo "::error::Found $CRITICAL critical issues"
      exit 1
    fi
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check for audit tags in staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(rs|py|kt)$')

if [ -n "$STAGED_FILES" ]; then
    echo "Checking for audit tags..."
    cargo run --bin audit-cli -- tags . --output /tmp/tags.json
    
    # Check for new TODO items
    NEW_TODOS=$(jq '[.[] | select(.tag_type == "Todo")] | length' /tmp/tags.json)
    
    if [ "$NEW_TODOS" -gt 10 ]; then
        echo "Warning: $NEW_TODOS TODO items found"
    fi
fi
```

---

## Troubleshooting

### Tags Not Detected

**Problem:** Tags in comments not being found

**Solution:** Ensure tags follow exact format:
- Start with `@audit-`
- No spaces in tag name
- Colon after tag name (except `@audit-freeze`)

### Large Directories Slow

**Problem:** Tree generation takes too long

**Solution:** 
- Use `--depth` to limit recursion
- Exclude large directories in `.gitignore`
- Run on specific subdirectories

### JSON Parse Errors

**Problem:** Cannot parse tag JSON output

**Solution:**
- Check file encoding (should be UTF-8)
- Verify no special characters in tag values
- Use `jq` to validate JSON structure

---

## Future Enhancements

Planned features:

1. **Tag History** - Track tag changes over time
2. **Tag Analytics** - Dashboard showing tag distribution
3. **Auto-tagging** - ML-based tag suggestions
4. **Tag Inheritance** - Directory-level tags
5. **Custom Tag Types** - User-defined tag schemas
6. **Tag Enforcement** - CI checks for required tags
7. **Tag Migration** - Bulk tag updates
8. **Visual Studio Code Extension** - IDE integration

---

## Reference

### Complete Tag Format Reference

```
@audit-tag: <status>[,<category>][,<priority>]
@audit-todo: <description>
@audit-freeze
@audit-review: <review notes>
@audit-security: <security concern>
```

### Status Values
`new`, `active`, `stable`, `deprecated`, `old`, `very-old`, `needs-review`, `frozen`, `experimental`

### Category Values
`security`, `performance`, `risk`, `debt`, `docs`, `testing`, `legacy`, `experimental`, `config`

### Priority Values
`critical`, `high`, `medium`, `low`

---

## Contributing

To add new tag types or patterns:

1. Update `src/tag_schema.rs` with new enum variants
2. Add regex patterns to `TagScanner`
3. Update this documentation
4. Add tests for new patterns
5. Submit PR with examples

---

## Support

For questions or issues:

- GitHub Issues: [fks/issues](https://github.com/nuniesmith/fks/issues)
- Documentation: `src/audit/docs/`
- Examples: `src/audit/examples/`

---

**Last Updated:** 2025-01-07
**Version:** 1.0.0