// ============================================================================
// Cache & Scan Viewer — Web UI Module
// ============================================================================
//
// Provides browsing of scan results, file analysis scores, directory coverage,
// and identification of unanalyzed files.
//
// Routes:
//   GET /cache                    — Overview: stats, cost, coverage per repo
//   GET /cache/:repo_id           — Repo detail: analyzed files + results
//   GET /cache/:repo_id/file      — Single file analysis result (?path=...)
//   GET /cache/:repo_id/gaps      — Unanalyzed files for a repo
//
// Integration:
//   In server.rs, merge with existing router:
//     .merge(web_ui_cache_viewer::create_cache_viewer_router(state))

use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, warn};

use crate::repo_cache_sql::RepoCacheSql;
use crate::web_ui::WebAppState;

// ============================================================================
// Router
// ============================================================================

pub fn create_cache_viewer_router(state: Arc<WebAppState>) -> Router {
    Router::new()
        .route("/cache", get(cache_overview_handler))
        .route("/cache/:repo_id", get(cache_repo_detail_handler))
        .route("/cache/:repo_id/file", get(cache_file_detail_handler))
        .route("/cache/:repo_id/gaps", get(cache_gaps_handler))
        .with_state(state)
}

// ============================================================================
// Shared Helpers (same pattern as web_ui_extensions.rs)
// ============================================================================

fn format_timestamp(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "—".to_string())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn ts(utc_str: &str) -> String {
    format!(r#"<span data-utc="{u}">{u}</span>"#, u = utc_str)
}

fn nav(active: &str) -> String {
    let items = [
        ("Dashboard", "/dashboard"),
        ("Repos", "/repos"),
        ("Cache Viewer", "/cache"),
        ("Queue", "/queue"),
        ("Ideas", "/ideas"),
        ("Docs", "/docs"),
        ("Activity", "/activity"),
    ];
    let links: String = items
        .iter()
        .map(|(label, href)| {
            let class = if *label == active {
                " class=\"active\""
            } else {
                ""
            };
            format!(r#"<a href="{href}"{class}>{label}</a>"#)
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"{links}
        {tz}"#,
        links = links,
        tz = crate::web_ui::timezone_selector_html()
    )
}

fn timezone_js() -> &'static str {
    r#"<script>
    function convertTimestamps() {
        const tz = localStorage.getItem('rustassistant_tz') || Intl.DateTimeFormat().resolvedOptions().timeZone;
        document.querySelectorAll('[data-utc]').forEach(el => {
            const utc = el.getAttribute('data-utc');
            if (utc && utc !== '—') {
                try {
                    const d = new Date(utc + 'Z');
                    el.textContent = d.toLocaleString('en-US', {timeZone: tz, month:'short', day:'numeric', hour:'2-digit', minute:'2-digit'});
                } catch(e) {}
            }
        });
        const sel = document.getElementById('tz-select');
        if (sel) {
            sel.value = tz;
            sel.addEventListener('change', e => { localStorage.setItem('rustassistant_tz', e.target.value); convertTimestamps(); });
        }
    }
    document.addEventListener('DOMContentLoaded', convertTimestamps);
    </script>"#
}

fn page_style() -> &'static str {
    r#"<style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0f172a; color: #e2e8f0; line-height: 1.6; }
        .container { max-width: 1400px; margin: 0 auto; padding: 1rem 2rem; }
        header { display: flex; justify-content: space-between; align-items: center;
            padding: 1rem 0; border-bottom: 1px solid #1e293b; margin-bottom: 1.5rem; }
        header h1 { font-size: 1.3rem; color: #0ea5e9; }
        nav { display: flex; gap: 0.5rem; flex-wrap: wrap; align-items: center; }
        nav a { color: #94a3b8; text-decoration: none; padding: 0.4rem 0.8rem;
            border-radius: 6px; font-size: 0.9rem; }
        nav a:hover { color: #e2e8f0; background: #1e293b; }
        nav a.active { color: #0ea5e9; background: #0c2d4a; font-weight: 600; }
        h2 { font-size: 1.4rem; margin-bottom: 1rem; color: #f1f5f9; }
        h3 { font-size: 1.1rem; margin-bottom: 0.5rem; color: #cbd5e1; }
        .card { background: #1e293b; border-radius: 8px; border: 1px solid #334155;
            padding: 1.2rem; margin-bottom: 1rem; }
        .stat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
            gap: 1rem; margin-bottom: 1.5rem; }
        .stat-card { background: #1e293b; border-radius: 8px; border: 1px solid #334155;
            padding: 1rem; text-align: center; }
        .stat-value { font-size: 1.8rem; font-weight: 700; color: #f1f5f9; }
        .stat-label { font-size: 0.85rem; color: #94a3b8; margin-top: 0.2rem; }
        .stat-value.green { color: #22c55e; }
        .stat-value.blue { color: #38bdf8; }
        .stat-value.orange { color: #f59e0b; }
        .stat-value.red { color: #ef4444; }
        .btn, .btn-small { padding: 0.5rem 1rem; border-radius: 6px; border: none; cursor: pointer;
            font-size: 0.9rem; font-weight: 500; text-decoration: none; display: inline-block; }
        .btn-small { padding: 0.25rem 0.6rem; font-size: 0.8rem; }
        .btn-primary { background: #0ea5e9; color: white; }
        .btn-primary:hover { background: #0284c7; }
        table { width: 100%; border-collapse: collapse; }
        th, td { text-align: left; padding: 0.6rem 0.8rem; border-bottom: 1px solid #334155; }
        th { background: #0f172a; color: #94a3b8; font-weight: 600; font-size: 0.85rem;
            text-transform: uppercase; letter-spacing: 0.03em; position: sticky; top: 0; }
        tr:hover { background: #1a2744; }
        .mono { font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 0.85rem; }
        .badge { display: inline-block; padding: 0.15rem 0.5rem; border-radius: 4px;
            font-size: 0.75rem; font-weight: 600; }
        .badge-green { background: #064e3b; color: #6ee7b7; }
        .badge-yellow { background: #713f12; color: #fde68a; }
        .badge-red { background: #7f1d1d; color: #fca5a5; }
        .badge-blue { background: #1e3a5f; color: #93c5fd; }
        .badge-gray { background: #374151; color: #9ca3af; }
        .progress-bar { background: #334155; border-radius: 4px; height: 8px; overflow: hidden; }
        .progress-fill { height: 100%; border-radius: 4px; transition: width 0.3s; }
        .dir-tree { font-family: monospace; font-size: 0.85rem; }
        .dir-tree .dir { color: #38bdf8; cursor: pointer; }
        .dir-tree .file-analyzed { color: #22c55e; }
        .dir-tree .file-pending { color: #94a3b8; }
        .dir-tree .file-skipped { color: #64748b; text-decoration: line-through; }
        .empty-state { text-align: center; padding: 3rem; color: #64748b; }
        a { color: #38bdf8; text-decoration: none; }
        a:hover { text-decoration: underline; }
        .file-result { background: #0f172a; border-radius: 6px; padding: 1rem;
            margin-top: 0.5rem; border: 1px solid #334155; }
        .file-result pre { white-space: pre-wrap; word-break: break-word;
            font-size: 0.8rem; color: #cbd5e1; max-height: 400px; overflow-y: auto; }
        .score-pill { display: inline-block; padding: 0.2rem 0.6rem; border-radius: 12px;
            font-weight: 700; font-size: 0.85rem; }
        .score-high { background: #064e3b; color: #6ee7b7; }
        .score-mid { background: #713f12; color: #fde68a; }
        .score-low { background: #7f1d1d; color: #fca5a5; }
        .search-box { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
        .search-box input { flex: 1; background: #334155; color: #e2e8f0;
            border: 1px solid #475569; border-radius: 6px; padding: 0.5rem 0.8rem;
            font-size: 0.9rem; }
        .search-box input:focus { outline: none; border-color: #0ea5e9; }
        .filter-bar { display: flex; gap: 0.5rem; margin-bottom: 1rem; flex-wrap: wrap; }
        .filter-bar select { background: #334155; color: #e2e8f0; border: 1px solid #475569;
            border-radius: 6px; padding: 0.4rem 0.6rem; font-size: 0.85rem; }
        .breadcrumb { font-size: 0.9rem; margin-bottom: 1rem; color: #94a3b8; }
        .breadcrumb a { color: #38bdf8; }
        .breadcrumb .sep { margin: 0 0.4rem; color: #475569; }
    </style>"#
}

// ============================================================================
// Helpers: cache DB access
// ============================================================================

/// Row from cache_entries
struct CacheRow {
    file_path: String,
    cache_type: String,
    tokens_used: Option<i64>,
    file_size: i64,
    created_at: String,
    last_accessed: String,
    access_count: i64,
    provider: String,
    model: String,
    result_blob: Vec<u8>,
}

/// Aggregated stats for a repo's cache
struct CacheOverviewStats {
    total_entries: i64,
    total_tokens: i64,
    estimated_cost: f64,
    unique_files: i64,
    total_file_size: i64,
    cache_hits: i64,
    cache_misses: i64,
}

/// Open the per-repo cache DB. Returns None if not found.
async fn open_repo_cache(repo_path: &str) -> Option<RepoCacheSql> {
    match RepoCacheSql::new_for_repo(std::path::Path::new(repo_path)).await {
        Ok(cache) => Some(cache),
        Err(e) => {
            warn!("Could not open cache for {}: {}", repo_path, e);
            None
        }
    }
}

/// Get all cache entries from a repo's cache DB
async fn get_cache_entries(cache: &RepoCacheSql) -> Vec<CacheRow> {
    let rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<i64>,
            i64,
            String,
            String,
            i64,
            String,
            String,
            Vec<u8>,
        ),
    >(
        r#"SELECT file_path, cache_type, tokens_used, file_size, created_at, last_accessed,
                  access_count, provider, model, result_blob
           FROM cache_entries
           ORDER BY file_path ASC"#,
    )
    .fetch_all(&cache.pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|r| CacheRow {
            file_path: r.0,
            cache_type: r.1,
            tokens_used: r.2,
            file_size: r.3,
            created_at: r.4,
            last_accessed: r.5,
            access_count: r.6,
            provider: r.7,
            model: r.8,
            result_blob: r.9,
        })
        .collect()
}

/// Get overview stats from cache DB
async fn get_cache_overview(cache: &RepoCacheSql) -> CacheOverviewStats {
    let (total_entries, total_tokens, total_file_size) =
        sqlx::query_as::<_, (i64, Option<i64>, i64)>(
            "SELECT COUNT(*), SUM(tokens_used), SUM(file_size) FROM cache_entries",
        )
        .fetch_one(&cache.pool)
        .await
        .unwrap_or((0, None, 0));

    let unique_files =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT file_path) FROM cache_entries")
            .fetch_one(&cache.pool)
            .await
            .unwrap_or(0);

    let (cache_hits, cache_misses) = sqlx::query_as::<_, (i64, i64)>(
        "SELECT cache_hits, cache_misses FROM cache_stats WHERE id = 1",
    )
    .fetch_one(&cache.pool)
    .await
    .unwrap_or((0, 0));

    let total_tokens = total_tokens.unwrap_or(0);
    // Grok 4.1 Fast pricing: $0.20/1M input, $0.50/1M output
    // Blended rate with ~70% input / 30% output split: $0.29/1M
    let estimated_cost = total_tokens as f64 * 0.29 / 1_000_000.0;

    CacheOverviewStats {
        total_entries,
        total_tokens,
        estimated_cost,
        unique_files,
        total_file_size,
        cache_hits,
        cache_misses,
    }
}

/// Decompress a result blob and extract the score (if present)
fn extract_score_from_blob(blob: &[u8]) -> Option<i64> {
    let json_str = zstd::decode_all(blob).ok()?;
    let text = String::from_utf8(json_str).ok()?;
    let val: serde_json::Value = serde_json::from_str(&text).ok()?;
    // Try common paths for scores
    val.get("score")
        .or_else(|| val.get("quality_score"))
        .or_else(|| val.get("overall_score"))
        .and_then(|v| v.as_i64())
}

/// Decompress result blob to pretty JSON string
fn decompress_result(blob: &[u8]) -> String {
    match zstd::decode_all(blob) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(val) => serde_json::to_string_pretty(&val).unwrap_or(text),
                Err(_) => text,
            },
            Err(_) => "(binary data)".to_string(),
        },
        Err(_) => "(decompression error)".to_string(),
    }
}

fn score_class(score: i64) -> &'static str {
    if score >= 80 {
        "score-high"
    } else if score >= 60 {
        "score-mid"
    } else {
        "score-low"
    }
}

fn format_tokens(tokens: i64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

fn format_bytes(bytes: i64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

// ============================================================================
// Handler: Cache Overview (all repos)
// ============================================================================

pub async fn cache_overview_handler(State(state): State<Arc<WebAppState>>) -> impl IntoResponse {
    let pool = &state.db.pool;

    // Get all repositories
    let repos: Vec<(String, String, String, Option<String>)> =
        sqlx::query_as("SELECT id, name, path, git_url FROM repositories ORDER BY name")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    let mut repo_cards = String::new();
    let mut grand_total_entries = 0i64;
    let mut grand_total_tokens = 0i64;
    let mut grand_total_cost = 0.0f64;

    for (repo_id, repo_name, repo_path, _git_url) in &repos {
        if let Some(cache) = open_repo_cache(repo_path).await {
            let stats = get_cache_overview(&cache).await;
            grand_total_entries += stats.total_entries;
            grand_total_tokens += stats.total_tokens;
            grand_total_cost += stats.estimated_cost;

            let hit_rate = if stats.cache_hits + stats.cache_misses > 0 {
                (stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64) * 100.0
            } else {
                0.0
            };

            let coverage_pct = if stats.total_entries > 0 {
                "partial"
            } else {
                "none"
            };

            repo_cards.push_str(&format!(
                r#"
                <div class="card">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.8rem;">
                        <h3><a href="/cache/{repo_id}">{repo_name}</a></h3>
                        <div style="display: flex; gap: 0.5rem;">
                            <a href="/cache/{repo_id}" class="btn-small btn-primary">📊 Browse Files</a>
                            <a href="/cache/{repo_id}/gaps" class="btn-small" style="background: #475569; color: #e2e8f0;">🔍 Find Gaps</a>
                        </div>
                    </div>
                    <div style="display: grid; grid-template-columns: repeat(5, 1fr); gap: 1rem; text-align: center;">
                        <div>
                            <div style="font-size: 1.3rem; font-weight: 700; color: #38bdf8;">{files}</div>
                            <div style="font-size: 0.8rem; color: #94a3b8;">Files Analyzed</div>
                        </div>
                        <div>
                            <div style="font-size: 1.3rem; font-weight: 700; color: #f1f5f9;">{tokens}</div>
                            <div style="font-size: 0.8rem; color: #94a3b8;">Tokens Used</div>
                        </div>
                        <div>
                            <div style="font-size: 1.3rem; font-weight: 700; color: #f59e0b;">${cost:.4}</div>
                            <div style="font-size: 0.8rem; color: #94a3b8;">Est. Cost</div>
                        </div>
                        <div>
                            <div style="font-size: 1.3rem; font-weight: 700; color: #22c55e;">{hit_rate:.0}%</div>
                            <div style="font-size: 0.8rem; color: #94a3b8;">Cache Hit Rate</div>
                        </div>
                        <div>
                            <div style="font-size: 1.3rem; font-weight: 700; color: #cbd5e1;">{file_size}</div>
                            <div style="font-size: 0.8rem; color: #94a3b8;">Source Size</div>
                        </div>
                    </div>
                </div>"#,
                repo_id = repo_id,
                repo_name = html_escape(repo_name),
                files = stats.unique_files,
                tokens = format_tokens(stats.total_tokens),
                cost = stats.estimated_cost,
                hit_rate = hit_rate,
                file_size = format_bytes(stats.total_file_size),
            ));
        } else {
            repo_cards.push_str(&format!(
                r#"
                <div class="card">
                    <div style="display: flex; justify-content: space-between; align-items: center;">
                        <h3 style="color: #64748b;">{repo_name}</h3>
                        <span class="badge badge-gray">No cache</span>
                    </div>
                    <p style="color: #64748b; font-size: 0.9rem; margin-top: 0.5rem;">
                        No analysis cache found. Run a scan or force-scan to build the cache.
                    </p>
                </div>"#,
                repo_name = html_escape(repo_name),
            ));
        }
    }

    if repos.is_empty() {
        repo_cards = r#"<div class="empty-state"><p>No repositories tracked yet. Add one from the Repos page.</p></div>"#.to_string();
    }

    Html(format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Cache Viewer — Rustassistant</title>
        {style}
        </head><body>
        <div class="container">
            <header>
                <h1>🔬 Rustassistant</h1>
                <nav>{nav}</nav>
            </header>

            <h2>📦 Cache Viewer</h2>

            <div class="stat-grid">
                <div class="stat-card">
                    <div class="stat-value blue">{repos}</div>
                    <div class="stat-label">Repositories</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value green">{total_files}</div>
                    <div class="stat-label">Files Analyzed</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">{total_tokens}</div>
                    <div class="stat-label">Total Tokens</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value orange">${total_cost:.4}</div>
                    <div class="stat-label">Total Cost</div>
                </div>
            </div>

            {repo_cards}
        </div>
        {tz_js}
        </body></html>"#,
        style = page_style(),
        nav = nav("Cache Viewer"),
        repos = repos.len(),
        total_files = grand_total_entries,
        total_tokens = format_tokens(grand_total_tokens),
        total_cost = grand_total_cost,
        repo_cards = repo_cards,
        tz_js = timezone_js(),
    ))
}

// ============================================================================
// Handler: Repo Detail (file list with scores)
// ============================================================================

#[derive(Deserialize)]
pub struct FileListQuery {
    pub sort: Option<String>,   // "name", "score", "tokens", "date"
    pub filter: Option<String>, // substring filter on file path
    pub dir: Option<String>,    // filter to specific directory
}

pub async fn cache_repo_detail_handler(
    State(state): State<Arc<WebAppState>>,
    Path(repo_id): Path<String>,
    Query(query): Query<FileListQuery>,
) -> impl IntoResponse {
    let pool = &state.db.pool;

    // Get repo info
    let repo: Option<(String, String)> =
        sqlx::query_as("SELECT name, path FROM repositories WHERE id = ?")
            .bind(&repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    let (repo_name, repo_path) = match repo {
        Some(r) => r,
        None => return Html("<h1>Repository not found</h1>".to_string()),
    };

    let cache = match open_repo_cache(&repo_path).await {
        Some(c) => c,
        None => {
            return Html(format!(
                r#"<!DOCTYPE html><html><head>{style}</head><body>
            <div class="container">
                <header><h1>🔬 Rustassistant</h1><nav>{nav}</nav></header>
                <div class="breadcrumb"><a href="/cache">Cache</a><span class="sep">›</span>{name}</div>
                <div class="empty-state"><p>No cache database found for this repository. Run a scan first.</p></div>
            </div></body></html>"#,
                style = page_style(),
                nav = nav("Cache Viewer"),
                name = html_escape(&repo_name)
            ))
        }
    };

    let entries = get_cache_entries(&cache).await;
    let stats = get_cache_overview(&cache).await;

    // Build file rows with scores
    struct FileRow {
        path: String,
        score: Option<i64>,
        tokens: i64,
        file_size: i64,
        created: String,
        cache_type: String,
    }

    let mut files: Vec<FileRow> = entries
        .iter()
        .map(|e| {
            let score = extract_score_from_blob(&e.result_blob);
            FileRow {
                path: e.file_path.clone(),
                score,
                tokens: e.tokens_used.unwrap_or(0),
                file_size: e.file_size,
                created: e.created_at.clone(),
                cache_type: e.cache_type.clone(),
            }
        })
        .collect();

    // Apply filters
    if let Some(ref filter) = query.filter {
        let f = filter.to_lowercase();
        files.retain(|row| row.path.to_lowercase().contains(&f));
    }
    if let Some(ref dir) = query.dir {
        files.retain(|row| row.path.starts_with(dir.as_str()));
    }

    // Sort
    match query.sort.as_deref() {
        Some("score") => files.sort_by(|a, b| b.score.cmp(&a.score)),
        Some("tokens") => files.sort_by(|a, b| b.tokens.cmp(&a.tokens)),
        Some("date") => files.sort_by(|a, b| b.created.cmp(&a.created)),
        Some("size") => files.sort_by(|a, b| b.file_size.cmp(&a.file_size)),
        _ => files.sort_by(|a, b| a.path.cmp(&b.path)),
    }

    // Build directory summary
    let mut dir_counts: HashMap<String, (usize, usize)> = HashMap::new(); // (count, total_tokens)
    for f in &files {
        if let Some(dir) = f.path.rsplit_once('/').map(|(d, _)| d.to_string()) {
            let entry = dir_counts.entry(dir).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += f.tokens as usize;
        }
    }
    let mut dirs: Vec<(String, usize, usize)> = dir_counts
        .into_iter()
        .map(|(d, (c, t))| (d, c, t))
        .collect();
    dirs.sort_by(|a, b| b.1.cmp(&a.1));

    let dir_filter_html: String = dirs
        .iter()
        .take(20)
        .map(|(d, count, _)| {
            format!(
                r#"<a href="/cache/{repo_id}?dir={dir}" class="btn-small" style="background: #334155; color: #e2e8f0;">{dir_short} ({count})</a>"#,
                repo_id = repo_id,
                dir = html_escape(d),
                dir_short = d.rsplit('/').next().unwrap_or(d),
                count = count,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Build table rows
    let file_rows: String = files
        .iter()
        .map(|f| {
            let score_html = match f.score {
                Some(s) => format!(
                    r#"<span class="score-pill {cls}">{s}</span>"#,
                    cls = score_class(s),
                    s = s
                ),
                None => r#"<span style="color: #64748b;">—</span>"#.to_string(),
            };
            let tokens_str = format_tokens(f.tokens);
            let size_str = format_bytes(f.file_size);

            format!(
                r#"<tr>
                    <td class="mono"><a href="/cache/{repo_id}/file?path={path}">{path}</a></td>
                    <td style="text-align: center;">{score}</td>
                    <td style="text-align: right;" class="mono">{tokens}</td>
                    <td style="text-align: right;" class="mono">{size}</td>
                    <td><span class="badge badge-blue">{ctype}</span></td>
                    <td>{created}</td>
                </tr>"#,
                repo_id = repo_id,
                path = html_escape(&f.path),
                score = score_html,
                tokens = tokens_str,
                size = size_str,
                ctype = f.cache_type,
                created = ts(&f.created),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let active_filter = query.filter.as_deref().unwrap_or("");
    let active_dir = query.dir.as_deref().unwrap_or("");

    Html(format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{repo_name} Cache — Rustassistant</title>
        {style}
        </head><body>
        <div class="container">
            <header><h1>🔬 Rustassistant</h1><nav>{nav}</nav></header>

            <div class="breadcrumb">
                <a href="/cache">Cache</a><span class="sep">›</span>
                <strong>{repo_name}</strong>
            </div>

            <div class="stat-grid">
                <div class="stat-card">
                    <div class="stat-value green">{files_count}</div>
                    <div class="stat-label">Files Analyzed</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">{total_tokens}</div>
                    <div class="stat-label">Tokens Used</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value orange">${cost:.4}</div>
                    <div class="stat-label">Est. Cost</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value blue">{showing}</div>
                    <div class="stat-label">Showing</div>
                </div>
            </div>

            <!-- Directory quick-filter -->
            <div class="card" style="padding: 0.8rem;">
                <div style="display: flex; gap: 0.5rem; flex-wrap: wrap; align-items: center;">
                    <span style="color: #94a3b8; font-size: 0.85rem;">📁 Directories:</span>
                    <a href="/cache/{repo_id}" class="btn-small btn-primary">All</a>
                    {dir_filters}
                </div>
            </div>

            <!-- Search -->
            <form method="get" action="/cache/{repo_id}" class="search-box">
                <input type="text" name="filter" placeholder="Filter by file path..." value="{active_filter}">
                <input type="hidden" name="dir" value="{active_dir}">
                <button type="submit" class="btn btn-primary">Search</button>
            </form>

            <!-- Sort controls -->
            <div class="filter-bar">
                <span style="color: #94a3b8; font-size: 0.85rem;">Sort:</span>
                <a href="/cache/{repo_id}?sort=name&filter={af}&dir={ad}" class="btn-small" style="background: #334155; color: #e2e8f0;">Name</a>
                <a href="/cache/{repo_id}?sort=score&filter={af}&dir={ad}" class="btn-small" style="background: #334155; color: #e2e8f0;">Score</a>
                <a href="/cache/{repo_id}?sort=tokens&filter={af}&dir={ad}" class="btn-small" style="background: #334155; color: #e2e8f0;">Tokens</a>
                <a href="/cache/{repo_id}?sort=size&filter={af}&dir={ad}" class="btn-small" style="background: #334155; color: #e2e8f0;">Size</a>
                <a href="/cache/{repo_id}?sort=date&filter={af}&dir={ad}" class="btn-small" style="background: #334155; color: #e2e8f0;">Date</a>
            </div>

            <!-- File table -->
            <div class="card" style="padding: 0; overflow-x: auto;">
                <table>
                    <thead>
                        <tr>
                            <th>File Path</th>
                            <th style="text-align: center;">Score</th>
                            <th style="text-align: right;">Tokens</th>
                            <th style="text-align: right;">Size</th>
                            <th>Type</th>
                            <th>Analyzed</th>
                        </tr>
                    </thead>
                    <tbody>
                        {file_rows}
                    </tbody>
                </table>
            </div>

            <div style="margin-top: 1rem; text-align: center;">
                <a href="/cache/{repo_id}/gaps" class="btn btn-primary">🔍 View Unanalyzed Files</a>
            </div>
        </div>
        {tz_js}
        </body></html>"#,
        repo_name = html_escape(&repo_name),
        style = page_style(),
        nav = nav("Cache Viewer"),
        files_count = stats.unique_files,
        total_tokens = format_tokens(stats.total_tokens),
        cost = stats.estimated_cost,
        showing = files.len(),
        dir_filters = dir_filter_html,
        repo_id = repo_id,
        active_filter = html_escape(active_filter),
        active_dir = html_escape(active_dir),
        af = html_escape(active_filter),
        ad = html_escape(active_dir),
        file_rows = file_rows,
        tz_js = timezone_js(),
    ))
}

// ============================================================================
// Handler: Single File Detail
// ============================================================================

#[derive(Deserialize)]
pub struct FileDetailQuery {
    pub path: String,
}

pub async fn cache_file_detail_handler(
    State(state): State<Arc<WebAppState>>,
    Path(repo_id): Path<String>,
    Query(query): Query<FileDetailQuery>,
) -> impl IntoResponse {
    let pool = &state.db.pool;

    let repo: Option<(String, String)> =
        sqlx::query_as("SELECT name, path FROM repositories WHERE id = ?")
            .bind(&repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    let (repo_name, repo_path) = match repo {
        Some(r) => r,
        None => return Html("<h1>Repository not found</h1>".to_string()),
    };

    let cache = match open_repo_cache(&repo_path).await {
        Some(c) => c,
        None => return Html("<h1>No cache found</h1>".to_string()),
    };

    // Fetch the specific file entry
    let entry: Option<(
        String,
        Option<i64>,
        i64,
        String,
        String,
        i64,
        String,
        String,
        Vec<u8>,
    )> = sqlx::query_as(
        r#"SELECT cache_type, tokens_used, file_size, created_at, last_accessed,
                      access_count, provider, model, result_blob
               FROM cache_entries
               WHERE file_path = ?
               ORDER BY created_at DESC
               LIMIT 1"#,
    )
    .bind(&query.path)
    .fetch_optional(&cache.pool)
    .await
    .ok()
    .flatten();

    let (
        cache_type,
        tokens,
        file_size,
        created,
        last_accessed,
        access_count,
        provider,
        model,
        blob,
    ) = match entry {
        Some(e) => e,
        None => {
            return Html(format!(
                r#"<!DOCTYPE html><html><head>{style}</head><body>
                    <div class="container">
                        <header><h1>🔬 Rustassistant</h1><nav>{nav}</nav></header>
                        <div class="breadcrumb">
                            <a href="/cache">Cache</a><span class="sep">›</span>
                            <a href="/cache/{repo_id}">{repo_name}</a><span class="sep">›</span>
                            <span class="mono">{path}</span>
                        </div>
                        <div class="empty-state"><p>No cached analysis found for this file.</p></div>
                    </div></body></html>"#,
                style = page_style(),
                nav = nav("Cache Viewer"),
                repo_id = repo_id,
                repo_name = html_escape(&repo_name),
                path = html_escape(&query.path),
            ))
        }
    };

    let result_json = decompress_result(&blob);
    let score = extract_score_from_blob(&blob);
    let score_html = match score {
        Some(s) => format!(
            r#"<span class="score-pill {cls}" style="font-size: 1.5rem; padding: 0.4rem 1rem;">{s}/100</span>"#,
            cls = score_class(s),
            s = s
        ),
        None => r#"<span style="color: #64748b;">No score extracted</span>"#.to_string(),
    };

    Html(format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{file_path} — Rustassistant</title>
        {style}
        </head><body>
        <div class="container">
            <header><h1>🔬 Rustassistant</h1><nav>{nav}</nav></header>

            <div class="breadcrumb">
                <a href="/cache">Cache</a><span class="sep">›</span>
                <a href="/cache/{repo_id}">{repo_name}</a><span class="sep">›</span>
                <span class="mono">{file_path}</span>
            </div>

            <div class="card">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                    <h2 class="mono" style="font-size: 1.1rem; margin: 0;">{file_path}</h2>
                    {score_html}
                </div>

                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; margin-bottom: 1rem;">
                    <div>
                        <div style="color: #94a3b8; font-size: 0.8rem;">Type</div>
                        <div><span class="badge badge-blue">{cache_type}</span></div>
                    </div>
                    <div>
                        <div style="color: #94a3b8; font-size: 0.8rem;">Tokens</div>
                        <div class="mono">{tokens}</div>
                    </div>
                    <div>
                        <div style="color: #94a3b8; font-size: 0.8rem;">File Size</div>
                        <div class="mono">{file_size}</div>
                    </div>
                    <div>
                        <div style="color: #94a3b8; font-size: 0.8rem;">Provider / Model</div>
                        <div class="mono">{provider} / {model}</div>
                    </div>
                    <div>
                        <div style="color: #94a3b8; font-size: 0.8rem;">Analyzed</div>
                        <div>{created}</div>
                    </div>
                    <div>
                        <div style="color: #94a3b8; font-size: 0.8rem;">Last Accessed</div>
                        <div>{last_accessed}</div>
                    </div>
                    <div>
                        <div style="color: #94a3b8; font-size: 0.8rem;">Access Count</div>
                        <div class="mono">{access_count}</div>
                    </div>
                </div>
            </div>

            <h3 style="margin-top: 1.5rem;">📋 Analysis Result</h3>
            <div class="file-result">
                <pre>{result_escaped}</pre>
            </div>

            <div style="margin-top: 1rem;">
                <a href="/cache/{repo_id}" class="btn" style="background: #475569; color: #e2e8f0;">← Back to File List</a>
            </div>
        </div>
        {tz_js}
        </body></html>"#,
        file_path = html_escape(&query.path),
        style = page_style(),
        nav = nav("Cache Viewer"),
        repo_id = repo_id,
        repo_name = html_escape(&repo_name),
        score_html = score_html,
        cache_type = cache_type,
        tokens = tokens
            .map(|t| format_tokens(t))
            .unwrap_or_else(|| "—".to_string()),
        file_size = format_bytes(file_size),
        provider = html_escape(&provider),
        model = html_escape(&model),
        created = ts(&created),
        last_accessed = ts(&last_accessed),
        access_count = access_count,
        result_escaped = html_escape(&result_json),
        tz_js = timezone_js(),
    ))
}

// ============================================================================
// Handler: Gaps — unanalyzed files
// ============================================================================

pub async fn cache_gaps_handler(
    State(state): State<Arc<WebAppState>>,
    Path(repo_id): Path<String>,
) -> impl IntoResponse {
    let pool = &state.db.pool;

    let repo: Option<(String, String)> =
        sqlx::query_as("SELECT name, path FROM repositories WHERE id = ?")
            .bind(&repo_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    let (repo_name, repo_path) = match repo {
        Some(r) => r,
        None => return Html("<h1>Repository not found</h1>".to_string()),
    };

    // Get set of analyzed files from cache
    let cache = open_repo_cache(&repo_path).await;
    let analyzed_files: std::collections::HashSet<String> = if let Some(ref c) = cache {
        sqlx::query_scalar::<_, String>("SELECT DISTINCT file_path FROM cache_entries")
            .fetch_all(&c.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .collect()
    } else {
        std::collections::HashSet::new()
    };

    // Walk the repo directory for source files
    let analyzable_extensions = [
        "rs", "py", "js", "ts", "tsx", "sh", "kt", "java", "go", "rb",
    ];
    let skip_dirs = [
        "dist",
        "build",
        "node_modules",
        "target",
        ".git",
        "vendor",
        "__pycache__",
        ".next",
        "out",
        "coverage",
        ".cache",
    ];

    let mut all_source_files: Vec<String> = Vec::new();
    let repo_root = std::path::Path::new(&repo_path);

    if repo_root.exists() {
        fn walk_dir(
            root: &std::path::Path,
            base: &std::path::Path,
            files: &mut Vec<String>,
            extensions: &[&str],
            skip: &[&str],
        ) {
            if let Ok(entries) = std::fs::read_dir(root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();

                    if path.is_dir() {
                        if !skip.contains(&name.as_str()) && !name.starts_with('.') {
                            walk_dir(&path, base, files, extensions, skip);
                        }
                    } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if extensions.contains(&ext) {
                            if let Ok(rel) = path.strip_prefix(base) {
                                files.push(rel.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
        walk_dir(
            repo_root,
            repo_root,
            &mut all_source_files,
            &analyzable_extensions,
            &skip_dirs,
        );
    }

    all_source_files.sort();

    let total_source = all_source_files.len();
    let total_analyzed = analyzed_files.len();
    let total_pending = total_source.saturating_sub(total_analyzed);
    let coverage_pct = if total_source > 0 {
        (total_analyzed as f64 / total_source as f64) * 100.0
    } else {
        0.0
    };

    // Separate into analyzed and pending
    let pending_files: Vec<&String> = all_source_files
        .iter()
        .filter(|f| !analyzed_files.contains(f.as_str()))
        .collect();

    // Group pending by directory
    let mut pending_by_dir: HashMap<String, Vec<&str>> = HashMap::new();
    for f in &pending_files {
        let dir = f
            .rsplit_once('/')
            .map(|(d, _)| d.to_string())
            .unwrap_or_else(|| ".".to_string());
        pending_by_dir.entry(dir).or_default().push(f.as_str());
    }
    let mut pending_dirs: Vec<(String, Vec<&str>)> = pending_by_dir.into_iter().collect();
    pending_dirs.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let pending_html: String = if pending_dirs.is_empty() {
        r#"<div class="empty-state" style="padding: 2rem;"><p>🎉 All source files have been analyzed!</p></div>"#.to_string()
    } else {
        pending_dirs
            .iter()
            .map(|(dir, files)| {
                let file_list: String = files
                    .iter()
                    .map(|f| format!(r#"<div class="mono" style="padding: 0.2rem 0; color: #94a3b8; font-size: 0.85rem;">  {f}</div>"#))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    r#"<details style="margin-bottom: 0.5rem;">
                        <summary style="cursor: pointer; color: #38bdf8; font-weight: 600; padding: 0.3rem 0;">
                            📁 {dir}/ <span class="badge badge-yellow">{count} pending</span>
                        </summary>
                        <div style="padding-left: 1rem;">{file_list}</div>
                    </details>"#,
                    dir = html_escape(dir),
                    count = files.len(),
                    file_list = file_list,
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Coverage bar color
    let bar_color = if coverage_pct >= 80.0 {
        "#22c55e"
    } else if coverage_pct >= 50.0 {
        "#f59e0b"
    } else {
        "#ef4444"
    };

    Html(format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Gaps — {repo_name} — Rustassistant</title>
        {style}
        </head><body>
        <div class="container">
            <header><h1>🔬 Rustassistant</h1><nav>{nav}</nav></header>

            <div class="breadcrumb">
                <a href="/cache">Cache</a><span class="sep">›</span>
                <a href="/cache/{repo_id}">{repo_name}</a><span class="sep">›</span>
                <strong>Coverage Gaps</strong>
            </div>

            <div class="stat-grid">
                <div class="stat-card">
                    <div class="stat-value">{total_source}</div>
                    <div class="stat-label">Source Files</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value green">{total_analyzed}</div>
                    <div class="stat-label">Analyzed</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value orange">{total_pending}</div>
                    <div class="stat-label">Pending</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value" style="color: {bar_color};">{coverage:.1}%</div>
                    <div class="stat-label">Coverage</div>
                </div>
            </div>

            <!-- Coverage bar -->
            <div class="card" style="padding: 1rem;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 0.3rem;">
                    <span style="color: #94a3b8; font-size: 0.85rem;">Analysis Coverage</span>
                    <span style="color: #f1f5f9; font-weight: 600;">{coverage:.1}%</span>
                </div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: {coverage}%; background: {bar_color};"></div>
                </div>
            </div>

            <h3 style="margin-top: 1.5rem;">📋 Unanalyzed Files ({total_pending})</h3>
            <div class="card" style="max-height: 600px; overflow-y: auto;">
                {pending_html}
            </div>

            <div style="margin-top: 1rem;">
                <a href="/cache/{repo_id}" class="btn" style="background: #475569; color: #e2e8f0;">← Back to File List</a>
            </div>
        </div>
        {tz_js}
        </body></html>"#,
        repo_name = html_escape(&repo_name),
        style = page_style(),
        nav = nav("Cache Viewer"),
        repo_id = repo_id,
        total_source = total_source,
        total_analyzed = total_analyzed,
        total_pending = total_pending,
        coverage = coverage_pct,
        bar_color = bar_color,
        pending_html = pending_html,
        tz_js = timezone_js(),
    ))
}
