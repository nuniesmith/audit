# RustAssistant — TODO Backlog

> This is a **living document** — it grows with the repo and is the primary interface between
> you and RustAssistant. Items are added manually, by the LLM Audit workflow (`todo-plan`,
> `todo-work`, `todo-review`), and by the Rust CLI. Each target repo gets its own `todo.md`
> that evolves over time, making it easy to pick up work during downtime.
>
> Items marked with ✅ have been completed. Items with ⚠️ are partial. Items with ❌ are blocked.

---

## 🔴 High Priority

### Build & CI
- [x] ~~Publish Docker image to Docker Hub~~ ✅ Done — `ci-cd.yml` builds AMD64 + ARM64 images and pushes to `docker.io/nuniesmith/rustassistant:latest` on every merge to `main`. The `llm-audit.yml` workflow now pulls the image and extracts the binary in seconds.
- [x] ~~Fix `Dockerfile.web` reference in ci-cd.yml~~ ✅ Fixed — was pointing to `docker/Dockerfile.web` which doesn't exist; corrected to `docker/Dockerfile`.
- [x] ~~Skip deployment in ci-cd.yml~~ ✅ Done — deploy job disabled (`if: false`). Images still build and push to Docker Hub; Raspberry Pi deploy can be re-enabled when needed.
- [ ] **PAT permissions for target repos** — `GH_PAT` needs `repo` scope (or fine-grained `Contents: Read and write`) on each target repo. Current failure: `403 Permission to nuniesmith/futures.git denied`. This is a settings fix, not a code fix.

### Rust-Native TODO System
- [ ] **`todo-scan` CLI command** — Add a `rustassistant todo-scan <repo-path>` command that scans inline `TODO/FIXME/HACK/XXX` comments, parses them with context, and outputs structured JSON. Replace the Python grep+LLM approach in the workflow with a call to this binary.
- [ ] **`todo-plan` CLI command** — Generate a GAMEPLAN from a `todo.md` file using the Rust LLM client (`src/grok_client.rs`). Should read `todo.md` + source context, call xAI, and output batched work items as JSON.
- [ ] **`todo-work` CLI command** — Execute a single batch from the gameplan: read the batch JSON, generate code changes via LLM, apply them, update `todo.md` status markers. This is the big one — replaces ~500 lines of inline Python in the workflow.
- [ ] **`todo.md` sync** — Build a `TodoFile` struct in Rust that can parse, update, and write back `todo.md` with proper status tracking (checkbox states, ✅/⚠️/❌ markers, timestamps). The workflow should call `rustassistant todo-sync` after each operation.
- [ ] **Wire workflow to Docker image** — The `llm-audit.yml` now pulls `docker.io/nuniesmith/rustassistant:latest` and extracts the binary. Once the CLI commands exist, replace the Python `todo-analyze`, `todo-plan`, and `todo-work` steps with calls to `./rustassistant-bin <command>`. Python stays as fallback if image pull fails.

### API & Data Layer
- [ ] Fix admin module — `pub mod admin` is commented out due to accessing non-existent `ApiState` fields (`src/api/mod.rs`)
- [ ] Implement proper document listing with filters — currently returns empty vec placeholder (`src/api/handlers.rs:345`)
- [ ] Implement document stats `by_type` counts — returns empty vec (`src/api/handlers.rs:132`)
- [ ] Calculate average chunk size — hardcoded to `0.0` (`src/api/handlers.rs:137`)

### Search & RAG
- [ ] Integrate RAG context search with LanceDB vector search — currently returns empty results (`src/research/worker.rs:275`)

### Indexing
- [ ] Implement concurrent batch indexing with semaphore — currently sequential only (`src/indexing.rs:395`)

---

## 🟡 Medium Priority

### CLI & Developer Experience
- [ ] Actually test the XAI API connection in `test-api` command — currently only checks if the key exists (`src/bin/cli.rs:726`)
- [ ] Parse detailed per-file test results in `TestRunner` — `results_by_file` is an empty `HashMap` (`src/tests_runner.rs:184`)

### Queue & Processing
- [ ] Implement tag refinement and project linking in queue processor tagging stage (`src/queue/processor.rs:430`)

### Web UI
- [ ] Add `pinned` field to `Document` struct for docs list and detail views (`src/web_ui_extensions.rs:375`, `src/web_ui_extensions.rs:719`)

---

## 🟢 Low Priority / Enhancements

### Large File Handling
- [x] ~~Skip LFS-tracked files (pre-trained models) during clone/audit~~ ✅ Fixed — `GIT_LFS_SKIP_SMUDGE=1` at clone time, LFS pointer files marked `assume-unchanged`, binary extensions excluded from file hashing and static scan
- [ ] Make the skip-extensions list configurable per-repo (currently hardcoded: `.onnx`, `.pt`, `.pth`, `.bin`, `.h5`, `.safetensors`, `.pkl`, `.pb`, `.tflite`, `.ckpt`, `.weights`, `.npy`, `.npz`)

### Docker & Compose
- [x] ~~Align `docker-compose.yml` README quick-start with actual SQLite-based setup~~ ✅ Fixed — README now references SQLite, port 3000, `docker compose`
- [ ] Add `docker-compose.yml` healthcheck for Redis connectivity from rustassistant container

### Workflow & CI/CD
- [x] ~~Move `llm-audit.yml` workflow to `nuniesmith/actions` repo where it belongs~~ ✅ Removed from rustassistant; see actions repo outline below
- [x] ~~Add a `docs/audit/` directory with `.gitkeep` so workflow report commits don't need to `mkdir`~~ ✅ Created
- [x] ~~Docker image pull in `llm-audit.yml`~~ ✅ Done — pulls `nuniesmith/rustassistant:latest` from Docker Hub, extracts binary to `./rustassistant-bin`, sets `ra_available` output for downstream steps
- [ ] Expose an `/api/audit` endpoint so the LLM audit workflow can leverage the Rust API + Redis cache instead of raw Python API calls
- [ ] Add `todo.md` generation to the `regular` audit mode — after the LLM audit, auto-append new findings as TODO items to the target repo's `todo.md`

### Code Quality
- [ ] Consolidate `todo_items` DB table usage with the `tasks` table — currently two parallel systems (`src/db/queue.rs:13`)
- [ ] Standardise error handling across API handlers (mix of `anyhow` and manual error responses)

---

## 📋 Notes

- The `auto_scanner` module already integrates `StaticAnalyzer` + `TodoScanner` + `PromptRouter` for smart file triage before LLM calls. The audit workflow should eventually call into this rather than reimplementing analysis in Python.
- Redis is configured in `docker-compose.yml` for LLM response caching (`allkeys-lru`, 256 MB). The workflow currently bypasses this entirely.
- The `.rustassistant/` directory is **tracked in git** (removed from `.gitignore`). It stores both the CLI analysis cache (`cache/`) and the LLM audit workflow's cross-run state (`cache.json`, `batches/`). See `.rustassistant/README.md` for details.
- **`todo.md` philosophy:** Each repo managed by RustAssistant has its own `todo.md` that serves as the single source of truth for pending work. It's meant to be an ever-changing document that grows with the repo — items get added by audits, completed by `todo-work`, and refined manually. Think of it as a living backlog that makes it easy to optimize downtime.
- **Migration path:** The current workflow is ~1900 lines of YAML+Python. The goal is to progressively move logic into the Rust binary (`src/llm_audit.rs`, `src/todo_scanner.rs`, `src/auto_scanner.rs`), publish via `ci-cd.yml` to Docker Hub (`nuniesmith/rustassistant:latest`), and have the workflow become a thin orchestrator that just calls `./rustassistant-bin <command>`. The image is already being pulled — now the CLI commands need to be implemented. This keeps the complex logic testable, cacheable, and reusable across CLI, Compose, and CI.
- **Pre-trained model files** (`.onnx`, `.pt`, etc.) in target repos are production artifacts, not source code. The workflow now skips them entirely via `GIT_LFS_SKIP_SMUDGE=1` at clone time. They're never downloaded, hashed, or included in the audit context.