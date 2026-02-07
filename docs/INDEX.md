# RustAssistant Documentation Index

Welcome to the RustAssistant documentation! This guide will help you find what you need quickly.

---

## 🚀 Quick Start

New to RustAssistant? Start here:

1. **[Getting Started](user/GETTING_STARTED.md)** - First-time setup and basics
2. **[Quick Start Guide](user/QUICKSTART.md)** - 5-minute setup
3. **[Web UI Quick Start](user/WEB_UI_QUICKSTART.md)** - Get the UI running in minutes

---

## 📚 Documentation by Category

### For Users

Essential guides for using RustAssistant:

- **[Web UI Status](user/WEB_UI_STATUS.md)** - Complete feature list and capabilities
- **[Web UI Quick Reference](user/WEB_UI_QUICK_REFERENCE.md)** - Command cheat sheet
- **[Simplified Setup Guide](user/SIMPLIFIED_SETUP.md)** - Migrate to 2-container deployment
- **[Auto-Scanner Setup](user/AUTO_SCANNER_SETUP.md)** - Configure automatic repository scanning
- **[CLI Cheat Sheet](user/CLI_CHEATSHEET.md)** - Command-line reference

### For Developers

Contributing to RustAssistant:

- **[Developer Guide](developer/DEVELOPER_GUIDE.md)** - How to contribute
- **[API Reference](developer/API_REFERENCE.md)** - REST API endpoints
- **[Code Review Guidelines](developer/CODE_REVIEW.md)** - Standards and best practices
- **[Testing Guide](developer/TESTING.md)** - Running and writing tests
- **[CI/CD Review](developer/CICD_REVIEW.md)** - Continuous integration setup

### Reference Documentation

Advanced topics and technical details:

- **[Docker Quick Start](DOCKER_QUICK_START.md)** - Docker deployment guide
- **[Raspberry Pi Guide](RASPBERRY_PI_GUIDE.md)** - ARM64 deployment
- **[Grok 4.1 Migration](GROK_4.1_MIGRATION.md)** - LLM API migration guide
- **[Advanced Features](ADVANCED_FEATURES_GUIDE.md)** - Power user features
- **[Batch Operations](BATCH_OPERATIONS.md)** - Bulk processing guide
- **[Repository Cache Design](REPO_CACHE_DESIGN.md)** - Caching architecture
- **[Research Guide](RESEARCH_GUIDE.md)** - Research workflow
- **[Research Topics](RESEARCH_TOPICS.md)** - Areas of exploration

---

## 🎯 Common Tasks

### Setup & Installation
- [Install with Docker](DOCKER_QUICK_START.md)
- [Set up auto-scanning](user/AUTO_SCANNER_SETUP.md)
- [Configure the Web UI](user/WEB_UI_QUICKSTART.md)

### Daily Workflow
- [Use the Web UI](user/WEB_UI_STATUS.md)
- [Manage repositories](user/WEB_UI_STATUS.md#repository-management)
- [Work with the queue](user/WEB_UI_STATUS.md#queue-management)
- [CLI commands](user/CLI_CHEATSHEET.md)

### Integration
- [Copy issues to IDE](user/WEB_UI_QUICK_REFERENCE.md#copy-issue-to-ide)
- [API endpoints](developer/API_REFERENCE.md)
- [Batch operations](BATCH_OPERATIONS.md)

### Troubleshooting
- [Simplified Setup FAQ](user/SIMPLIFIED_SETUP.md#troubleshooting)
- [Web UI troubleshooting](user/WEB_UI_QUICKSTART.md#troubleshooting)
- [Docker issues](DOCKER_QUICK_START.md)

---

## 📦 Deployment Guides

### Quick Deployments
```bash
# Standard Docker setup (2 containers)
docker compose up -d
open http://localhost:3000
```

See:
- [Simplified Setup Guide](user/SIMPLIFIED_SETUP.md) - Two-container deployment
- [Docker Quick Start](DOCKER_QUICK_START.md) - Docker details

### Production Deployment
```bash
# Production with pre-built images
docker compose -f docker-compose.prod.yml up -d
```

See archived migration guides in `archive/migrations/` for historical context.

---

## 🏗️ Architecture Overview

RustAssistant consists of:

1. **Unified Server** (Port 3000)
   - Web UI (dashboard, repos, queue)
   - REST API endpoints
   - Auto-scanner background task
   
2. **Redis Cache** (Port 6379)
   - LLM response caching
   - Performance optimization

See [Repository Cache Design](REPO_CACHE_DESIGN.md) for technical details.

---

## 📂 Directory Structure

```
docs/
├── INDEX.md (this file)         # Documentation hub
│
├── user/                         # End-user guides
│   ├── GETTING_STARTED.md       # First-time setup
│   ├── QUICKSTART.md            # 5-minute guide
│   ├── WEB_UI_*.md              # Web UI documentation
│   ├── AUTO_SCANNER_SETUP.md    # Scanner config
│   └── CLI_CHEATSHEET.md        # Command reference
│
├── developer/                    # Contributor docs
│   ├── DEVELOPER_GUIDE.md       # Contributing
│   ├── API_REFERENCE.md         # API docs
│   ├── CODE_REVIEW.md           # Standards
│   └── TESTING.md               # Test guide
│
├── archive/                      # Historical docs
│   ├── sessions/                # Session summaries
│   ├── migrations/              # Old migration guides
│   └── deprecated/              # Obsolete docs
│
└── *.md                         # Reference documentation
```

---

## 🔍 Finding What You Need

### I want to...

**Get started quickly**
→ [Quick Start Guide](user/QUICKSTART.md)

**Use the Web UI**
→ [Web UI Quick Start](user/WEB_UI_QUICKSTART.md)

**Set up auto-scanning**
→ [Auto-Scanner Setup](user/AUTO_SCANNER_SETUP.md)

**Learn CLI commands**
→ [CLI Cheat Sheet](user/CLI_CHEATSHEET.md)

**Contribute code**
→ [Developer Guide](developer/DEVELOPER_GUIDE.md)

**Deploy to production**
→ [Docker Quick Start](DOCKER_QUICK_START.md)

**Deploy to Raspberry Pi**
→ [Raspberry Pi Guide](RASPBERRY_PI_GUIDE.md)

**Use the API**
→ [API Reference](developer/API_REFERENCE.md)

**Troubleshoot issues**
→ [Simplified Setup FAQ](user/SIMPLIFIED_SETUP.md#troubleshooting)

**Learn advanced features**
→ [Advanced Features Guide](ADVANCED_FEATURES_GUIDE.md)

---

## 📖 Documentation Status

| Category | Files | Status |
|----------|-------|--------|
| User Guides | 13 | ✅ Current |
| Developer Docs | 5 | ✅ Current |
| Reference | 10 | ✅ Current |
| Archived | 59 | 📦 Historical |

Last updated: 2024-01-15

---

## 🆘 Getting Help

1. **Check the docs** - Start with the index above
2. **Review examples** - See guides for common tasks
3. **Check archived docs** - `archive/` has historical context
4. **Open an issue** - GitHub issues for bugs/features

---

## 📝 Contributing to Docs

Found an error or want to improve the documentation?

1. See [Developer Guide](developer/DEVELOPER_GUIDE.md)
2. Edit the relevant .md file
3. Submit a pull request

All documentation is written in Markdown and stored in the `docs/` directory.

---

**Current Version**: 0.2.0  
**Documentation Structure**: Reorganized 2024-01-15  
**Archived Docs**: Available in `archive/` for historical reference