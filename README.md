# DevFlow - Developer Workflow Management System

> 🚀 **A Rust-based workflow manager for solo developers to track repos, capture ideas, and leverage LLM-powered insights**

DevFlow helps you manage the entire development lifecycle from idea capture to production deployment. Built with Rust, powered by Grok 4.1 LLM, designed for developers who manage multiple GitHub repositories.

## 🎯 Core Features

### 📝 Note & Thought Capture
- Quick note input with tag-based categorization
- Personal notes for random thoughts
- Project-specific notes linked to repos
- Forward notes to specific projects for future work

### 🗂️ Repository Management
- Track all your GitHub repositories
- Cache directory trees and file contents
- Monitor changes across repos
- Standardize tooling and patterns

### 🤖 LLM-Powered Analysis
- Grok 4.1 API integration (2M context window)
- Score files for quality, security, and complexity
- Find issues and suggest improvements
- Identify common patterns and shared logic
- Generate actionable tasks from analysis

### 🎯 Solo Developer Workflow
- **Research**: Validate and expand research areas
- **Planning**: Break down complex features
- **Prototype**: Track experimental code
- **Production**: Monitor production-ready systems
- **Next Actions**: Always know what to work on next

### 💾 RAG System with Git-Friendly Storage
- Vector embeddings for semantic code search
- Store vector data as small files in git
- Incremental updates (minimal daily churn)
- Build contextual understanding of your codebase

### 🏗️ Tech Stack Support
Built to work with your stack:
- **Languages**: Rust, Kotlin Multiplatform, JavaScript, TypeScript, Python
- **Infrastructure**: Docker Compose, Nginx, Prometheus, Alertmanager, Grafana, Loki
- **Databases**: PostgreSQL, Redis, QuestDB
- **CI/CD**: GitHub Actions (test → build-push → deploy)

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (`rustup update`)
- Git
- XAI API Key (Grok 4.1)
- Docker & Docker Compose (for deployment)

### Installation

```bash
# Clone the repo
git clone https://github.com/your-username/devflow.git
cd devflow

# Build the project
cargo build --release

# Set up environment
cp .env.example .env
# Edit .env and add your XAI_API_KEY
```

### Environment Setup

Create a `.env` file:

```bash
# XAI API Configuration (Grok 4.1)
XAI_API_KEY=xai-your-api-key-here
XAI_BASE_URL=https://api.x.ai/v1

# Server Configuration
HOST=127.0.0.1
PORT=3000

# GitHub Integration
GITHUB_TOKEN=ghp_your_token_here  # Optional, for private repos

# Database (future)
DATABASE_URL=postgresql://user:pass@localhost:5432/devflow

# Logging
RUST_LOG=info,devflow=debug
```

## 📖 Usage

### 1. Start the Server

```bash
# Run the web server
cargo run --release --bin devflow-server

# Server starts at http://localhost:3000
```

### 2. CLI Tool

```bash
# Quick note capture
devflow note add "Idea for new feature X" --tags idea,research

# Analyze a repository
devflow repo analyze /path/to/repo --deep

# Score all files in a repo
devflow repo score /path/to/repo --output json

# Generate tasks from analysis
devflow tasks generate /path/to/repo

# Find next actions
devflow next --category prototype

# List all tracked repos
devflow repo list --status active
```

### 3. Web Interface

```bash
# Open the web UI
http://localhost:3000

# Features:
# - Dashboard: Overview of all repos and tasks
# - Notes: Create and organize notes/thoughts
# - Repos: Browse cached directory trees
# - Analysis: View LLM insights and scores
# - Tasks: Manage generated tasks
```

## 🏗️ Project Structure

```
devflow/
├── src/
│   ├── bin/
│   │   ├── server.rs          # Web server entry point
│   │   └── cli.rs             # CLI tool
│   ├── api/                   # REST API handlers
│   ├── notes/                 # Note-taking system
│   ├── repos/                 # Repository management
│   ├── llm/                   # LLM integration (Grok)
│   ├── analysis/              # Code analysis
│   ├── rag/                   # RAG system & vector storage
│   ├── tasks/                 # Task generation
│   └── lib.rs
├── static/                    # Web UI assets
├── config/                    # Configuration profiles
├── data/                      # Local data storage
│   ├── repos/                 # Cached repo data
│   ├── vectors/               # Vector embeddings (git-tracked)
│   └── notes/                 # Notes database
├── docker/                    # Docker deployment files
├── docs/                      # Documentation
└── Cargo.toml
```

## 🐳 Docker Deployment

### Home Server Setup

```bash
# Build and run locally
docker-compose up -d

# Access at http://localhost:3000
```

### Production Deployment (Linode)

```bash
# Deploy via Docker Compose
docker-compose -f docker-compose.prod.yml up -d

# Set up Nginx reverse proxy
# Configure SSL with Let's Encrypt
# Set up monitoring (Prometheus + Grafana)
```

## 💡 Workflow Examples

### Capture an Idea

```bash
# Quick thought
devflow note add "Maybe use WASM for the data processing pipeline" --tags idea,research

# Project-specific
devflow note add "Refactor auth system" --project myapp --tags refactor,prod
```

### Analyze a Repository

```bash
# Full analysis with Grok
devflow repo analyze ~/github/myproject --cache --score

# Output:
# 📊 Repository Analysis: myproject
# ├── Files: 234
# ├── Total Lines: 45,678
# ├── Languages: Rust (65%), JavaScript (25%), Other (10%)
# ├── Quality Score: 7.8/10
# ├── Issues Found: 12 (3 high, 9 medium)
# └── Cached: ✓
```

### Find Next Actions

```bash
# Show me what to work on
devflow next

# Output:
# 🎯 Next Actions (5 items):
#
# 1. [HIGH] Fix security issue in auth module (myapp)
#    - File: src/auth/mod.rs:45
#    - Issue: Potential SQL injection
#
# 2. [MEDIUM] Complete user API docs (api-server)
#    - Missing: 12 endpoints
#
# 3. [LOW] Refactor common error handling (shared-lib)
#    - Pattern found in 8 repos
```

### Generate Tasks from Research

```bash
# You've done research with Claude Opus 4.5
# Now break it down into tasks with cheaper Grok

devflow research import research-notes.md --generate-tasks

# Output:
# ✅ Imported research document
# 🔍 Analyzing with Grok 4.1...
# 📋 Generated 14 tasks:
#    - 5 research validation tasks
#    - 6 implementation tasks
#    - 3 testing tasks
```

## 🔑 Core Concepts

### Tags & Categories

**Built-in Categories:**
- `personal` - Random thoughts and ideas
- `research` - Research topics and validation
- `prototype` - Experimental features
- `production` - Production systems
- `infrastructure` - DevOps and tooling
- `documentation` - Docs and guides

**Custom Tags:**
- Any tag you want: `urgent`, `blocked`, `waiting`, `review`, etc.

### Repository Tracking

DevFlow maintains a cached view of each repository:
- Directory tree structure
- File metadata (size, modified date, language)
- Git status (branch, commits, changes)
- Analysis results (scores, issues, patterns)
- Historical trends

### LLM Cost Management

**Grok 4.1 Fast Reasoning** (cheap & efficient):
- Input: ~$0.20 per 1M tokens
- Output: ~$2.00 per 1M tokens
- Perfect for: Routine analysis, task generation, scoring

**Claude Opus 4.5** (expensive, for deep work):
- Use for: Research validation, architecture review
- DevFlow helps you: Split work for cheaper models
- Manual trigger: Use when quality matters most

### Vector Storage Strategy

DevFlow stores embeddings as git-trackable files:
- One file per code file: `data/vectors/repo-name/src/main.rs.vec`
- JSON format for human readability
- Small updates (only changed files)
- Incremental daily commits

## 🛠️ Configuration

### Repository Profiles

Create profiles for different types of projects:

```toml
# config/rust-service.toml
[profile]
name = "rust-service"
description = "Standard Rust microservice"

[structure]
required_dirs = ["src", "tests", "docker"]
required_files = ["Cargo.toml", "Dockerfile", "README.md"]

[quality]
min_doc_coverage = 0.8
min_test_coverage = 0.7
max_complexity = 15

[checks]
enforce_clippy = true
enforce_fmt = true
require_ci = true
```

### Analysis Presets

```toml
# config/analysis-presets.toml
[presets.quick]
static_only = true
skip_tests = true
cost = "free"

[presets.standard]
llm_enabled = true
model = "grok-4-1-fast-reasoning"
max_cost = 0.50

[presets.deep]
llm_enabled = true
model = "claude-opus-4-5"
max_cost = 5.00
```

## 📊 Monitoring & Metrics

Track your development workflow:
- Notes captured per day/week
- Repos analyzed
- Tasks completed
- LLM API costs
- Code quality trends

## 🗺️ Roadmap

### Phase 1: Core Foundation (Current)
- ✅ Basic note-taking
- ✅ Repository caching
- ✅ Grok 4.1 integration
- ✅ File scoring
- 🚧 Web UI
- 🚧 Task generation

### Phase 2: Intelligence Layer
- RAG system with semantic search
- Pattern detection across repos
- Automated task prioritization
- Research validation pipeline

### Phase 3: Automation
- Auto-generate GitHub issues
- CI/CD integration for task tracking
- Slack/Discord notifications
- Automated research → task → PR flow

### Phase 4: Collaboration
- Team features (optional)
- Shared knowledge base
- Code review assistance

## 🤝 Philosophy

DevFlow is built for solo developers who:
- Manage multiple GitHub repos
- Want to standardize tooling and patterns
- Need help prioritizing work
- Want LLM assistance without breaking the bank
- Prefer local/self-hosted tools
- Value incremental, trackable data

## 📝 License

MIT OR Apache-2.0

## 🆘 Support

- GitHub Issues: https://github.com/your-username/devflow/issues
- Docs: https://devflow.dev/docs
- Discord: https://discord.gg/devflow

---

**Built with ❤️ in Rust for developers who ship**