# Research Pipeline

The Research Pipeline extends the audit service to handle content ingestion and synthesis, transforming raw research materials (papers, articles, notes, web scrapes) into structured implementation plans and actionable tasks.

## Overview

The Research Pipeline reuses the existing LLM infrastructure to:

1. **Ingest** unstructured text from various sources
2. **Analyze** content using context-aware AI prompts
3. **Generate** structured implementation plans
4. **Extract** actionable coding tasks
5. **Integrate** findings with the JANUS architecture

## Architecture

```
research/inbox/          → Raw research materials (.md, .txt)
       ↓
[Research Module]        → LLM analysis with custom prompts
       ↓
docs/research_breakdowns/ → Structured markdown plans
       ↓
*.tasks.json             → Actionable task lists
```

### Components

- **`research.rs`**: Core module handling analysis and task extraction
- **`config.rs`**: Extended with `ResearchConfig` for prompts and settings
- **CLI**: `audit research` command for local usage
- **API**: `/api/research/*` endpoints for web integration

## Usage

### CLI Usage

#### Basic Analysis

Analyze a research file and generate an implementation plan:

```bash
./target/release/audit research research/inbox/new_strategy.md
```

Output:
- `docs/research_breakdowns/new_strategy_PLAN.md` - Full breakdown

#### With Task Extraction

Generate tasks automatically:

```bash
./target/release/audit research research/inbox/websocket_library.md --generate-tasks
```

Output:
- `docs/research_breakdowns/websocket_library_PLAN.md` - Implementation plan
- `docs/research_breakdowns/websocket_library_PLAN.tasks.json` - Task list

#### Custom Output Directory

Specify where to save the breakdown:

```bash
./target/release/audit research paper.md -o docs/strategies/
```

#### JSON Output Format

Get JSON output for programmatic processing:

```bash
./target/release/audit research paper.md --format json
```

### API Usage

#### Analyze Direct Content

```bash
curl -X POST http://localhost:8080/api/research/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "content": "# WebSocket Trading Strategy\n\nImplement real-time...",
    "title": "websocket_strategy",
    "generate_tasks": true
  }'
```

Response:
```json
{
  "breakdown": {
    "title": "websocket_strategy",
    "executive_summary": "...",
    "technical_requirements": "...",
    "architecture_integration": "...",
    "implementation_steps": ["...", "..."],
    "markdown_content": "..."
  },
  "tasks": [
    {
      "title": "Implement WebSocket client",
      "description": "...",
      "complexity": "medium",
      "target_component": "Rust",
      "estimated_hours": 8,
      "dependencies": []
    }
  ],
  "breakdown_path": "docs/research_breakdowns/websocket_strategy_PLAN.md",
  "tasks_path": "docs/research_breakdowns/websocket_strategy_PLAN.tasks.json"
}
```

#### Analyze File from Path

```bash
curl -X POST http://localhost:8080/api/research/file \
  -H "Content-Type: application/json" \
  -d '{
    "file_path": "research/inbox/market_making.md",
    "generate_tasks": false
  }'
```

## Configuration

### Environment Variables

```bash
# Enable/disable research pipeline
RESEARCH_ENABLED=true

# Output directory for breakdowns
RESEARCH_OUTPUT_DIR=docs/research_breakdowns

# LLM must be enabled
LLM_ENABLED=true
XAI_API_KEY=your_api_key_here

# Or use Google Gemini
LLM_PROVIDER=google
GOOGLE_API_KEY=your_api_key_here
```

### Custom Prompts

Edit `config/research.toml` to customize the AI prompts:

```toml
[research]
enabled = true
output_dir = "docs/research_breakdowns"
file_extensions = ["md", "txt", "tex"]

[research.prompts]
breakdown = """
Your custom system prompt for generating implementation plans...
"""

tasks = """
Your custom prompt for extracting tasks...
"""
```

The default prompts are context-aware for JANUS:
- **Rust** for high-performance execution ("Muscle")
- **Python** for AI and strategy ("Brain")
- **Kotlin** for frontend interfaces
- **Infrastructure** for Docker, databases, etc.

## Workflow Examples

### Example 1: New Trading Strategy

**Input**: Research notes on a reinforcement learning market-making strategy

```bash
# 1. Save research to inbox
cat > research/inbox/rl_market_making.md << EOF
# Reinforcement Learning for Market Making

Paper: "Deep Reinforcement Learning for Market Making"
Source: arXiv 2023

Key concepts:
- Use DQN for quote placement
- State: order book depth, recent trades, inventory
- Reward: PnL - inventory risk penalty
...
EOF

# 2. Analyze and generate tasks
./target/release/audit research research/inbox/rl_market_making.md --generate-tasks

# 3. Review generated plan
cat docs/research_breakdowns/rl_market_making_PLAN.md

# 4. Review tasks
cat docs/research_breakdowns/rl_market_making_PLAN.tasks.json
```

**Output Plan Sections**:
1. Executive Summary - Why this improves JANUS
2. Technical Requirements - Data, performance, dependencies
3. Architecture Integration - Rust components, Python models
4. Implementation Steps - Ordered action items
5. Testing Strategy - Unit, integration, performance tests
6. Risks and Mitigations - Technical challenges

**Generated Tasks** (example):
```json
[
  {
    "title": "Create order book state representation",
    "description": "Implement OrderBookState struct in src/execution/orderbook.rs...",
    "complexity": "medium",
    "target_component": "Rust",
    "estimated_hours": 6
  },
  {
    "title": "Implement DQN model in Python",
    "description": "Create src/janus/brain/models/dqn.py using PyTorch...",
    "complexity": "high",
    "target_component": "Python",
    "estimated_hours": 20
  }
]
```

### Example 2: Library Evaluation

**Input**: Notes on a WebSocket library

```bash
# Quick analysis without task generation
./target/release/audit research research/inbox/tokio_tungstenite.md

# Output: Implementation plan for integrating the library
```

### Example 3: Web UI Integration

Use the API from your web interface:

```javascript
async function analyzeResearch() {
    const content = document.getElementById('research-text').value;
    const title = document.getElementById('research-title').value;
    
    const response = await fetch('/api/research/analyze', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({
            content: content,
            title: title,
            generate_tasks: true
        })
    });
    
    const result = await response.json();
    displayBreakdown(result.breakdown);
    displayTasks(result.tasks);
}
```

## Output Format

### Breakdown Markdown Structure

```markdown
# [Topic] - Implementation Plan

## 1. Executive Summary
Brief overview and value proposition

## 2. Technical Requirements
- Data requirements
- Performance constraints
- Dependencies
- Security considerations

## 3. Architecture Integration
### Rust Components
Specific modules, structs, traits needed

### Python Components
Scripts, models, algorithms needed

### Infrastructure Changes
Database schemas, message queues, etc.

## 4. Implementation Steps
1. Detailed step 1
2. Detailed step 2
...

## 5. Testing Strategy
Test requirements and scenarios

## 6. Risks and Mitigations
Potential challenges and solutions

## 7. Rollout Plan
Phased deployment strategy
```

### Task JSON Structure

```json
[
  {
    "title": "Short task title",
    "description": "Detailed technical description with file paths",
    "complexity": "low|medium|high",
    "target_component": "Rust|Python|Kotlin|Infrastructure|Documentation",
    "estimated_hours": 8,
    "dependencies": ["Other task title"]
  }
]
```

## Integration with JANUS

The Research Pipeline is designed to align with JANUS architecture:

### Muscle (Rust)
- Execution layer tasks
- High-performance requirements
- Low-latency constraints

### Brain (Python)
- Strategy implementation
- Machine learning models
- Backtesting and analysis

### Infrastructure
- Database schemas
- Message queue topics
- Docker services
- Deployment configurations

### Documentation
- API documentation
- Architecture diagrams
- Setup guides

## Best Practices

### 1. Organize Research Files

```
research/
├── inbox/              # New research to be processed
├── papers/             # Academic papers
├── libraries/          # Library evaluations
└── strategies/         # Trading strategy research
```

### 2. Use Descriptive Filenames

✅ Good: `rl_market_making_arxiv_2023.md`
❌ Bad: `paper1.md`

### 3. Include Context in Research Files

Add metadata at the top:
```markdown
---
title: Deep RL for Market Making
source: arXiv:2301.12345
date: 2024-01-15
priority: high
---

# Deep Reinforcement Learning for Market Making
...
```

### 4. Review Generated Plans

Always review LLM output:
- Verify technical accuracy
- Check dependency order
- Validate time estimates
- Ensure JANUS alignment

### 5. Iterate on Tasks

Use generated tasks as a starting point:
- Refine descriptions
- Adjust estimates
- Add missing dependencies
- Break down complex tasks

## Troubleshooting

### LLM Not Enabled

```
Error: LLM is not enabled. Set LLM_ENABLED=true
```

**Solution**: Set environment variables:
```bash
export LLM_ENABLED=true
export XAI_API_KEY=your_key_here
```

### Research Pipeline Disabled

```
Error: Research pipeline is disabled
```

**Solution**: Enable in configuration:
```bash
export RESEARCH_ENABLED=true
```

### File Not Found

```
Error: File not found: research/inbox/paper.md
```

**Solution**: Verify file path is correct and file exists.

### JSON Parse Error

```
Error: Failed to parse LLM JSON response
```

**Solution**: This usually means the LLM returned markdown instead of JSON. Check:
- Prompt configuration
- LLM model compatibility
- API response format

### Rate Limiting

```
Error: Rate limit exceeded
```

**Solution**: 
- Wait and retry
- Use batch processing
- Consider upgrading API tier

## Advanced Usage

### Batch Processing

Process multiple files:

```bash
for file in research/inbox/*.md; do
    ./target/release/audit research "$file" --generate-tasks
done
```

### Custom Prompts

Override default prompts programmatically:

```rust
use audit::research;
use audit::Config;

let mut config = Config::load()?;
config.research.as_mut().unwrap().prompts.insert(
    "breakdown".to_string(),
    "Custom prompt for your use case...".to_string()
);
```

### Integration with CI/CD

Add to your pipeline:

```yaml
# .github/workflows/research.yml
name: Process Research

on:
  push:
    paths:
      - 'research/inbox/**'

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Analyze research
        run: |
          ./target/release/audit research research/inbox/*.md --generate-tasks
      - name: Commit results
        run: |
          git add docs/research_breakdowns/
          git commit -m "Add research breakdowns"
          git push
```

## API Reference

### POST `/api/research/analyze`

Analyze raw content.

**Request**:
```json
{
  "content": "string",
  "title": "string",
  "generate_tasks": boolean
}
```

**Response**:
```json
{
  "breakdown": ResearchBreakdown,
  "tasks": ResearchTask[] | null,
  "breakdown_path": "string",
  "tasks_path": "string" | null
}
```

### POST `/api/research/file`

Analyze file from path.

**Request**:
```json
{
  "file_path": "string",
  "generate_tasks": boolean
}
```

**Response**: Same as `/analyze`

## Future Enhancements

- [ ] Web UI with drag-and-drop file upload
- [ ] Automatic research file monitoring
- [ ] Integration with task management systems
- [ ] Multi-document synthesis (combine multiple papers)
- [ ] Citation tracking and bibliography generation
- [ ] Research knowledge graph
- [ ] Automated literature review
- [ ] Version control for breakdowns
- [ ] Comparison of alternative approaches

## Contributing

To extend the Research Pipeline:

1. **Add new prompts** in `config/research.toml`
2. **Customize parsing** in `research.rs::parse_breakdown_response()`
3. **Add new endpoints** in `server.rs`
4. **Enhance CLI** in `bin/cli.rs`

## License

Same as parent project (see root LICENSE file).