//! Rustassistant Server
//!
//! A clean REST API for notes, repositories, and tasks.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

// Import from our crate
use rustassistant::db::{
    self, create_note, get_next_task, get_stats, list_notes, list_repositories, list_tasks,
    search_notes, update_note_status, update_task_status,
};

// ============================================================================
// Application State
// ============================================================================

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct CreateNoteRequest {
    content: String,
    tags: Option<String>,
    project: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListNotesQuery {
    limit: Option<i64>,
    status: Option<String>,
    project: Option<String>,
    tag: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct UpdateStatusRequest {
    status: String,
}

#[derive(Debug, Deserialize)]
struct AddRepoRequest {
    path: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListTasksQuery {
    limit: Option<i64>,
    status: Option<String>,
    priority: Option<i32>,
    repo_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Json<Self> {
        Json(Self {
            success: true,
            data: Some(data),
            error: None,
        })
    }
}

impl ApiResponse<()> {
    fn error(msg: impl Into<String>) -> (StatusCode, Json<Self>) {
        (
            StatusCode::BAD_REQUEST,
            Json(Self {
                success: false,
                data: None,
                error: Some(msg.into()),
            }),
        )
    }

    fn not_found(msg: impl Into<String>) -> (StatusCode, Json<Self>) {
        (
            StatusCode::NOT_FOUND,
            Json(Self {
                success: false,
                data: None,
                error: Some(msg.into()),
            }),
        )
    }
}

// ============================================================================
// Handlers
// ============================================================================

// Health check
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "rustassistant",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Stats
async fn get_statistics(State(state): State<AppState>) -> impl IntoResponse {
    match get_stats(&state.db).await {
        Ok(stats) => ApiResponse::ok(stats).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

// --- Notes ---

async fn create_note_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateNoteRequest>,
) -> impl IntoResponse {
    match create_note(
        &state.db,
        &req.content,
        req.tags.as_deref(),
        req.project.as_deref(),
    )
    .await
    {
        Ok(note) => (StatusCode::CREATED, ApiResponse::ok(note)).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

async fn list_notes_handler(
    State(state): State<AppState>,
    Query(query): Query<ListNotesQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    match list_notes(
        &state.db,
        limit,
        query.status.as_deref(),
        query.project.as_deref(),
        query.tag.as_deref(),
    )
    .await
    {
        Ok(notes) => ApiResponse::ok(notes).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

async fn search_notes_handler(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20);
    match search_notes(&state.db, &query.q, limit).await {
        Ok(notes) => ApiResponse::ok(notes).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

async fn get_note_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match db::get_note(&state.db, &id).await {
        Ok(note) => (StatusCode::OK, ApiResponse::ok(note)).into_response(),
        Err(db::DbError::NotFound(msg)) => ApiResponse::not_found(msg).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

async fn update_note_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    match update_note_status(&state.db, &id, &req.status).await {
        Ok(()) => ApiResponse::ok(serde_json::json!({"updated": true})).into_response(),
        Err(db::DbError::NotFound(msg)) => ApiResponse::not_found(msg).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

async fn delete_note_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match db::delete_note(&state.db, &id).await {
        Ok(()) => (
            StatusCode::OK,
            ApiResponse::ok(serde_json::json!({"deleted": true})),
        )
            .into_response(),
        Err(db::DbError::NotFound(msg)) => ApiResponse::not_found(msg).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

// --- Repositories ---

async fn add_repo_handler(
    State(state): State<AppState>,
    Json(req): Json<AddRepoRequest>,
) -> impl IntoResponse {
    // Derive name from path if not provided
    let name = req.name.unwrap_or_else(|| {
        std::path::Path::new(&req.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string()
    });

    match db::add_repository(&state.db, &req.path, &name).await {
        Ok(repo) => (StatusCode::CREATED, ApiResponse::ok(repo)).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

async fn list_repos_handler(State(state): State<AppState>) -> impl IntoResponse {
    match list_repositories(&state.db).await {
        Ok(repos) => ApiResponse::ok(repos).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

async fn get_repo_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match db::get_repository(&state.db, &id).await {
        Ok(repo) => (StatusCode::OK, ApiResponse::ok(repo)).into_response(),
        Err(db::DbError::NotFound(msg)) => ApiResponse::not_found(msg).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

async fn delete_repo_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match db::remove_repository(&state.db, &id).await {
        Ok(()) => (
            StatusCode::OK,
            ApiResponse::ok(serde_json::json!({"deleted": true})),
        )
            .into_response(),
        Err(db::DbError::NotFound(msg)) => ApiResponse::not_found(msg).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

// --- Tasks ---

async fn list_tasks_handler(
    State(state): State<AppState>,
    Query(query): Query<ListTasksQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    match list_tasks(
        &state.db,
        limit,
        query.status.as_deref(),
        query.priority,
        query.repo_id.as_deref(),
    )
    .await
    {
        Ok(tasks) => ApiResponse::ok(tasks).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

async fn get_next_task_handler(State(state): State<AppState>) -> impl IntoResponse {
    match get_next_task(&state.db).await {
        Ok(Some(task)) => ApiResponse::ok(task).into_response(),
        Ok(None) => {
            ApiResponse::ok(serde_json::json!({"message": "No pending tasks"})).into_response()
        }
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

async fn update_task_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    match update_task_status(&state.db, &id, &req.status).await {
        Ok(()) => ApiResponse::ok(serde_json::json!({"updated": true})).into_response(),
        Err(db::DbError::NotFound(msg)) => ApiResponse::not_found(msg).into_response(),
        Err(e) => ApiResponse::error(e.to_string()).into_response(),
    }
}

// ============================================================================
// Router
// ============================================================================

fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health & Stats
        .route("/health", get(health_check))
        .route("/api/stats", get(get_statistics))
        // Notes
        .route("/api/notes", post(create_note_handler))
        .route("/api/notes", get(list_notes_handler))
        .route("/api/notes/search", get(search_notes_handler))
        .route("/api/notes/:id", get(get_note_handler))
        .route("/api/notes/:id", put(update_note_handler))
        .route("/api/notes/:id", delete(delete_note_handler))
        // Repositories
        .route("/api/repos", post(add_repo_handler))
        .route("/api/repos", get(list_repos_handler))
        .route("/api/repos/:id", get(get_repo_handler))
        .route("/api/repos/:id", delete(delete_repo_handler))
        // Tasks
        .route("/api/tasks", get(list_tasks_handler))
        .route("/api/tasks/next", get(get_next_task_handler))
        .route("/api/tasks/:id", put(update_task_handler))
        .layer(cors)
        .with_state(state)
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,rustassistant=debug".into()),
        )
        .init();

    // Get configuration
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/rustassistant.db".into());
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("{}:{}", host, port);

    // Initialize database
    info!("Initializing database at {}", database_url);
    let db = db::init_db(&database_url).await?;

    // Create app state
    let state = AppState { db };

    // Build router
    let app = create_router(state);

    // Start server
    info!("🚀 Rustassistant server starting on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
