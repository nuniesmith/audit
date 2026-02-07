# Rustassistant Developer Guide

**Last Updated:** February 2, 2025  
**Version:** 1.0.0

This is your single source of truth for developing on Rustassistant.

---

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Architecture Overview](#architecture-overview)
- [Database Schema](#database-schema)
- [API Reference](#api-reference)
- [Testing](#testing)
- [Docker Development](#docker-development)
- [CI/CD Pipeline](#cicd-pipeline)
- [Common Tasks](#common-tasks)
- [Troubleshooting](#troubleshooting)
- [Contributing Guidelines](#contributing-guidelines)

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ ([install](https://rustup.rs/))
- Docker & Docker Compose (optional, for containerized development)
- SQLite 3
- Git

### First Time Setup

```bash
# Clone the repository
cd /home/jordan/github/rustassistant

# Run the setup script (interactive mode)
./run.sh

# This will:
# - Create .env file with secure secrets
# - Ask for your XAI API key (optional)
# - Create data directory
# - Build and start services

# Alternatively, build locally without Docker
cargo build --release

# Run the CLI
./target/release/rustassistant --help

# Run the server
./target/release/rustassistant-server
```

### Daily Development

```bash
# Start development environment
cargo run --bin rustassistant-server

# In another terminal, use the CLI
cargo run --bin rustassistant -- note add "Working on feature X" --tags dev

# Run tests
cargo test

# Run specific test
cargo test db::tests::test_create_and_get_note

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets --all-features
```

---

## 📁 Project Structure

```
rustassistant/
├── .github/
│   └── workflows/
│       └── ci.yml              # GitHub Actions CI/CD
├── config/                     # Configuration files
├── data/                       # SQLite database (gitignored)
├── docker/
│   └── Dockerfile              # Production Docker image
├── docs/
│   ├── DEVELOPER_GUIDE.md      # This file
│   ├── integration/            # Integration documentation
│   └── archive/                # Old documentation
├── src/
│   ├── bin/
│   │   ├── cli.rs              # CLI tool (512 lines)
│   │   └── server.rs           # REST API server (426 lines)
│   ├── db.rs                   # Database module (714 lines) ⭐ NEW
│   ├── cache.rs                # Response caching
│   ├── git.rs                  # Git operations
│   ├── grok_reasoning.rs       # Grok API client
│   ├── scanner.rs              # File scanning
│   ├── scoring.rs              # Code quality scoring
│   ├── tasks.rs                # Task generation
│   ├── todo_scanner.rs         # TODO/FIXME scanner
│   ├── tree_state.rs           # Directory tree caching
│   └── lib.rs                  # Library exports
├── static/                     # Static assets
├── templates/                  # Askama templates
├── tests/                      # Integration tests
├── .env                        # Environment config (gitignored)
├── .env.example                # Example environment
├── Cargo.toml                  # Rust dependencies
├── docker-compose.yml          # Docker orchestration
├── run.sh                      # Setup and run script
└── README.md                   # Project overview
```

### Key Files to Know

| File | Purpose | Lines | Status |
|------|---------|-------|--------|
| `src/db.rs` | Database CRUD operations | 714 | ✅ Production |
| `src/bin/server.rs` | REST API handlers | 426 | ✅ Production |
| `src/bin/cli.rs` | CLI commands | 512 | ✅ Production |
| `src/grok_reasoning.rs` | LLM integration | ~500 | 🟡 Ready for Week 3 |
| `src/todo_scanner.rs` | TODO extraction | ~200 | 🟡 Ready for Week 2 |
| `src/tree_state.rs` | Directory caching | ~300 | 🟡 Ready for Week 2 |

---

## 🔄 Development Workflow

### Feature Development

1. **Create a branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Write tests first (TDD)**
   ```bash
   # Add test in src/db.rs or tests/
   cargo test --lib your_test_name
   ```

3. **Implement the feature**
   ```rust
   // Example: Adding a new DB function
   pub async fn my_new_function(pool: &SqlitePool) -> DbResult<Vec<Item>> {
       // Implementation
   }
   ```

4. **Run all checks**
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features
   cargo test
   ```

5. **Commit and push**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   git push origin feature/your-feature-name
   ```

6. **Open a Pull Request**
   - CI will run automatically
   - Ensure all checks pass
   - Request review if needed

### Code Style

- **Formatting:** Follow `rustfmt` (run `cargo fmt`)
- **Naming:** Snake_case for functions/variables, PascalCase for types
- **Documentation:** Use `///` for public APIs
- **Error Handling:** Use `Result<T, DbError>` or `anyhow::Result<T>`
- **Async:** Use `async/await` with `tokio`

---

## 🏗️ Architecture Overview

### System Components

```
┌─────────────┐
│   CLI Tool  │  (bin/cli.rs)
└──────┬──────┘
       │
       ├─────────────────┐
       │                 │
       ▼                 ▼
┌─────────────┐   ┌──────────────┐
│   Database  │◄──┤  REST API    │  (bin/server.rs)
│   (SQLite)  │   │  (Axum)      │
└─────────────┘   └──────┬───────┘
       ▲                 │
       │                 │
       └─────────────────┘
              │
              ▼
       ┌─────────────┐
       │  db.rs      │  (Database module)
       │  - Notes    │
       │  - Repos    │
       │  - Tasks    │
       └─────────────┘
```

### Request Flow

#### CLI Request
```
User → rustassistant note add "text"
  ↓
CLI parses args (clap)
  ↓
Calls db::create_note(pool, content, tags, project)
  ↓
SQLite INSERT
  ↓
Returns Note { id, content, ... }
  ↓
CLI prints colored output
```

#### API Request
```
Client → POST /api/notes {content: "text"}
  ↓
Axum router → create_note_handler
  ↓
JSON parsing & validation
  ↓
db::create_note(pool, content, tags, project)
  ↓
SQLite INSERT
  ↓
Returns JSON {success: true, data: {...}}
```

### Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Language** | Rust 1.75+ | Systems programming, safety, performance |
| **Web Framework** | Axum 0.7 | REST API server |
| **Database** | SQLite (via sqlx 0.8) | Embedded database |
| **CLI** | Clap 4.4 | Argument parsing |
| **Async Runtime** | Tokio 1.35 | Async/await execution |
| **Serialization** | Serde 1.0 | JSON/TOML handling |
| **HTTP Client** | Reqwest 0.11 | LLM API calls |
| **Error Handling** | Anyhow 1.0, Thiserror 2.0 | Error management |
| **Git** | git2 0.18 | Repository operations |
| **Logging** | Tracing 0.1 | Structured logging |

---

## 🗄️ Database Schema

### Notes Table
```sql
CREATE TABLE notes (
    id TEXT PRIMARY KEY,        -- UUID v4
    content TEXT NOT NULL,      -- Note text
    tags TEXT,                  -- Comma-separated tags
    project TEXT,               -- Optional project name
    status TEXT DEFAULT 'inbox', -- inbox|processed|archived
    created_at INTEGER NOT NULL, -- Unix timestamp
    updated_at INTEGER NOT NULL  -- Unix timestamp
);

-- Indexes
CREATE INDEX idx_notes_status ON notes(status);
CREATE INDEX idx_notes_project ON notes(project);
CREATE INDEX idx_notes_created ON notes(created_at DESC);
```

### Repositories Table
```sql
CREATE TABLE repositories (
    id TEXT PRIMARY KEY,         -- UUID v4
    path TEXT UNIQUE NOT NULL,   -- Filesystem path
    name TEXT NOT NULL,          -- Display name
    status TEXT DEFAULT 'active', -- active|archived
    last_analyzed INTEGER,       -- Last analysis timestamp
    metadata TEXT,               -- JSON blob for future use
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### Tasks Table ⭐ NEW
```sql
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,         -- TASK-XXXXXXXX format
    title TEXT NOT NULL,
    description TEXT,
    priority INTEGER DEFAULT 3,  -- 1=critical, 2=high, 3=medium, 4=low
    status TEXT DEFAULT 'pending', -- pending|in_progress|done
    source TEXT DEFAULT 'manual', -- note|analysis|manual
    source_id TEXT,              -- Reference to originating item
    repo_id TEXT,                -- Foreign key to repositories
    file_path TEXT,              -- Source file if from code
    line_number INTEGER,         -- Line number if from code
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (repo_id) REFERENCES repositories(id)
);

-- Indexes
CREATE INDEX idx_tasks_priority ON tasks(priority, status);
CREATE INDEX idx_tasks_repo ON tasks(repo_id);
```

### Database API (src/db.rs)

**Function-based API** (new architecture):

```rust
// Initialization
pub async fn init_db(url: &str) -> DbResult<SqlitePool>

// Notes
pub async fn create_note(pool, content, tags, project) -> DbResult<Note>
pub async fn get_note(pool, id) -> DbResult<Note>
pub async fn list_notes(pool, limit, status, project, tag) -> DbResult<Vec<Note>>
pub async fn search_notes(pool, query, limit) -> DbResult<Vec<Note>>
pub async fn update_note_status(pool, id, status) -> DbResult<()>
pub async fn delete_note(pool, id) -> DbResult<()>

// Repositories
pub async fn add_repository(pool, path, name) -> DbResult<Repository>
pub async fn get_repository(pool, id) -> DbResult<Repository>
pub async fn list_repositories(pool) -> DbResult<Vec<Repository>>
pub async fn remove_repository(pool, id) -> DbResult<()>

// Tasks ⭐ NEW
pub async fn create_task(pool, title, desc, priority, ...) -> DbResult<Task>
pub async fn list_tasks(pool, limit, status, priority, repo_id) -> DbResult<Vec<Task>>
pub async fn update_task_status(pool, id, status) -> DbResult<()>
pub async fn get_next_task(pool) -> DbResult<Option<Task>>

// Stats
pub async fn get_stats(pool) -> DbResult<DbStats>
```

**Legacy API** (backward compatibility):

```rust
// Database struct wrapper (for old code)
pub struct Database { pool: SqlitePool }

impl Database {
    pub async fn new(url: &str) -> DbResult<Self>
    pub async fn create_note(&self, content, status) -> DbResult<String>
    // ... other methods
}
```

---

## 🌐 API Reference

### REST Endpoints

**Base URL:** `http://localhost:3000`

#### Health & Stats

```http
GET /health
```
Returns: `{"status":"ok","service":"rustassistant","version":"0.1.0"}`

```http
GET /api/stats
```
Returns:
```json
{
  "success": true,
  "data": {
    "total_notes": 10,
    "inbox_notes": 5,
    "total_repos": 2,
    "total_tasks": 8,
    "pending_tasks": 3
  }
}
```

#### Notes

```http
POST /api/notes
Content-Type: application/json

{
  "content": "My note text",
  "tags": "tag1,tag2",      // Optional
  "project": "myproject"    // Optional
}
```

```http
GET /api/notes?limit=10&status=inbox&project=myproject&tag=important
```

```http
GET /api/notes/search?q=keyword&limit=20
```

```http
GET /api/notes/:id
```

```http
PUT /api/notes/:id
Content-Type: application/json

{
  "status": "processed"
}
```

```http
DELETE /api/notes/:id
```

#### Repositories

```http
POST /api/repos
Content-Type: application/json

{
  "path": "/home/user/project",
  "name": "My Project"      // Optional
}
```

```http
GET /api/repos
```

```http
GET /api/repos/:id
```

```http
DELETE /api/repos/:id
```

#### Tasks

```http
GET /api/tasks?limit=20&status=pending&priority=2
```

```http
GET /api/tasks/next
```
Returns the highest priority pending task.

```http
PUT /api/tasks/:id
Content-Type: application/json

{
  "status": "in_progress"  // or "done"
}
```

### CLI Commands

```bash
# Notes
rustassistant note add "content" [--tags tag1,tag2] [--project name]
rustassistant note list [--limit N] [--status inbox|processed|archived]
rustassistant note search "query"

# Repositories
rustassistant repo add <path> [--name name]
rustassistant repo list
rustassistant repo remove <id>

# Tasks
rustassistant tasks list [--limit N] [--status pending|in_progress|done]
rustassistant tasks start <id>
rustassistant tasks done <id>

# Utilities
rustassistant next        # Get next recommended task
rustassistant stats       # Show statistics
rustassistant test-api    # Test XAI API connection
```

---

## 🧪 Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test suite
cargo test --lib db::tests

# Specific test
cargo test test_create_and_get_note

# With output
cargo test -- --nocapture

# Documentation tests
cargo test --doc
```

### Database Tests

Located in `src/db.rs`:

```rust
#[cfg(test)]
mod tests {
    async fn setup_test_db() -> SqlitePool {
        init_db("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_create_and_get_note() {
        let pool = setup_test_db().await;
        let note = create_note(&pool, "Test", Some("tags"), None).await.unwrap();
        assert_eq!(note.content, "Test");
    }
}
```

Current test coverage:
- ✅ `test_create_and_get_note`
- ✅ `test_list_notes`
- ✅ `test_search_notes`
- ✅ `test_repository_crud`
- ✅ `test_task_creation_and_next`
- ✅ `test_stats`

### Writing New Tests

```rust
#[tokio::test]
async fn test_my_feature() {
    // Setup
    let pool = init_db("sqlite::memory:").await.unwrap();
    
    // Act
    let result = my_function(&pool, args).await;
    
    // Assert
    assert!(result.is_ok());
    let data = result.unwrap();
    assert_eq!(data.field, expected_value);
}
```

---

## 🐳 Docker Development

### Building Images

```bash
# Build the API server image
docker build -f docker/Dockerfile -t rustassistant:latest .

# Or use docker-compose
docker-compose build
```

### Running with Docker Compose

```bash
# Start all services
./run.sh up

# Or manually
docker-compose up -d

# View logs
docker-compose logs -f api

# Stop services
docker-compose down

# Clean up everything
./run.sh clean
```

### Docker Compose Services

```yaml
services:
  api:          # Main API server (port 3000)
  redis:        # Cache (optional, port 6379)
```

### Environment Variables

Create `.env` file:

```bash
DATABASE_URL=sqlite:/app/data/rustassistant.db
XAI_API_KEY=xai-your-key-here
HOST=0.0.0.0
PORT=3000
RUST_LOG=info,rustassistant=debug
```

---

## 🚀 CI/CD Pipeline

### GitHub Actions Workflow

Located at `.github/workflows/ci.yml`

**Triggers:**
- Push to `main` or `develop`
- Pull requests to `main` or `develop`
- Manual workflow dispatch

**Jobs:**

1. **lint** - Format and Clippy checks
2. **test** - Run tests on Linux, macOS, Windows
3. **security** - Cargo audit for vulnerabilities
4. **docker** - Build and push Docker images
5. **integration** - Integration tests with Docker Compose
6. **release** - Build release binaries for all platforms
7. **deploy** - Deploy to production (optional)

### Required Secrets

Add to GitHub repository settings → Secrets:

```
XAI_API_KEY         # Your XAI API key (for integration tests)
```

### CI/CD Usage

```bash
# Non-interactive mode for CI
XAI_API_KEY=${{ secrets.XAI_API_KEY }} ./run.sh --non-interactive up

# The script will:
# - Create .env with provided XAI_API_KEY
# - Generate random secrets for DB/Redis
# - Start services without prompts
```

---

## 📝 Common Tasks

### Adding a New Database Table

1. **Update schema in `src/db.rs`:**
   ```rust
   async fn create_tables(pool: &SqlitePool) -> DbResult<()> {
       sqlx::query(r#"
           CREATE TABLE IF NOT EXISTS my_table (
               id TEXT PRIMARY KEY,
               name TEXT NOT NULL,
               created_at INTEGER NOT NULL
           )
       "#).execute(pool).await?;
       Ok(())
   }
   ```

2. **Add struct:**
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
   pub struct MyItem {
       pub id: String,
       pub name: String,
       pub created_at: i64,
   }
   ```

3. **Add CRUD functions:**
   ```rust
   pub async fn create_item(pool: &SqlitePool, name: &str) -> DbResult<MyItem> {
       let id = uuid::Uuid::new_v4().to_string();
       let now = chrono::Utc::now().timestamp();
       
       sqlx::query("INSERT INTO my_table (id, name, created_at) VALUES (?, ?, ?)")
           .bind(&id).bind(name).bind(now)
           .execute(pool).await?;
       
       Ok(MyItem { id, name: name.to_string(), created_at: now })
   }
   ```

4. **Add tests:**
   ```rust
   #[tokio::test]
   async fn test_create_item() {
       let pool = setup_test_db().await;
       let item = create_item(&pool, "Test").await.unwrap();
       assert_eq!(item.name, "Test");
   }
   ```

### Adding a New API Endpoint

1. **Add handler in `src/bin/server.rs`:**
   ```rust
   async fn my_handler(
       State(state): State<AppState>,
       Json(req): Json<MyRequest>,
   ) -> impl IntoResponse {
       match my_db_function(&state.db, &req.param).await {
           Ok(data) => ApiResponse::ok(data).into_response(),
           Err(e) => ApiResponse::error(e.to_string()).into_response(),
       }
   }
   ```

2. **Add route:**
   ```rust
   Router::new()
       .route("/api/myendpoint", post(my_handler))
       // ... other routes
   ```

3. **Test with curl:**
   ```bash
   curl -X POST http://localhost:3000/api/myendpoint \
     -H "Content-Type: application/json" \
     -d '{"param":"value"}'
   ```

### Adding a New CLI Command

1. **Add to enum in `src/bin/cli.rs`:**
   ```rust
   #[derive(Subcommand)]
   enum Commands {
       // ... existing commands
       MyCommand {
           #[arg(short, long)]
           param: String,
       },
   }
   ```

2. **Add handler:**
   ```rust
   match cli.command {
       Commands::MyCommand { param } => {
           let result = my_function(&pool, &param).await?;
           println!("✓ Success: {:?}", result);
       }
       // ... other commands
   }
   ```

3. **Test:**
   ```bash
   cargo run --bin rustassistant -- my-command --param value
   ```

---

## 🔧 Troubleshooting

### Database Issues

**Problem:** `error returned from database: no such column`

**Solution:** Old database schema. Delete and recreate:
```bash
rm data/rustassistant.db*
./target/release/rustassistant stats  # Recreates with new schema
```

---

**Problem:** `unable to open database file`

**Solution:** Data directory doesn't exist:
```bash
mkdir -p data
```

---

### Build Issues

**Problem:** `linker 'cc' not found`

**Solution:** Install build tools:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
xcode-select --install
```

---

**Problem:** Cargo cache corruption

**Solution:**
```bash
cargo clean
rm -rf ~/.cargo/registry
cargo build
```

---

### Docker Issues

**Problem:** Port 3000 already in use

**Solution:**
```bash
# Find and kill process
lsof -i :3000
kill -9 <PID>

# Or use different port
PORT=3000 ./run.sh up
```

---

**Problem:** Docker build fails with "no space left on device"

**Solution:**
```bash
docker system prune -a
docker volume prune
```

---

## 👥 Contributing Guidelines

### Code Reviews

- Keep PRs small and focused (< 500 lines)
- Write descriptive commit messages
- Add tests for new features
- Update documentation
- Ensure CI passes

### Commit Messages

Follow conventional commits:

```
feat: add task priority sorting
fix: correct database connection pooling
docs: update API reference
test: add integration tests for repos
chore: update dependencies
```

### Pull Request Process

1. Create feature branch from `develop`
2. Make changes and commit
3. Push and open PR
4. Wait for CI checks
5. Address review comments
6. Merge when approved

---

## 📚 Additional Resources

### Documentation
- [Integration Success Report](integration/SUCCESS.md)
- [Quick Start Guide](integration/QUICK_START.md)
- [API Documentation](https://docs.rs/rustassistant)

### External Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum)
- [SQLx Guide](https://docs.rs/sqlx)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

---

## 📞 Support

- **Issues:** Open a GitHub issue
- **Discussions:** Use GitHub Discussions
- **Email:** dev@rustassistant.dev (if configured)

---

**Happy coding! 🦀 🚀**