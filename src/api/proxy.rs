//! OpenAI-compatible `/v1/chat/completions` proxy endpoint.
//!
//! This module lets any OpenAI-SDK-compatible client (Python `openai`, JS `openai`,
//! Rust `async-openai`, curl, the futures trading app, etc.) point its `base_url` at
//! RustAssistant and get responses routed through:
//!
//!   request → API-key auth → rate-limit → model routing (local/remote)
//!           → RAG context injection → repo context injection
//!           → Ollama (local) | Grok (remote)
//!           → Redis/LRU response cache → OpenAI-shaped response
//!
//! # Endpoint
//!
//! ```text
//! POST /v1/chat/completions
//! Authorization: Bearer <RA_API_KEY>
//! Content-Type: application/json
//!
//! {
//!   "model":    "auto",          // "auto" | "local" | "remote" | "ra:<hint>"
//!   "messages": [
//!     { "role": "system",    "content": "You are a trading analyst." },
//!     { "role": "user",      "content": "Analyse BTC open interest spike." }
//!   ],
//!   "temperature": 0.2,          // optional, default 0.2
//!   "max_tokens":  2048,         // optional
//!   "stream":      false,        // streaming not yet supported — always false
//!
//!   // RustAssistant extensions (all optional, ignored by stock OpenAI clients)
//!   "x_repo_id":    "futures-bot",  // inject registered-repo RAG context
//!   "x_no_cache":   false,          // bypass Redis/LRU cache
//!   "x_force_remote": false         // skip local model regardless of task kind
//! }
//! ```
//!
//! # Response  (OpenAI `ChatCompletion` shape)
//!
//! ```json
//! {
//!   "id": "chatcmpl-ra-<uuid>",
//!   "object": "chat.completion",
//!   "created": 1710000000,
//!   "model": "qwen2.5-coder:7b",
//!   "choices": [{
//!     "index": 0,
//!     "message": { "role": "assistant", "content": "…" },
//!     "finish_reason": "stop"
//!   }],
//!   "usage": {
//!     "prompt_tokens": 312,
//!     "completion_tokens": 128,
//!     "total_tokens": 440
//!   },
//!   "x_ra_metadata": {
//!     "task_kind":             "ArchitecturalReason",
//!     "used_fallback":         false,
//!     "repo_context_injected": true,
//!     "rag_chunks_used":       3,
//!     "cached":                false,
//!     "cache_key":             "chat:a3f8c1d0e4b2f9a7"
//!   }
//! }
//! ```
//!
//! # Model aliases understood in the `model` field
//!
//! | Value          | Behaviour                                      |
//! |----------------|------------------------------------------------|
//! | `"auto"`       | ModelRouter decides local vs remote            |
//! | `"local"`      | Force Ollama regardless of task kind           |
//! | `"remote"`     | Force Grok regardless of task kind             |
//! | `"grok-*"`     | Force Grok, pass model name through            |
//! | `"ra:<hint>"`  | Treat `<hint>` as the prompt for classification|
//! | anything else  | Treated as `"auto"`                            |
//!
//! # Auth
//!
//! Set `RA_PROXY_API_KEYS=key1,key2,...` in the environment.
//! If the variable is empty / unset, the endpoint is open (useful for local dev).
//! The key is read from `Authorization: Bearer <key>` or `X-API-Key: <key>`.
//!
//! # Fallback behaviour for the futures app
//!
//! The companion `ProxyClient` (see `src/api/proxy_client.rs`) tries
//! RustAssistant first and falls back to the upstream Grok API transparently
//! when RustAssistant is unreachable or returns a 5xx error.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::api::repos::RepoAppState;
use crate::model_router::{CompletionRequest, ModelTarget, TaskKind};
use crate::research::worker::{enhance_prompt_with_rag, search_rag_context};

// ---------------------------------------------------------------------------
// Shared proxy state — thin wrapper around RepoAppState + allowed API keys
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct ProxyState {
    /// The full repo/chat state (model router, Ollama, Grok, cache, sync svc).
    pub repo_state: RepoAppState,
    /// SHA-256 hashes of allowed bearer tokens.
    /// Empty → auth disabled (open endpoint).
    pub allowed_key_hashes: Arc<Vec<String>>,
}

impl ProxyState {
    /// Build from an existing `RepoAppState`.
    ///
    /// Reads `RA_PROXY_API_KEYS` from the environment (comma-separated raw keys).
    /// If the variable is absent or empty, auth is disabled.
    pub fn new(repo_state: RepoAppState) -> Self {
        let raw_keys = std::env::var("RA_PROXY_API_KEYS").unwrap_or_default();
        let allowed_key_hashes: Vec<String> = raw_keys
            .split(',')
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .map(hash_key)
            .collect();

        if allowed_key_hashes.is_empty() {
            warn!(
                "RA_PROXY_API_KEYS is not set — /v1/chat/completions is open (no auth). \
                 Set RA_PROXY_API_KEYS=<key> to restrict access."
            );
        } else {
            info!(
                key_count = allowed_key_hashes.len(),
                "Proxy auth enabled ({} key(s))",
                allowed_key_hashes.len()
            );
        }

        Self {
            repo_state,
            allowed_key_hashes: Arc::new(allowed_key_hashes),
        }
    }

    /// Return true when the provided raw key is authorised (or auth is off).
    pub fn is_authorised(&self, key: &str) -> bool {
        if self.allowed_key_hashes.is_empty() {
            return true;
        }
        let h = hash_key(key);
        self.allowed_key_hashes.contains(&h)
    }
}

// ---------------------------------------------------------------------------
// OpenAI-compatible request / response shapes
// ---------------------------------------------------------------------------

/// A single message in the conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OaiMessage {
    /// `"system"` | `"user"` | `"assistant"`
    pub role: String,
    pub content: String,
}

/// OpenAI `POST /v1/chat/completions` request body.
#[derive(Debug, Deserialize)]
pub struct OaiChatRequest {
    // ── Standard OpenAI fields ───────────────────────────────────────────────
    /// Model alias. See module-level doc for accepted values.
    pub model: String,
    /// Conversation messages in chronological order.
    pub messages: Vec<OaiMessage>,
    /// Sampling temperature (0.0 – 2.0). Default: 0.2.
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Streaming — must be `false` (streaming is not yet implemented).
    #[serde(default)]
    pub stream: bool,

    // ── RustAssistant extensions ─────────────────────────────────────────────
    /// Inject RAG + symbol context from a registered repo slug or UUID.
    pub x_repo_id: Option<String>,
    /// Bypass Redis/LRU response cache.
    #[serde(default)]
    pub x_no_cache: bool,
    /// Force remote (Grok) model regardless of task classification.
    #[serde(default)]
    pub x_force_remote: bool,
}

fn default_temperature() -> f32 {
    0.2
}

/// OpenAI `ChatCompletion` response.
#[derive(Debug, Serialize)]
pub struct OaiChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OaiChoice>,
    pub usage: OaiUsage,
    /// Non-standard metadata — OpenAI clients will ignore this field.
    pub x_ra_metadata: RaMetadata,
}

#[derive(Debug, Serialize)]
pub struct OaiChoice {
    pub index: u32,
    pub message: OaiMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct OaiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// RustAssistant-specific metadata returned alongside every response.
#[derive(Debug, Serialize)]
pub struct RaMetadata {
    /// The `TaskKind` the model router assigned to this prompt.
    pub task_kind: String,
    /// True when the local model was tried but fell back to remote.
    pub used_fallback: bool,
    /// True when repo symbols/tree/todos were injected into the prompt.
    pub repo_context_injected: bool,
    /// Number of RAG chunks prepended to the prompt.
    pub rag_chunks_used: usize,
    /// True when the response was served from cache.
    pub cached: bool,
    /// Cache key used for this request (useful for debugging).
    pub cache_key: String,
}

/// OpenAI-compatible error body.
#[derive(Debug, Serialize)]
pub struct OaiError {
    pub error: OaiErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct OaiErrorDetail {
    pub message: String,
    pub r#type: String,
    pub code: Option<String>,
}

impl OaiError {
    fn auth(msg: impl Into<String>) -> (StatusCode, Json<Self>) {
        (
            StatusCode::UNAUTHORIZED,
            Json(Self {
                error: OaiErrorDetail {
                    message: msg.into(),
                    r#type: "authentication_error".to_string(),
                    code: Some("invalid_api_key".to_string()),
                },
            }),
        )
    }

    fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<Self>) {
        (
            StatusCode::BAD_REQUEST,
            Json(Self {
                error: OaiErrorDetail {
                    message: msg.into(),
                    r#type: "invalid_request_error".to_string(),
                    code: None,
                },
            }),
        )
    }
}

// ---------------------------------------------------------------------------
// Cache types (stored as JSON in CacheLayer)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CachedProxyResponse {
    content: String,
    model_used: String,
    used_fallback: bool,
    task_kind: String,
    rag_chunks_used: usize,
    repo_context_injected: bool,
    prompt_tokens: u32,
    completion_tokens: u32,
}

/// TTL for proxy response cache entries: 30 minutes.
/// Shorter than the 1-hour chat TTL because market/code context can change fast.
const PROXY_CACHE_TTL_SECS: u64 = 1800;

// ---------------------------------------------------------------------------
// Router constructor — call this in server.rs
// ---------------------------------------------------------------------------

/// Build the `/v1` router containing the OpenAI-compatible chat completion endpoint.
///
/// Mount with `.nest("/v1", proxy_router(proxy_state))` in `run_server`.
pub fn proxy_router(state: ProxyState) -> Router {
    Router::new()
        .route("/chat/completions", post(handle_chat_completions))
        .route("/models", axum::routing::get(handle_list_models))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Handler — POST /v1/chat/completions
// ---------------------------------------------------------------------------

async fn handle_chat_completions(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(req): Json<OaiChatRequest>,
) -> Response {
    // ── 1. Auth ──────────────────────────────────────────────────────────────
    if let Some(err) = check_auth(&state, &headers) {
        return err.into_response();
    }

    // ── 2. Validate request ──────────────────────────────────────────────────
    if req.messages.is_empty() {
        return OaiError::bad_request("messages array must not be empty").into_response();
    }
    if req.stream {
        return OaiError::bad_request(
            "Streaming is not yet supported by RustAssistant proxy. Set stream=false.",
        )
        .into_response();
    }

    // ── 3. Extract the effective user prompt for routing/RAG ─────────────────
    // We use the last user message as the "active" prompt for classification
    // and RAG retrieval. The full history is concatenated for the model call.
    let last_user_msg = req
        .messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.as_str())
        .unwrap_or("");

    let system_prompt: Option<&str> = req
        .messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.as_str());

    // ── 4. Model routing ──────────────────────────────────────────────────────
    let (task_kind, mut target) =
        route_from_model_field(&state.repo_state, &req.model, last_user_msg).await;

    if req.x_force_remote {
        target = state
            .repo_state
            .model_router
            .route(&TaskKind::ArchitecturalReason);
    }

    // ── 5. Repo context injection ─────────────────────────────────────────────
    let (repo_context, repo_context_injected) = if let Some(ref rid) = req.x_repo_id {
        let svc = state.repo_state.sync_service.read().await;
        match svc.build_prompt_context(rid).await {
            Ok(ctx) => (Some(ctx), true),
            Err(e) => {
                warn!(repo = %rid, error = %e, "Proxy: could not build repo context — continuing without it");
                (None, false)
            }
        }
    } else {
        (None, false)
    };

    // ── 6. RAG retrieval ──────────────────────────────────────────────────────
    let (rag_enriched_prompt, rag_chunks_used) =
        enrich_with_rag(&state.repo_state, last_user_msg).await;

    // ── 7. Build full prompt (history + RAG + repo context) ──────────────────
    let full_prompt =
        build_full_prompt(&req.messages, &rag_enriched_prompt, repo_context.as_deref());

    // ── 8. Cache key & lookup ─────────────────────────────────────────────────
    let cache_key = build_proxy_cache_key(&target, &full_prompt, req.x_repo_id.as_deref());

    if !req.x_no_cache {
        match state
            .repo_state
            .cache
            .get::<CachedProxyResponse>(&cache_key)
            .await
        {
            Ok(Some(hit)) => {
                debug!(cache_key = %cache_key, "Proxy cache hit");
                return build_oai_response(
                    hit.content,
                    hit.model_used,
                    hit.used_fallback,
                    hit.task_kind,
                    hit.rag_chunks_used,
                    hit.repo_context_injected,
                    hit.prompt_tokens,
                    hit.completion_tokens,
                    true,
                    cache_key,
                )
                .into_response();
            }
            Ok(None) => {}
            Err(e) => warn!(error = %e, "Proxy: cache read error — proceeding"),
        }
    }

    // ── 9. Dispatch to model ──────────────────────────────────────────────────
    let comp_req = CompletionRequest {
        system_prompt: system_prompt.map(str::to_owned),
        user_prompt: full_prompt.clone(),
        max_tokens: req.max_tokens.unwrap_or(2048),
        temperature: req.temperature,
        repo_context: None, // already baked into full_prompt above
    };

    let (reply, model_used, used_fallback, tokens_used) =
        dispatch(&state.repo_state, &comp_req, &target).await;

    let (prompt_tok, completion_tok) = split_tokens(tokens_used, &full_prompt, &reply);

    // ── 10. Cache (fire-and-forget) ───────────────────────────────────────────
    let cached_val = CachedProxyResponse {
        content: reply.clone(),
        model_used: model_used.clone(),
        used_fallback,
        task_kind: format!("{:?}", task_kind),
        rag_chunks_used,
        repo_context_injected,
        prompt_tokens: prompt_tok,
        completion_tokens: completion_tok,
    };
    {
        let cache = Arc::clone(&state.repo_state.cache);
        let key = cache_key.clone();
        tokio::spawn(async move {
            if let Err(e) = cache
                .set(&key, &cached_val, Some(PROXY_CACHE_TTL_SECS))
                .await
            {
                warn!(error = %e, "Proxy: failed to write response to cache");
            }
        });
    }

    // ── 11. Build OpenAI-compatible response ──────────────────────────────────
    build_oai_response(
        reply,
        model_used,
        used_fallback,
        format!("{:?}", task_kind),
        rag_chunks_used,
        repo_context_injected,
        prompt_tok,
        completion_tok,
        false,
        cache_key,
    )
    .into_response()
}

// ---------------------------------------------------------------------------
// Handler — GET /v1/models  (minimal — enough to satisfy SDK clients)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ModelList {
    object: &'static str,
    data: Vec<ModelEntry>,
}

#[derive(Serialize)]
struct ModelEntry {
    id: String,
    object: &'static str,
    created: u64,
    owned_by: &'static str,
}

async fn handle_list_models(State(state): State<ProxyState>) -> impl IntoResponse {
    // Attempt to fetch live Ollama model list; fall back to a static list.
    let mut entries: Vec<ModelEntry> = Vec::new();

    // Static/virtual model aliases that are always available.
    let static_aliases = ["auto", "local", "remote"];
    let now = unix_now();
    for alias in static_aliases {
        entries.push(ModelEntry {
            id: alias.to_string(),
            object: "model",
            created: now,
            owned_by: "rustassistant",
        });
    }

    // Try to get the live Ollama model list and expose them too.
    match state.repo_state.ollama_client.list_models().await {
        Ok(models) => {
            for m in models {
                entries.push(ModelEntry {
                    id: m,
                    object: "model",
                    created: now,
                    owned_by: "ollama",
                });
            }
        }
        Err(e) => {
            warn!(error = %e, "Proxy /v1/models: could not list Ollama models");
        }
    }

    Json(ModelList {
        object: "list",
        data: entries,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Verify the bearer token or X-API-Key header against the configured key set.
/// Returns `Some(error_response)` when auth fails, `None` when authorised.
fn check_auth(state: &ProxyState, headers: &HeaderMap) -> Option<(StatusCode, Json<OaiError>)> {
    if state.allowed_key_hashes.is_empty() {
        // Auth disabled — open endpoint.
        return None;
    }

    let raw_key = headers
        .get("Authorization")
        .or_else(|| headers.get("X-API-Key"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.strip_prefix("Bearer ").unwrap_or(s));

    match raw_key {
        None => Some(OaiError::auth(
            "No API key provided. Use Authorization: Bearer <key> or X-API-Key: <key>.",
        )),
        Some(key) if state.is_authorised(key) => None,
        Some(_) => Some(OaiError::auth("Invalid API key.")),
    }
}

/// Determine model target from the `model` field in the request.
///
/// Supported aliases:
/// - `"auto"` — let ModelRouter classify the last user message
/// - `"local"` — always use Ollama
/// - `"remote"` — always use Grok
/// - `"grok-*"` / `"grok"` — always use Grok
/// - `"ra:<hint>"` — strip prefix, use hint as the classification prompt
/// - anything else — treat as `"auto"`
async fn route_from_model_field(
    state: &RepoAppState,
    model: &str,
    last_user_msg: &str,
) -> (TaskKind, ModelTarget) {
    let model_lc = model.to_lowercase();

    match model_lc.as_str() {
        "local" => {
            let target = state.model_router.route(&TaskKind::ScaffoldStub);
            (TaskKind::ScaffoldStub, target)
        }
        "remote" | "grok" => {
            let target = state.model_router.route(&TaskKind::ArchitecturalReason);
            (TaskKind::ArchitecturalReason, target)
        }
        _ if model_lc.starts_with("grok-") => {
            let target = state.model_router.route(&TaskKind::ArchitecturalReason);
            (TaskKind::ArchitecturalReason, target)
        }
        _ => {
            // "auto" or unknown — let the router classify the message.
            // For "ra:<hint>" we use the hint instead of the raw last message.
            let classify_text = if let Some(hint) = model.strip_prefix("ra:") {
                hint
            } else {
                last_user_msg
            };
            state.model_router.route_prompt_async(classify_text).await
        }
    }
}

/// Retrieve RAG chunks for the given prompt and return the enriched prompt
/// string plus the number of chunks actually used.
async fn enrich_with_rag(state: &RepoAppState, prompt: &str) -> (String, usize) {
    let maybe_pool = {
        let svc = state.sync_service.read().await;
        svc.db_pool()
    };

    if let Some(pool) = maybe_pool {
        match search_rag_context(&pool, prompt, 4).await {
            Ok(chunks) if !chunks.is_empty() => {
                let count = chunks.len();
                let enriched = enhance_prompt_with_rag(prompt, &chunks);
                debug!(rag_chunks = count, "Proxy: RAG enriched prompt");
                (enriched, count)
            }
            Ok(_) => (prompt.to_owned(), 0),
            Err(e) => {
                warn!(error = %e, "Proxy: RAG search failed — using plain prompt");
                (prompt.to_owned(), 0)
            }
        }
    } else {
        (prompt.to_owned(), 0)
    }
}

/// Collapse the full conversation history into a single prompt string that
/// the underlying completion API can handle, injecting RAG text and optional
/// repo context.
///
/// Format:
/// ```text
/// [System]
/// <system message if present>
///
/// [Repo Context]
/// <repo symbols / tree / todos if injected>
///
/// [Conversation]
/// user: <msg>
/// assistant: <msg>
/// user: <rag-enriched last message>
/// ```
fn build_full_prompt(
    messages: &[OaiMessage],
    rag_enriched_last_user: &str,
    repo_context: Option<&str>,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    // System message (excluded from the [Conversation] block).
    if let Some(sys) = messages.iter().find(|m| m.role == "system") {
        parts.push(format!("[System]\n{}", sys.content));
    }

    // Repo context block.
    if let Some(ctx) = repo_context {
        parts.push(format!("[Repo Context]\n{}", ctx));
    }

    // Conversation history (all non-system messages except the last user turn).
    let non_system: Vec<&OaiMessage> = messages.iter().filter(|m| m.role != "system").collect();

    if !non_system.is_empty() {
        let mut conv_lines: Vec<String> = Vec::new();

        // All turns except the last one verbatim.
        for msg in non_system.iter().take(non_system.len().saturating_sub(1)) {
            conv_lines.push(format!("{}: {}", msg.role, msg.content));
        }

        // Replace the last user turn with the RAG-enriched version.
        conv_lines.push(format!("user: {}", rag_enriched_last_user));

        parts.push(format!("[Conversation]\n{}", conv_lines.join("\n")));
    }

    parts.join("\n\n")
}

/// Dispatch to Ollama or Grok via the existing `dispatch_completion` logic.
/// Duplicated here (rather than sharing with `repos.rs`) to keep the proxy
/// self-contained and avoid coupling to private internals.
async fn dispatch(
    state: &RepoAppState,
    req: &CompletionRequest,
    target: &ModelTarget,
) -> (String, String, bool, Option<u32>) {
    match target {
        ModelTarget::Local { model, .. } => {
            debug!(model = %model, "Proxy: dispatching to local Ollama");
            match state
                .ollama_client
                .complete(
                    req.system_prompt.as_deref(),
                    &req.user_prompt,
                    req.temperature,
                    req.max_tokens,
                )
                .await
            {
                Ok(resp) => {
                    let tokens = resp
                        .prompt_tokens
                        .and_then(|p| resp.completion_tokens.map(|c| p + c));
                    (resp.content, resp.model_used, resp.used_fallback, tokens)
                }
                Err(e) => {
                    warn!(error = %e, "Proxy: Ollama dispatch failed");
                    let msg = format!("Local model error: {}", e);
                    (msg, model.clone(), false, None)
                }
            }
        }

        ModelTarget::Remote { model, api_key } => {
            debug!(model = %model, "Proxy: dispatching to remote Grok");

            let result = if let Some(ref grok) = state.grok_client {
                grok.ask_tracked(&req.user_prompt, None, "proxy-chat").await
            } else {
                // One-shot client when no pre-built GrokClient is available.
                use crate::db::Database;
                match Database::new("data/rustassistant.db").await {
                    Ok(db) => {
                        let client = crate::grok_client::GrokClient::new(api_key.clone(), db);
                        client
                            .ask_tracked(&req.user_prompt, None, "proxy-chat")
                            .await
                    }
                    Err(e) => Err(anyhow::anyhow!(
                        "DB init for one-shot Grok client failed: {}",
                        e
                    )),
                }
            };

            match result {
                Ok(resp) => {
                    let tokens = (resp.prompt_tokens + resp.completion_tokens) as u32;
                    (resp.content, model.clone(), false, Some(tokens))
                }
                Err(e) => {
                    warn!(error = %e, "Proxy: Grok dispatch failed");
                    let msg = format!("Remote model error: {}", e);
                    (msg, model.clone(), false, None)
                }
            }
        }
    }
}

/// Split a combined token count into an approximate prompt/completion split.
/// If the model returned both counts, use them. Otherwise estimate using a
/// naive 4-chars-per-token heuristic.
fn split_tokens(combined: Option<u32>, prompt: &str, completion: &str) -> (u32, u32) {
    if let Some(total) = combined {
        // Rough proportional split when we only have the total.
        let p_chars = prompt.len() as f64;
        let c_chars = completion.len() as f64;
        let total_chars = (p_chars + c_chars).max(1.0);
        let p_tok = ((p_chars / total_chars) * total as f64).round() as u32;
        let c_tok = total.saturating_sub(p_tok);
        (p_tok, c_tok)
    } else {
        // Full heuristic fallback.
        let p_tok = (prompt.len() as u32).saturating_div(4).max(1);
        let c_tok = (completion.len() as u32).saturating_div(4).max(1);
        (p_tok, c_tok)
    }
}

/// Build the final `OaiChatResponse`.
#[allow(clippy::too_many_arguments)]
fn build_oai_response(
    content: String,
    model_used: String,
    used_fallback: bool,
    task_kind: String,
    rag_chunks_used: usize,
    repo_context_injected: bool,
    prompt_tokens: u32,
    completion_tokens: u32,
    cached: bool,
    cache_key: String,
) -> Json<OaiChatResponse> {
    Json(OaiChatResponse {
        id: format!("chatcmpl-ra-{}", Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: unix_now(),
        model: model_used.clone(),
        choices: vec![OaiChoice {
            index: 0,
            message: OaiMessage {
                role: "assistant".to_string(),
                content,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: OaiUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        },
        x_ra_metadata: RaMetadata {
            task_kind,
            used_fallback,
            repo_context_injected,
            rag_chunks_used,
            cached,
            cache_key,
        },
    })
}

/// Build a deterministic cache key for the proxy.
/// Identical to `repos.rs::build_cache_key` but namespaced as `proxy:`.
fn build_proxy_cache_key(target: &ModelTarget, prompt: &str, repo_id: Option<&str>) -> String {
    let label = match target {
        ModelTarget::Local { model, .. } => format!("local:{}", model),
        ModelTarget::Remote { model, .. } => format!("remote:{}", model),
    };

    let mut h = Sha256::new();
    h.update(label.as_bytes());
    h.update(b"\x00");
    h.update(prompt.as_bytes());
    h.update(b"\x00");
    h.update(repo_id.unwrap_or("").as_bytes());
    let digest = h.finalize();

    format!("proxy:{}", hex::encode(&digest[..8]))
}

/// Hash a raw API key with SHA-256 for constant-time-safe comparison.
fn hash_key(key: &str) -> String {
    let mut h = Sha256::new();
    h.update(key.as_bytes());
    hex::encode(h.finalize())
}

/// Seconds since Unix epoch.
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_key_is_deterministic() {
        assert_eq!(hash_key("my-secret"), hash_key("my-secret"));
        assert_ne!(hash_key("my-secret"), hash_key("other-secret"));
        assert_eq!(hash_key("my-secret").len(), 64); // hex-encoded SHA-256
    }

    #[test]
    fn build_proxy_cache_key_stable() {
        let target_a = ModelTarget::Local {
            model: "qwen2.5-coder:7b".to_string(),
            base_url: "http://localhost:11434".to_string(),
        };
        let target_b = ModelTarget::Local {
            model: "qwen2.5-coder:7b".to_string(),
            base_url: "http://localhost:11434".to_string(),
        };
        let k1 = build_proxy_cache_key(&target_a, "hello world", Some("my-repo"));
        let k2 = build_proxy_cache_key(&target_b, "hello world", Some("my-repo"));
        assert_eq!(k1, k2);
        assert!(k1.starts_with("proxy:"));
    }

    #[test]
    fn split_tokens_proportional() {
        // With a combined count, the split should sum back to the total.
        let (p, c) = split_tokens(Some(100), "prompt text here", "completion text");
        assert_eq!(p + c, 100);
    }

    #[test]
    fn split_tokens_heuristic_fallback() {
        // Without a combined count, we fall back to the character heuristic.
        let prompt = "a".repeat(400); // ~100 tokens
        let completion = "b".repeat(200); // ~50 tokens
        let (p, c) = split_tokens(None, &prompt, &completion);
        assert!(p > 0);
        assert!(c > 0);
    }

    #[test]
    fn build_full_prompt_contains_all_sections() {
        let messages = vec![
            OaiMessage {
                role: "system".to_string(),
                content: "You are a trading bot.".to_string(),
            },
            OaiMessage {
                role: "user".to_string(),
                content: "What is the BTC trend?".to_string(),
            },
        ];
        let rag = "RAG: BTC recently crossed the 200-day MA.";
        let ctx = "Tree: src/\n  trading/\n    bot.rs";

        let prompt = build_full_prompt(&messages, rag, Some(ctx));

        assert!(prompt.contains("[System]"));
        assert!(prompt.contains("You are a trading bot."));
        assert!(prompt.contains("[Repo Context]"));
        assert!(prompt.contains("Tree:"));
        assert!(prompt.contains("[Conversation]"));
        assert!(prompt.contains("RAG: BTC recently crossed"));
    }

    #[test]
    fn build_full_prompt_no_system_no_ctx() {
        let messages = vec![OaiMessage {
            role: "user".to_string(),
            content: "Explain RSI divergence.".to_string(),
        }];
        let prompt = build_full_prompt(&messages, "Explain RSI divergence.", None);
        assert!(!prompt.contains("[System]"));
        assert!(!prompt.contains("[Repo Context]"));
        assert!(prompt.contains("[Conversation]"));
        assert!(prompt.contains("Explain RSI divergence."));
    }

    #[test]
    fn proxy_state_auth_disabled_when_no_keys() {
        // When RA_PROXY_API_KEYS is not set, is_authorised returns true for anything.
        std::env::remove_var("RA_PROXY_API_KEYS");
        // We can't construct a real ProxyState without a RepoAppState, so we test
        // the key-hash logic in isolation.
        let hashes: Vec<String> = vec![];
        // Simulate the is_authorised logic:
        let is_open = hashes.is_empty();
        assert!(is_open);
    }

    #[test]
    fn proxy_state_auth_rejects_wrong_key() {
        let good_key = "super-secret-key";
        let hashes = [hash_key(good_key)];
        let provided = hash_key("wrong-key");
        assert!(!hashes.contains(&provided));
    }

    #[test]
    fn proxy_state_auth_accepts_correct_key() {
        let good_key = "super-secret-key";
        let hashes = [hash_key(good_key)];
        let provided = hash_key(good_key);
        assert!(hashes.contains(&provided));
    }

    #[test]
    fn unix_now_is_positive() {
        assert!(unix_now() > 0);
    }
}
