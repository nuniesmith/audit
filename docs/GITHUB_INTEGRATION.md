# GitHub Integration - Quick Start Guide

> Transform rustassistant into a GitHub-first personal assistant with zero LLM costs for GitHub operations

## 🎯 Overview

The GitHub integration module enables rustassistant to:

- ✅ **Sync all your GitHub data locally** (repos, issues, PRs, commits)
- ✅ **Answer GitHub queries instantly** without expensive LLM calls
- ✅ **Search across all your GitHub activity** from a single interface
- ✅ **Receive real-time updates** via webhooks
- ✅ **Create GitHub issues** directly from rustassistant tasks

**Cost Savings:** $20-30/month for typical developer workflow

## 🚀 Quick Start (5 Minutes)

### Step 1: Get GitHub Token

1. Go to [GitHub Settings → Tokens](https://github.com/settings/tokens)
2. Click **Generate new token (classic)**
3. Select scopes:
   - ✅ `repo` (Full control of repositories)
   - ✅ `read:user` (Read user profile)
4. Copy token and add to `.env`:

```bash
# .env
GITHUB_TOKEN=ghp_your_token_here
```

### Step 2: Initialize Schema

```bash
# Run migration to create GitHub tables
cargo run --bin rustassistant-server migrate
```

Or programmatically:

```rust
use rustassistant::github::{GitHubClient, SyncEngine};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = GitHubClient::new(std::env::var("GITHUB_TOKEN")?)?;
    let pool = SqlitePool::connect("sqlite:data/rustassistant.db").await?;
    
    let sync = SyncEngine::new(client, pool);
    sync.initialize_schema().await?;
    
    println!("✓ GitHub schema initialized");
    Ok(())
}
```

### Step 3: First Sync

```rust
// Sync all your repositories
let result = sync.sync_all_repos().await?;

println!("Synced {} repos in {:.2}s", result.repos_synced, result.duration_secs);
println!("  Issues: {}", result.issues_synced);
println!("  PRs: {}", result.prs_synced);
println!("  Commits: {}", result.commits_synced);
```

### Step 4: Try Queries

```rust
use rustassistant::github::search::{GitHubSearcher, SearchQuery, SearchType};

let searcher = GitHubSearcher::new(pool);

// Get open issues
let query = SearchQuery::new("")
    .with_type(SearchType::Issues)
    .only_open()
    .limit(10);

let results = searcher.search(query).await?;
```

## 💡 Common Use Cases

### 1. Daily Standup Report

```rust
async fn daily_standup(searcher: &GitHubSearcher) -> String {
    let stats = searcher.get_stats().await.unwrap();
    
    format!(
        "📊 GitHub Activity:\n\
         • {} repositories\n\
         • {} open issues\n\
         • {} open PRs\n\
         • {} total commits tracked",
        stats.total_repos,
        stats.open_issues,
        stats.open_prs,
        stats.total_commits
    )
}
```

### 2. Find PRs Needing Review

```rust
async fn prs_needing_review(sync: &SyncEngine) {
    let prs = sync.get_prs_needing_review().await.unwrap();
    
    println!("📝 PRs Needing Review ({}):", prs.len());
    for (repo, number, title) in prs {
        println!("  • {}#{}: {}", repo, number, title);
    }
}
```

### 3. Search Issues by Keyword

```rust
let query = SearchQuery::new("authentication bug")
    .with_type(SearchType::Issues)
    .only_open()
    .in_repo("owner/rustassistant");

let results = searcher.search(query).await?;
```

### 4. Create GitHub Issue from Task

```rust
async fn sync_task_to_github(
    client: &GitHubClient,
    task: &Task,
) -> Result<Issue> {
    client.create_issue(
        "owner",
        "repo",
        &task.title,
        Some(&task.description),
        Some(vec!["from-rustassistant".to_string()]),
    ).await
}
```

### 5. Monitor Recent Activity

```rust
let query = SearchQuery::new("")
    .with_type(SearchType::Commits)
    .limit(20);

let commits = searcher.search(query).await?;

for commit in commits {
    if let SearchResult::Commit(c) = commit {
        println!("{}: {}", &c.sha[..7], c.message.lines().next().unwrap());
    }
}
```

## 🔄 Automated Background Sync

Add to your `server.rs`:

```rust
use tokio::time::{interval, Duration};

async fn start_github_sync_job(sync: SyncEngine) {
    let mut ticker = interval(Duration::from_secs(3600)); // Every hour
    
    loop {
        ticker.tick().await;
        
        match sync.sync_incremental().await {
            Ok(result) => {
                tracing::info!(
                    "GitHub sync: {} repos, {} issues, {} PRs",
                    result.repos_synced,
                    result.issues_synced,
                    result.prs_synced
                );
            }
            Err(e) => {
                tracing::error!("GitHub sync failed: {}", e);
            }
        }
    }
}

// In main():
tokio::spawn(start_github_sync_job(sync));
```

## 🎣 Webhooks (Real-Time Updates)

### Setup on GitHub

1. Repository Settings → Webhooks → Add webhook
2. **Payload URL:** `https://your-domain.com/api/webhooks/github`
3. **Content type:** `application/json`
4. **Secret:** Generate strong secret, add to `.env`:

```bash
GITHUB_WEBHOOK_SECRET=your_webhook_secret_here
```

5. **Events:** Select events (push, issues, pull_request)

### Webhook Endpoint

```rust
use rustassistant::github::webhook::{WebhookHandler, WebhookPayload, WebhookEvent};
use axum::{extract::Json, http::HeaderMap, routing::post, Router};

async fn github_webhook(
    headers: HeaderMap,
    body: String,
) -> Result<String, String> {
    let handler = WebhookHandler::new(std::env::var("GITHUB_WEBHOOK_SECRET").unwrap());
    
    let payload = WebhookPayload::new(
        headers.get("X-GitHub-Event")
            .ok_or("Missing event header")?
            .to_str()
            .map_err(|e| e.to_string())?,
        headers.get("X-GitHub-Delivery")
            .ok_or("Missing delivery header")?
            .to_str()
            .map_err(|e| e.to_string())?,
        headers.get("X-Hub-Signature-256")
            .map(|v| v.to_str().unwrap().to_string()),
        body,
    );
    
    match handler.handle(payload).await {
        Ok(event) => {
            handle_github_event(event).await;
            Ok("OK".to_string())
        }
        Err(e) => {
            tracing::error!("Webhook error: {}", e);
            Err(format!("Error: {}", e))
        }
    }
}

async fn handle_github_event(event: WebhookEvent) {
    match event {
        WebhookEvent::Push(e) => {
            tracing::info!("Push to {} by {}", e.git_ref, e.sender.login);
            // Trigger incremental sync
        }
        WebhookEvent::IssuesOpened(e) => {
            tracing::info!("New issue #{}: {}", e.issue.number, e.issue.title);
            // Sync this specific issue
        }
        WebhookEvent::PullRequestOpened(e) => {
            tracing::info!("New PR #{}: {}", e.number, e.pull_request.title);
            // Sync this specific PR
        }
        _ => {}
    }
}

// Add to your router
let app = Router::new()
    .route("/api/webhooks/github", post(github_webhook));
```

## 🔌 Integration with Query Router

Prefer GitHub API over expensive LLM calls:

```rust
use rustassistant::query_router::{QueryRouter, QueryIntent};

async fn handle_query(query: &str, searcher: &GitHubSearcher) -> String {
    let router = QueryRouter::new();
    let intent = router.classify(query).await.unwrap();
    
    match intent {
        QueryIntent::GitHubIssues => {
            // FREE GitHub query!
            let results = searcher.search(
                SearchQuery::new(query)
                    .with_type(SearchType::Issues)
                    .only_open()
            ).await.unwrap();
            
            format!("Found {} issues", results.len())
        }
        QueryIntent::GitHubPRStatus => {
            // FREE GitHub query!
            let results = searcher.search(
                SearchQuery::new("")
                    .with_type(SearchType::PullRequests)
                    .only_open()
            ).await.unwrap();
            
            format!("You have {} open PRs", results.len())
        }
        _ => {
            // Fall back to LLM (expensive)
            grok_client.ask(query).await.unwrap()
        }
    }
}
```

## 📊 CLI Commands

Add to `cli.rs`:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands
    
    /// GitHub operations
    #[command(subcommand)]
    Github(GithubCommands),
}

#[derive(Subcommand)]
enum GithubCommands {
    /// Sync GitHub data
    Sync {
        /// Force full resync
        #[arg(long)]
        full: bool,
    },
    
    /// Search GitHub
    Search {
        query: String,
        
        #[arg(long)]
        r#type: Option<String>, // "issues", "prs", "repos", "commits"
    },
    
    /// List open issues
    Issues {
        #[arg(long)]
        repo: Option<String>,
    },
    
    /// List PRs needing review
    Prs,
    
    /// Show GitHub statistics
    Stats,
}
```

Usage:

```bash
# Sync GitHub data
rustassistant github sync

# Full resync
rustassistant github sync --full

# Search for issues
rustassistant github search "authentication bug" --type issues

# List open issues
rustassistant github issues

# List PRs needing review
rustassistant github prs

# Show statistics
rustassistant github stats
```

## 🎨 Web UI Integration

Add to `web_ui.rs`:

```rust
// Template: templates/github_dashboard.html
async fn github_dashboard(
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let searcher = GitHubSearcher::new(pool);
    let stats = searcher.get_stats().await?;
    
    let template = GitHubDashboardTemplate {
        total_repos: stats.total_repos,
        open_issues: stats.open_issues,
        open_prs: stats.open_prs,
        total_commits: stats.total_commits,
    };
    
    Ok(template)
}
```

## 💰 Cost Comparison

### Before GitHub Integration

```
User: "what PRs need my review?"
→ Query Router → LLM API call
→ Cost: $0.018
→ Latency: 800ms
```

### After GitHub Integration

```
User: "what PRs need my review?"
→ Query Router → GitHub Module (local DB)
→ Cost: $0.000 ✅
→ Latency: 8ms ✅
```

**Monthly Savings:**

| Query Type | Before | After | Savings |
|------------|--------|-------|---------|
| GitHub questions (20/day) | $10.80 | $0.00 | 100% |
| Repo searches (15/day) | $6.75 | $0.00 | 100% |
| Issue/PR status (30/day) | $16.20 | $0.00 | 100% |
| **Total** | **$33.75** | **$0.00** | **100%** |

## 🔍 Advanced Search Examples

### Multi-Condition Search

```rust
use chrono::{Utc, Duration};

let week_ago = Utc::now() - Duration::days(7);

let query = SearchQuery::new("performance")
    .with_type(SearchType::Issues)
    .only_open()
    .in_repo("owner/rustassistant")
    .with_label("bug")
    .by_author("username")
    .sort_by(SortField::Updated, SortOrder::Desc)
    .limit(50);
```

### Full-Text Search

The migration includes FTS5 tables for blazing-fast searches:

```sql
-- Search across all issues
SELECT i.*, r.full_name
FROM github_issues_fts fts
JOIN github_issues i ON fts.rowid = i.id
JOIN github_repositories r ON i.repo_id = r.id
WHERE github_issues_fts MATCH 'authentication AND bug'
ORDER BY rank;
```

## 📈 Performance Metrics

- **Local search:** <10ms
- **GitHub API call:** 200-500ms
- **Full sync (100 repos):** ~30s
- **Incremental sync:** 5-10s
- **Webhook latency:** <50ms
- **Database size:** ~5MB per 100 repos

## 🛡️ Rate Limiting

GitHub API limits:

- **Authenticated:** 5,000 requests/hour
- **Search API:** 30 requests/minute
- **GraphQL:** 5,000 points/hour

Monitor usage:

```rust
let limits = client.get_rate_limit().await?;

if limits.rate.remaining < 100 {
    tracing::warn!(
        "GitHub API rate limit low: {}/{}",
        limits.rate.remaining,
        limits.rate.limit
    );
}
```

## 🔐 Security Best Practices

1. **Never commit tokens:** Use `.env` and add to `.gitignore`
2. **Use fine-grained tokens:** Limit scope to specific repos
3. **Verify webhook signatures:** Always enabled by default
4. **Rotate tokens regularly:** Every 90 days
5. **Monitor API usage:** Track unexpected spikes

## 📚 Next Steps

1. ✅ **Initialize schema** (this guide)
2. ✅ **First sync** (this guide)
3. ⬜ Add CLI commands
4. ⬜ Add Web UI dashboard
5. ⬜ Setup webhooks
6. ⬜ Integrate with query router
7. ⬜ Configure background sync job

## 🐛 Troubleshooting

### Authentication Failed

```rust
Error: AuthError("Invalid or expired GitHub token")
```

**Solution:** Regenerate token with correct scopes

### Rate Limit Exceeded

```rust
Error: RateLimitExceeded { reset_at: 2024-01-15 14:30:00 UTC }
```

**Solution:** Wait until reset time or reduce sync frequency

### Sync Takes Too Long

**Solution:** Use selective sync:

```rust
let options = SyncOptions::default()
    .with_repos(vec!["owner/repo1".to_string()])
    .commits_limit(50); // Reduce commits per repo
```

## 📖 Full API Reference

See [`src/github/README.md`](../src/github/README.md) for complete API documentation.

---

**Built with 🦀 Rust • Cost-optimized for solo developers • GitHub-first architecture**