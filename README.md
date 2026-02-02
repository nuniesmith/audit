# Rustassistant

> A powerful developer workflow management system with LLM-powered code analysis

[![CI/CD](https://github.com/nuniesmith/rustassistant/workflows/CI/badge.svg)](https://github.com/nuniesmith/rustassistant/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

---

## 🚀 Quick Start

```bash
# Clone and setup
cd rustassistant
./run.sh

# Or build locally
cargo build --release

# Use the CLI
./target/release/rustassistant note add "My first note!" --tags milestone
./target/release/rustassistant stats

# Start the API server
./target/release/rustassistant-server
```

**That's it!** The setup script handles everything:
- Creates `.env` with secure secrets
- Asks for your XAI API key (optional)
- Sets up the database
- Builds and starts services

---

## ✨ Features

### Core Functionality
- 📝 **Note Management** - Capture thoughts, ideas, and TODOs with tags and projects
- 📂 **Repository Tracking** - Monitor your projects and codebases
- ✅ **Task Management** - Create, prioritize, and track tasks
- 🤖 **LLM Integration** - Grok AI-powered code analysis (Week 3 feature)
- 🔍 **Smart Search** - Find notes and tasks quickly
- 📊 **Statistics Dashboard** - Overview of your workflow

### Developer Experience
- 🎨 **Beautiful CLI** - Colored output with emoji icons
- 🌐 **REST API** - Full CRUD operations for all resources
- 🐳 **Docker Ready** - Production-ready containers
- 🔒 **Secure** - Encrypted secrets, secure defaults
- 🧪 **100% Test Coverage** - All database operations tested
- 📖 **Comprehensive Docs** - Everything you need to contribute

---

## 📋 What Can You Do?

### Capture Notes
```bash
rustassistant note add "Remember to refactor auth module" --tags todo,code
rustassistant note add "Great idea for dark mode" --project myapp --tags feature,ui
rustassistant note search "refactor"
rustassistant note list --status inbox
```

### Manage Tasks
```bash
rustassistant tasks list --status pending
rustassistant tasks start TASK-A1B2C3D4
rustassistant tasks done TASK-A1B2C3D4
rustassistant next  # Get next recommended task
```

### Track Repositories
```bash
rustassistant repo add ~/projects/myapp --name myapp
rustassistant repo list
```

### Use the API
```bash
# Start the server
rustassistant-server

# Create a note via API
curl -X POST http://localhost:3000/api/notes \
  -H "Content-Type: application/json" \
  -d '{"content":"Created via API!","tags":"api,test"}'

# Get statistics
curl http://localhost:3000/api/stats

# Health check
curl http://localhost:3000/health
```

---

## 🏗️ Architecture

```
┌─────────────┐
│   CLI Tool  │  (512 lines, colored output)
└──────┬──────┘
       │
       ├─────────────────┐
       │                 │
       ▼                 ▼
┌─────────────┐   ┌──────────────┐
│   Database  │◄──┤  REST API    │  (426 lines, Axum)
│   (SQLite)  │   │  (Axum)      │
└─────────────┘   └──────────────┘
       │
       ├── Notes (with tags & projects)
       ├── Repositories (with metadata)
       └── Tasks (with priorities & status)
```

**Tech Stack:**
- **Language:** Rust 1.75+
- **Web Framework:** Axum 0.7
- **Database:** SQLite (via sqlx 0.8)
- **CLI:** Clap 4.4
- **Async Runtime:** Tokio 1.35
- **HTTP Client:** Reqwest 0.11

---

## 📊 Database Schema

### Notes
```sql
notes (
  id TEXT PRIMARY KEY,      -- UUID
  content TEXT,
  tags TEXT,                -- Comma-separated
  project TEXT,             -- Optional project
  status TEXT,              -- inbox|processed|archived
  created_at INTEGER,
  updated_at INTEGER
)
```

### Repositories
```sql
repositories (
  id TEXT PRIMARY KEY,      -- UUID
  path TEXT UNIQUE,         -- Filesystem path
  name TEXT,
  status TEXT,              -- active|archived
  last_analyzed INTEGER,
  metadata TEXT,            -- JSON blob
  created_at INTEGER,
  updated_at INTEGER
)
```

### Tasks
```sql
tasks (
  id TEXT PRIMARY KEY,      -- TASK-XXXXXXXX
  title TEXT,
  description TEXT,
  priority INTEGER,         -- 1=critical, 2=high, 3=medium, 4=low
  status TEXT,              -- pending|in_progress|done
  source TEXT,              -- note|analysis|manual
  repo_id TEXT,             -- Foreign key
  file_path TEXT,           -- Source file
  line_number INTEGER,
  created_at INTEGER,
  updated_at INTEGER
)
```

---

## 🐳 Docker Deployment

### Using Docker Compose
```bash
# Start all services
./run.sh up

# View logs
./run.sh logs

# Stop services
./run.sh down

# Clean up
./run.sh clean
```

### Manual Docker
```bash
# Build image
docker build -f docker/Dockerfile -t rustassistant:latest .

# Run container
docker run -d \
  -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  -e DATABASE_URL=sqlite:/app/data/rustassistant.db \
  -e XAI_API_KEY=your-key-here \
  rustassistant:latest
```

---

## 🧪 Development

### Prerequisites
- Rust 1.75+ ([install](https://rustup.rs/))
- Docker & Docker Compose (optional)
- SQLite 3
- Git

### Build & Test
```bash
# Build
cargo build --release

# Run tests
cargo test

# Run specific test
cargo test db::tests::test_create_and_get_note

# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features
```

### Project Structure
```
rustassistant/
├── src/
│   ├── bin/
│   │   ├── cli.rs              # CLI tool (512 lines)
│   │   └── server.rs           # REST API (426 lines)
│   ├── db.rs                   # Database module (714 lines)
│   ├── grok_reasoning.rs       # LLM integration
│   ├── todo_scanner.rs         # TODO extraction
│   ├── tree_state.rs           # Directory caching
│   └── lib.rs
├── docs/
│   └── DEVELOPER_GUIDE.md      # Complete dev guide
├── docker/
│   └── Dockerfile
├── .github/workflows/
│   └── ci.yml                  # GitHub Actions
├── docker-compose.yml
├── run.sh                      # Setup & run script
└── README.md                   # This file
```

---

## 📚 Documentation

- **[Developer Guide](docs/DEVELOPER_GUIDE.md)** - Complete development documentation
- **[Integration Report](docs/integration/SUCCESS.md)** - Recent integration details
- **[Quick Start](docs/integration/QUICK_START.md)** - Get started in 5 minutes

---

## 🚀 CI/CD

### GitHub Actions
Automated pipeline includes:
- ✅ Linting (rustfmt, clippy)
- ✅ Testing (Linux, macOS, Windows)
- ✅ Security audit
- ✅ Docker image build
- ✅ Integration tests
- ✅ Release binaries

### Required Secrets
- `XAI_API_KEY` - Your XAI API key for integration tests

---

## 🗺️ Roadmap

### ✅ Week 1 - Core MVP (COMPLETE)
- [x] Database module with tasks
- [x] REST API server
- [x] CLI tool
- [x] Docker deployment
- [x] CI/CD pipeline

### 🔄 Week 2 - Repository Tracking (IN PROGRESS)
- [ ] Repository analysis
- [ ] TODO scanner integration
- [ ] File scoring
- [ ] Task generation from TODOs

### 📅 Week 3 - LLM Integration (PLANNED)
- [ ] Grok API integration
- [ ] Code analysis
- [ ] Cost tracking
- [ ] Smart recommendations

### 📅 Week 4 - Smart Workflow (PLANNED)
- [ ] Task prioritization
- [ ] Next task recommendations
- [ ] Batch operations
- [ ] Advanced search

---

## 🤝 Contributing

We welcome contributions! Please see our [Developer Guide](docs/DEVELOPER_GUIDE.md) for details.

### Quick Contribution Steps
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Commit (`git commit -m 'feat: add amazing feature'`)
6. Push (`git push origin feature/amazing-feature`)
7. Open a Pull Request

---

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Powered by [Axum](https://github.com/tokio-rs/axum)
- Database via [SQLx](https://github.com/launchbadge/sqlx)
- LLM integration with [XAI Grok](https://x.ai/)

---

## 📞 Support

- **Issues:** [GitHub Issues](https://github.com/nuniesmith/rustassistant/issues)
- **Discussions:** [GitHub Discussions](https://github.com/nuniesmith/rustassistant/discussions)
- **Documentation:** [docs/](docs/)

---

**Built with ❤️ and 🦀 by the Rustassistant team**

*Last updated: February 2, 2025*