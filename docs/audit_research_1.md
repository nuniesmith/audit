# Building a Rust-powered notes and repository intelligence system

A solo developer managing multiple Rust, KMP, and JS projects needs an integrated system that captures thoughts, tracks repositories, and uses LLM-powered analysis to surface next tasks and standardization opportunities. This report provides a complete technical blueprint using Axum, HTMX, Grok 4.1's **2M context window**, and a git-trackable vector database—all containerized with Docker Compose.

The most critical discovery: **custom file-based vector storage** with sharded JSON files is the only approach that truly satisfies git-trackable requirements, as neither Qdrant nor LanceDB support diffable small-file storage. Combined with Grok 4.1's massive context window (priced at just **$0.20/M input tokens** with 90% cache savings possible), this enables cost-effective whole-repository analysis.

---

## Git-trackable vector storage requires a custom approach

The requirement to store embeddings in git-trackable small files eliminates all major vector databases from consideration. Qdrant uses RocksDB with large binary segments, LanceDB uses columnar binary fragments, and pgvector requires a running Postgres instance. The solution is a **hybrid architecture**: JSON files for git-tracked embeddings, binary HNSW index (gitignored, rebuilt on startup).

**Recommended directory structure:**
```
embeddings/
├── vectors/                    # Git-tracked (JSON, diffable)
│   ├── 00/                     # Hash-sharded directories
│   │   ├── 00a1b2c3.json       # One file per document chunk
│   │   └── 00f5e6d7.json
│   └── 01/
├── index/
│   └── hnsw_graph.bin          # Gitignored, rebuild on load
└── manifest.json               # Git-tracked metadata
```

Each embedding file stores the vector alongside metadata for self-contained traceability:
```json
{
  "id": "abc123def456",
  "source": "notes/rust-patterns.md",
  "content_hash": "sha256:abc123...",
  "embedding": [0.123, -0.456, ...],
  "model": "text-embedding-3-small",
  "dimensions": 1536,
  "created_at": "2026-01-15T10:30:00Z"
}
```

For HNSW implementation, **`usearch`** delivers **10x faster performance than FAISS** with built-in filtered search capability. For simpler needs, `hnsw_rs` provides Rust-native mmap support and SIMD acceleration. File sizes at **1536 dimensions** run approximately **18KB per JSON file** including metadata—for 10,000 documents, expect ~180MB of git-tracked data with manageable diff churn when changes are isolated to modified files.

**Incremental update strategy**: Track content hashes in a manifest file and compare against current file hashes on startup. Use `git diff --name-only` integration to identify changed source files, re-embed only those, and incrementally update the HNSW index via `insert_data()` rather than full rebuilds.

---

## HTMX with Axum provides JavaScript-minimal interactivity

The **`axum-htmx`** crate (v0.8) provides first-class integration for detecting and responding to HTMX requests. The key pattern is serving partial HTML for HTMX requests and full pages for direct navigation:

```rust
use axum_htmx::{HxBoosted, HxRequest};

async fn get_notes(HxBoosted(boosted): HxBoosted) -> impl IntoResponse {
    if boosted {
        NotesPartialTemplate { notes: fetch_notes().await }
    } else {
        NotesPageTemplate { notes: fetch_notes().await }
    }
}
```

**Askama templates** provide compile-time checking and excellent performance (~330µs render times versus ~857µs for Tera). Structure templates with inheritance:

```html
<!-- templates/base.html -->
<!DOCTYPE html>
<html>
<head>
    <script src="https://unpkg.com/htmx.org@2.0.0"></script>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.classless.min.css">
</head>
<body>{% block content %}{% endblock %}</body>
</html>

<!-- templates/partials/note_item.html -->
<li id="note-{{ note.id }}" hx-target="this" hx-swap="outerHTML">
    <span>{{ note.content }}</span>
    <button hx-delete="/notes/{{ note.id }}">Delete</button>
</li>
```

For CSS, **PicoCSS classless** provides immediate styling of semantic HTML with ~10KB overhead and built-in dark mode—perfect for rapid development. The Tailwind CSS standalone CLI (no Node.js required) offers more control for complex layouts. Use **`syntect`** for server-side syntax highlighting of code files, avoiding JavaScript syntax highlighters entirely.

**Project structure recommendation:**
```
src/
├── handlers/
│   ├── pages.rs          # Full page handlers
│   └── partials.rs       # HTMX fragment handlers
├── templates/
│   └── mod.rs            # Askama template structs
templates/
├── base.html
├── pages/
└── partials/
static/
└── css/
```

---

## RAG architecture leverages Grok's massive context window

With **2M tokens** available, the retrieval strategy shifts from "find the needle" to "provide rich context and let the model reason." The pipeline combines vector similarity with BM25 keyword search using **Reciprocal Rank Fusion**:

```
Query → Embed → [Vector Search (usearch) + BM25 Search (tantivy)]
                              ↓
                    RRF Score Fusion → Top 50-100
                              ↓
                    Cross-Encoder Rerank → Top 5-15
                              ↓
                    U-Shaped Context Assembly → Grok 4.1
```

**Embedding generation options:**
- **Local**: `ort` crate (ONNX Runtime) provides 3-5x faster inference than Python with GPU support
- **API**: OpenAI `text-embedding-3-small` (1536 dims) or `text-embedding-3-large` (3072 dims)
- **Code-specific**: CodeBERT or GraphCodeBERT (768 dims) for semantic code search

**Chunking strategies differ by content type:**
- **Code files**: Split on function/class boundaries using language-specific separators (`\n\nfn `, `\nimpl `, `\nstruct `), 500-1000 tokens per chunk
- **Prose/notes**: Semantic chunking with 256-512 tokens, 10-20% overlap
- **Configs**: Keep whole files when under 1000 tokens

**Context assembly for "Lost in the Middle" problem**: Place highest-relevance chunks at the **start and end** of the context (U-shaped ordering), with medium-relevance content in the middle. This exploits LLM attention patterns where beginning and end receive more weight.

**Recommended crates:**
```toml
ort = "2.0.0-rc"           # ONNX Runtime embeddings
usearch = "2.16"           # Vector search (10x FAISS speed)
tantivy = "0.22"           # BM25 full-text search
tokenizers = "0.21"        # HuggingFace tokenizers
tiktoken-rs = "0.7"        # Token counting for context budgeting
```

---

## Grok 4.1 API integration maximizes the 2M context window

The xAI API is **fully OpenAI-compatible**—simply change the base URL to `https://api.x.ai/v1`. Pricing is highly competitive at **$0.20/M input, $0.50/M output, and just $0.02/M for cached tokens** (90% savings on repeated prefixes).

**Key implementation patterns:**

```rust
pub struct GrokClient {
    client: reqwest::Client,
    api_key: String,
}

impl GrokClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3600))  // Long timeout for 2M context
            .build()
            .expect("HTTP client");
        Self { client, api_key, base_url: "https://api.x.ai/v1".into() }
    }
}
```

**Structured outputs** ensure reliable JSON parsing for code analysis:
```rust
let response_format = ResponseFormat {
    format_type: "json_schema".to_string(),
    json_schema: JsonSchema {
        name: "code_analysis".to_string(),
        strict: true,
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "issues": { "type": "array", "items": { /* ... */ }},
                "quality_score": { "type": "number", "minimum": 0, "maximum": 100 }
            },
            "required": ["issues", "quality_score"]
        }),
    },
};
```

**Prompt templates for repository analysis:**

| Use Case | Prompt Pattern | Est. Cost |
|----------|---------------|-----------|
| Full repo analysis | 2M context with all files, structured output | ~$0.45 |
| Single file review | 256K context, non-reasoning model | ~$0.05 |
| Quick lint | Key files only, 256K | ~$0.05 |
| Daily monitoring | Incremental + cache prefix | ~$0.10 |

**Leverage prompt caching**: Keep the repository content as a stable system message prefix, varying only the task instruction. The `x-grok-conv-id` header helps xAI identify cacheable prefixes, achieving **90% cache hit rates** for repeated analysis patterns.

**Handling repos exceeding 2M tokens**: Chunk into ~1.5M segments, analyze each independently with shared structural context, then synthesize results in a final aggregation pass.

---

## Repository caching with git2 enables efficient change detection

The user's existing git2 crate usage provides the foundation for efficient repository caching. The key optimization is **SHA-based change detection**—comparing commit SHAs to identify exactly which files need re-processing:

```rust
pub fn get_changed_files(repo: &Repository, old_sha: &str, new_sha: &str) -> Vec<ChangedFile> {
    let old_tree = repo.find_commit(Oid::from_str(old_sha)?)?.tree()?;
    let new_tree = repo.find_commit(Oid::from_str(new_sha)?)?.tree()?;
    
    let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)?;
    
    diff.foreach(&mut |delta, _| {
        // Extract path and change type (Added/Modified/Deleted)
        true
    }, None, None, None)?;
}
```

**Caching architecture:**
- **Filesystem**: Clone repos to local cache directory, use git2's native format
- **Metadata**: bincode-serialized `CachedRepository` structs with tree snapshots
- **In-memory**: Moka cache with TTL for hot data (tree structures, recent file contents)

**Webhook vs polling**: Prefer GitHub webhooks for real-time updates (subscribe to `push` events), with scheduled polling as fallback. Use the `governor` crate for rate limiting (5000 requests/hour for authenticated GitHub API access).

**Memory-efficient tree representation:**
```rust
pub struct CompactRepoTree {
    nodes: Vec<TreeNode>,           // Flat storage
    path_index: BTreeMap<String, usize>,  // O(1) path lookup
    commit_sha: String,
}
```

This structure enables efficient tree diffing by comparing blob SHAs directly, avoiding content comparison for unchanged files.

---

## Docker Compose architecture spans development to production

The **base + override pattern** cleanly separates environment-specific configuration:

```yaml
# docker-compose.yml (base)
services:
  app:
    depends_on:
      postgres: { condition: service_healthy }
      redis: { condition: service_healthy }

# docker-compose.override.yml (auto-loaded for local dev)
services:
  app:
    build: { dockerfile: Dockerfile.dev }
    volumes:
      - ./vector-data:/app/data:ro  # Git-tracked embeddings
    environment:
      - RUST_LOG=debug
```

**Optimal Rust Dockerfile with cargo-chef** (5x faster rebuilds via dependency caching):
```dockerfile
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json  # Cached layer
COPY . .
RUN cargo build --release && strip target/release/myapp

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
COPY --from=builder /app/target/release/myapp /usr/local/bin/
USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/myapp"]
```

**Runtime image comparison:**
| Image | Size | Best For |
|-------|------|----------|
| `gcr.io/distroless/cc` | ~20MB | glibc Rust binaries (recommended) |
| `scratch` | ~8-15MB | musl static binaries |
| `alpine` | ~7MB | When shell access needed |

**Secrets management**: Use Docker Compose file-based secrets for development, reading via `_FILE` suffix pattern:
```rust
fn get_secret(name: &str) -> String {
    if let Ok(path) = env::var(format!("{}_FILE", name)) {
        return fs::read_to_string(path).unwrap().trim().to_string();
    }
    env::var(name).unwrap()
}
```

**Monitoring stack** (Prometheus + Grafana + Loki): Mount configuration files as read-only volumes, use Grafana provisioning for dashboards. For solo developer alerting, configure Alertmanager with email or Discord webhooks for critical issues only.

**GitHub Actions CI/CD workflow:**
1. Test: `cargo fmt --check`, `cargo clippy`, `cargo test`
2. Build: Multi-stage Docker build with Buildx for amd64/arm64
3. Push: ghcr.io container registry
4. Deploy: SSH to Linode, `docker compose pull && docker compose up -d`

---

## Cross-repo standardization leverages templates and LLM detection

**Template synchronization with Copier**: Unlike Cookiecutter, Copier supports **updating existing projects** when templates evolve—essential for maintaining consistency across repos over time.

**Centralized configuration pattern:**
```
shared-configs/
├── docker/compose.base.yml
├── ci/rust-ci.yml          # Reusable GitHub workflow
├── linting/clippy.toml
└── templates/README.md
```

**Detecting drift automatically**: Weekly cron job compares each repo's configs against templates, generates divergence score, and optionally creates PRs for sync.

**LLM-powered pattern detection** prompts:
```
Analyze code from these repositories for shared logic:
- Repo A (Rust): config parsing in src/config/
- Repo B (KMP): settings loader
- Repo C (JS): environment config

Identify:
1. Duplicate configuration schemas
2. Common validation logic that could be extracted
3. Candidates for shared library extraction
```

Code embeddings (using CodeBERT) enable **semantic clone detection**—finding functionally similar code even when syntax differs across languages. This surfaces refactoring opportunities that text-based tools miss.

---

## Note-to-task workflow routes thoughts to the right projects

The capture system must be **sub-3-second** to avoid breaking developer flow. Multiple input channels funnel to a single inbox:

```
Mobile → iOS Shortcut → POST /api/notes
Desktop → CLI `note "fix auth bug"` → POST /api/notes  
IDE → Extract TODO comments → POST /api/notes
Voice → Transcription → POST /api/notes
         ↓
    [ INBOX TABLE ]
         ↓
    Daily Processing: Tag + Route
         ↓
    [ PROCESSED ] → GitHub Issue (optional)
```

**Tag taxonomy:**
- **Project**: `#rust-api`, `#kmp-mobile`, `#web-frontend`
- **Type**: `#bug`, `#feature`, `#idea`, `#research`
- **Energy**: `#deep-work`, `#quick-win`
- **Context**: `#blocked`, `#waiting-on`

**Task recommendation scoring** (for "what should I work on next"):
```
Score = Urgency(0.25) + Importance(0.25) + MomentumBonus(0.15) 
      + EnergyMatch(0.15) + (1/ContextSwitchCost)(0.10) + Dependencies(0.10)
```

**LLM-powered recommendation prompt:**
```
Given my projects' state:
- rust-api: 5 open issues, failing CI (test_auth), last commit 2 days ago
- kmp-mobile: 3 open issues, CI passing, last commit 1 week ago

My energy: Medium | Available time: 2 hours | Recent context: rust-api

Recommend next 3 tasks with reasoning.
```

**Database schema essentials:**
```sql
CREATE TABLE notes (
    id UUID PRIMARY KEY,
    content TEXT NOT NULL,
    status VARCHAR(20) DEFAULT 'inbox',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE note_project_links (
    note_id UUID REFERENCES notes(id),
    project_id UUID REFERENCES projects(id),
    github_issue_url TEXT,
    file_path TEXT
);
```

---

## Implementation roadmap for solo developer

**Phase 1 (Week 1-2): Core infrastructure**
- SQLite notes database with tags
- CLI capture tool: `note "text" --tag=project`
- Basic Axum + Askama + HTMX web interface
- Docker Compose for local services

**Phase 2 (Week 3-4): Repository intelligence**
- git2 repository cloning and tree caching
- Custom file-based vector storage (JSON + usearch)
- Embedding pipeline with content-hash change detection
- Basic RAG queries

**Phase 3 (Week 5-6): LLM integration**
- Grok 4.1 API client with structured outputs
- Repository analysis prompts (issues, patterns, quality scores)
- Note-to-task routing with GitHub issue creation
- Task recommendation engine

**Phase 4 (Week 7-8): Production hardening**
- GitHub Actions CI/CD pipeline
- Prometheus metrics and Grafana dashboards
- Nginx reverse proxy with SSL
- Linode deployment

**Key technology choices summary:**

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Vector storage | Custom JSON files + usearch | Only git-trackable option |
| Web framework | Axum + HTMX + Askama | Rust-native, minimal JS |
| LLM | Grok 4.1 (2M context) | Cost-effective repo analysis |
| Embeddings | `ort` (ONNX) or OpenAI API | Local speed vs API simplicity |
| Search | usearch + tantivy | Hybrid vector + BM25 |
| Container | distroless + cargo-chef | Security + fast builds |
| Templates | Copier | Update support |

This architecture delivers a unified system where quick thoughts flow seamlessly into tracked tasks, repository analysis surfaces next actions automatically, and standardization opportunities emerge from cross-repo pattern detection—all running on infrastructure the solo developer fully controls.