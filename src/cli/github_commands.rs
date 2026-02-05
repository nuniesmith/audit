//! GitHub CLI commands
//!
//! Provides command-line interface for GitHub integration features.

use crate::github::search::{GitHubSearcher, SearchQuery, SearchType};
use crate::github::{GitHubClient, SyncEngine, SyncOptions};
use clap::{Args, Subcommand};
use sqlx::SqlitePool;
use std::env;

#[derive(Debug, Subcommand)]
pub enum GithubCommands {
    /// Sync GitHub data to local database
    Sync {
        /// Perform a full sync (default: incremental)
        #[arg(long)]
        full: bool,

        /// Sync specific repository (owner/repo format)
        #[arg(short, long)]
        repo: Option<String>,
    },

    /// Search across GitHub data
    Search {
        /// Search query
        query: String,

        /// Type of content to search (repo, issue, pr, commit, all)
        #[arg(short = 't', long, default_value = "all")]
        r#type: String,

        /// Open first result in browser
        #[arg(short, long)]
        open: bool,

        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: i32,
    },

    /// List issues
    Issues {
        /// Filter by repository (owner/repo format)
        #[arg(short, long)]
        repo: Option<String>,

        /// Filter by state (open, closed, all)
        #[arg(short, long, default_value = "open")]
        state: String,

        /// Limit number of results
        #[arg(short, long, default_value = "20")]
        limit: i32,
    },

    /// List pull requests
    Prs {
        /// Filter by repository (owner/repo format)
        #[arg(short, long)]
        repo: Option<String>,

        /// Filter by state (open, closed, merged, all)
        #[arg(short, long, default_value = "open")]
        state: String,

        /// Limit number of results
        #[arg(short, long, default_value = "20")]
        limit: i32,
    },

    /// Show GitHub integration statistics
    Stats,

    /// Show repository information
    Repos {
        /// Filter by language
        #[arg(short, long)]
        language: Option<String>,

        /// Show only starred repositories
        #[arg(short, long)]
        starred: bool,

        /// Limit number of results
        #[arg(short = 'n', long, default_value = "20")]
        limit: i32,
    },

    /// Check GitHub API rate limits
    RateLimit,
}

pub async fn handle_github_command(
    command: GithubCommands,
    pool: &SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get GitHub token
    let token = env::var("GITHUB_TOKEN").map_err(|_| {
        "GITHUB_TOKEN environment variable not set. Create a token at https://github.com/settings/tokens"
    })?;

    let client = GitHubClient::new(token)?;
    let sync_engine = SyncEngine::new(client.clone(), pool.clone());
    let searcher = GitHubSearcher::new(pool.clone());

    match command {
        GithubCommands::Sync { full, repo } => {
            println!("🔄 Starting GitHub sync...");

            let options = if full {
                println!("📊 Performing full sync");
                SyncOptions::default().force_full()
            } else {
                SyncOptions::default()
            };

            let options = if let Some(repo_name) = repo {
                println!("📦 Syncing repository: {}", repo_name);
                let parts: Vec<&str> = repo_name.split('/').collect();
                if parts.len() != 2 {
                    return Err("Repository must be in owner/repo format".into());
                }
                options.with_repos(vec![repo_name])
            } else {
                options
            };

            let result = sync_engine.sync_with_options(options).await?;

            println!("\n✅ Sync complete!");
            println!("   Repositories: {}", result.repos_synced);
            println!("   Issues: {}", result.issues_synced);
            println!("   Pull Requests: {}", result.prs_synced);
            println!("   Duration: {:.2}s", result.duration_secs);
        }

        GithubCommands::Search {
            query,
            r#type,
            open,
            limit,
        } => {
            println!("🔍 Searching for: {}", query);

            let search_type = match r#type.as_str() {
                "repo" | "repository" => SearchType::Repositories,
                "issue" => SearchType::Issues,
                "pr" | "pull-request" => SearchType::PullRequests,
                "commit" => SearchType::Commits,
                "all" => SearchType::All,
                _ => return Err(format!("Unknown search type: {}", r#type).into()),
            };

            let mut search_query = SearchQuery::new(&query).with_type(search_type).limit(limit);

            if open && search_type != SearchType::Repositories && search_type != SearchType::Commits
            {
                search_query = search_query.only_open();
            }

            let results = searcher.search(search_query).await?;

            println!("\n📊 Found {} results:\n", results.len());

            for result in &results {
                match result {
                    crate::github::search::SearchResult::Repository(repo) => {
                        println!(
                            "  📦 {} - ⭐ {} 🍴 {}",
                            repo.full_name, repo.stars, repo.forks
                        );
                        if let Some(desc) = &repo.description {
                            println!("     {}", desc);
                        }
                        println!("     {}", repo.html_url);
                    }
                    crate::github::search::SearchResult::Issue(issue) => {
                        println!("  🐛 #{} - {} [{}]", issue.number, issue.title, issue.state);
                        println!("     {}", issue.html_url);
                    }
                    crate::github::search::SearchResult::PullRequest(pr) => {
                        println!("  🔀 #{} - {} [{}]", pr.number, pr.title, pr.state);
                        println!("     {}", pr.html_url);
                    }
                    crate::github::search::SearchResult::Commit(commit) => {
                        println!("  📝 {} - {}", &commit.sha[..8], commit.message);
                    }
                }
                println!();
            }
        }

        GithubCommands::Issues { repo, state, limit } => {
            let mut query = SearchQuery::new("")
                .with_type(SearchType::Issues)
                .limit(limit);

            if state == "open" {
                query = query.only_open();
            } else if state == "closed" {
                query = query.only_closed();
            }

            let results = searcher.search(query).await?;

            let mut filtered_results = results;
            if let Some(repo_filter) = repo {
                // Note: Filtering by repo would need to be added to the query
                println!("⚠️  Repository filtering not yet implemented in search");
            }

            println!("\n🐛 Issues ({} found):\n", filtered_results.len());
            for result in &filtered_results {
                if let crate::github::search::SearchResult::Issue(issue) = result {
                    println!("#{} [{}] {}", issue.number, issue.state, issue.title);
                    println!("  URL: {}", issue.html_url);
                    println!();
                }
            }
        }

        GithubCommands::Prs { repo, state, limit } => {
            let mut query = SearchQuery::new("")
                .with_type(SearchType::PullRequests)
                .limit(limit);

            if state == "open" {
                query = query.only_open();
            } else if state == "closed" {
                query = query.only_closed();
            }

            let results = searcher.search(query).await?;

            let mut filtered_results = results;
            if let Some(repo_filter) = repo {
                println!("⚠️  Repository filtering not yet implemented in search");
            }

            println!("\n🔀 Pull Requests ({} found):\n", filtered_results.len());
            for result in &filtered_results {
                if let crate::github::search::SearchResult::PullRequest(pr) = result {
                    println!("#{} [{}] {}", pr.number, pr.state, pr.title);
                    println!("  URL: {}", pr.html_url);
                    println!();
                }
            }
        }

        GithubCommands::Stats => {
            let stats: (i64, i64, i64, i64, i64) = sqlx::query_as(
                r#"
                SELECT
                    (SELECT COUNT(*) FROM github_repositories) as repos,
                    (SELECT COUNT(*) FROM github_issues) as issues,
                    (SELECT COUNT(*) FROM github_pull_requests) as prs,
                    (SELECT COUNT(*) FROM github_commits) as commits,
                    (SELECT COUNT(*) FROM github_events) as events
                "#,
            )
            .fetch_one(pool)
            .await?;

            println!("\n📊 GitHub Integration Statistics\n");
            println!("  📦 Repositories:   {}", stats.0);
            println!("  🐛 Issues:         {}", stats.1);
            println!("  🔀 Pull Requests:  {}", stats.2);
            println!("  📝 Commits:        {}", stats.3);
            println!("  📡 Events:         {}", stats.4);

            // Get last sync time
            let last_sync: Option<String> = sqlx::query_scalar(
                "SELECT MAX(last_synced_at) FROM github_repositories WHERE last_synced_at IS NOT NULL"
            )
            .fetch_optional(pool)
            .await?;

            if let Some(sync_time) = last_sync {
                println!("\n  🕐 Last sync: {}", sync_time);
            }

            // Get top repositories by stars
            let top_repos: Vec<(String, i64)> = sqlx::query_as(
                "SELECT full_name, stargazers_count FROM github_repositories
                 ORDER BY stargazers_count DESC LIMIT 5",
            )
            .fetch_all(pool)
            .await?;

            if !top_repos.is_empty() {
                println!("\n  ⭐ Top Repositories:");
                for (name, stars) in top_repos {
                    println!("     • {} (⭐ {})", name, stars);
                }
            }
        }

        GithubCommands::Repos {
            language,
            starred,
            limit,
        } => {
            let mut query = SearchQuery::new("")
                .with_type(SearchType::Repositories)
                .limit(limit);

            // Note: Language and starred filtering would need to be added to SearchQuery
            if language.is_some() || starred {
                println!("⚠️  Advanced filtering not yet fully implemented");
            }

            let results = searcher.search(query).await?;

            println!("\n📦 Repositories ({} found):\n", results.len());
            for result in &results {
                if let crate::github::search::SearchResult::Repository(repo) = result {
                    println!("{}", repo.full_name);
                    if let Some(desc) = &repo.description {
                        println!("  {}", desc);
                    }
                    println!(
                        "  ⭐ {} | 🍴 {} | {}",
                        repo.stars,
                        repo.forks,
                        repo.language.as_deref().unwrap_or("N/A")
                    );
                    println!("  {}", repo.html_url);
                    println!();
                }
            }
        }

        GithubCommands::RateLimit => {
            let rate_limit = client.get_rate_limit().await?;

            println!("\n📊 GitHub API Rate Limits\n");
            println!("  Core API:");
            println!("    Remaining: {}", rate_limit.resources.core.remaining);
            println!("    Limit: {}", rate_limit.resources.core.limit);
            println!("    Resets at: {}", rate_limit.resources.core.reset);

            println!("\n  Search API:");
            println!("    Remaining: {}", rate_limit.resources.search.remaining);
            println!("    Limit: {}", rate_limit.resources.search.limit);
            println!("    Resets at: {}", rate_limit.resources.search.reset);

            println!("\n  GraphQL API:");
            println!("    Remaining: {}", rate_limit.resources.graphql.remaining);
            println!("    Limit: {}", rate_limit.resources.graphql.limit);
            println!("    Resets at: {}", rate_limit.resources.graphql.reset);
        }
    }

    Ok(())
}
