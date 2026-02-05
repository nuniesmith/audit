# Task System Integration Guide

This guide explains how to integrate and use the simplified task management system in rustassistant.

## Overview

The simplified task system consolidates the previous queue-based architecture into a more maintainable, single-table approach with smart grouping and IDE integration features.

### Key Features

- **Unified Task Model**: Single `tasks` table replaces multiple queue tables
- **Smart Grouping**: Automatically groups related tasks by file, category, or similarity
- **IDE Integration**: Format task groups for easy copy/paste into Zed or other IDEs
- **LLM Analysis**: Batch processing with priority scoring and suggestions
- **Database Permissions**: Proper handling across dev/prod environments
- **Cost Tracking**: Monitor LLM token usage and costs

## Architecture Changes

### Before (Queue System)
```
QueueItem → FileAnalysis → TodoItem
  ↓            ↓              ↓
Multiple tables, complex state management
```

### After (Simplified Tasks)
```
Task (single table)
  ├─ Pending → Processing → Review → Ready → Done
  └─ Smart grouping for batch processing
```

## Installation Steps

### 1. Database Migration

The migration has been created at `migrations/001_simplified_tasks.sql`. To apply it:

```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features sqlite

# Run migrations
sqlx migrate run

# Or let the app auto-migrate on startup
export RUSTASSISTANT_AUTO_MIGRATE=true
```

The migration creates:
- `tasks` - Main task table
- `repositories` - Simplified repo tracking
- `task_groups` - Batch IDE export groups
- `llm_usage` - Cost tracking
- Views for common queries

### 2. Environment Configuration

Copy the example configuration:

```bash
cp config/env.example.txt .env
```

Key variables to set:

```bash
# Database path (environment-specific)
RUSTASSISTANT_DB_PATH=./data/rustassistant.db  # Dev
# RUSTASSISTANT_DB_PATH=/var/lib/rustassistant/rustassistant.db  # Prod

# Auto-run migrations on startup
RUSTASSISTANT_AUTO_MIGRATE=true

# Environment mode
RUSTASSISTANT_ENV=development

# Grok API key (required for LLM features)
XAI_API_KEY=your_api_key_here
```

### 3. Update CLI Binary

The new task commands are available in the CLI. Update your main CLI file to include them:

```rust
// In src/bin/cli.rs
use rustassistant::cli::{TaskCommands, handle_task_command};

#[derive(Parser)]
enum Commands {
    // ... existing commands ...
    
    /// Task management commands
    #[command(subcommand)]
    Task(TaskCommands),
}

// In the match statement:
Commands::Task(cmd) => {
    let pool = init_db().await?;
    handle_task_command(&pool, cmd).await?;
}
```

## Usage Guide

### Basic Task Management

```bash
# Add a new task manually
rustassistant task add "Fix memory leak in processor" \
  -p 8 \
  -c bug \
  -f src/queue/processor.rs \
  -r rustassistant

# List pending tasks
rustassistant task list pending

# View all task statistics
rustassistant task stats

# Mark a task as done
rustassistant task done <task-id>
```

### Task Grouping

```bash
# View all task groups (smart grouping by default)
rustassistant task groups

# Use different grouping strategies
rustassistant task groups --group-by file
rustassistant task groups --group-by category
rustassistant task groups --group-by repo

# Filter by minimum priority
rustassistant task groups --min-priority 7
```

### IDE Integration (Zed)

Get the next highest-priority task group formatted for your IDE:

```bash
# Get next group and copy to clipboard
rustassistant task next --copy

# Format for different outputs
rustassistant task next --format zed      # Default: Zed IDE format
rustassistant task next --format markdown # Markdown checklist
rustassistant task next --format json     # JSON for API integration

# Filter by priority
rustassistant task next --min-priority 8 --copy
```

Example output (Zed format):
```
=== src/queue/processor.rs (3 tasks) | Priority: 8 ===

Tasks:
1. [BUG] Fix retry backoff [src/queue/processor.rs:45]
   Suggestion: Implement exponential backoff with jitter
2. [BUG] Handle rate limiting [src/queue/processor.rs:78]
3. [REFACTOR] Error message unclear [src/queue/processor.rs:92]

Relevant files: src/queue/processor.rs, src/llm/client.rs
```

### Programmatic Usage

```rust
use rustassistant::task::{Task, TaskSource, TaskStatus, create_task, group_tasks, GroupingStrategy};
use sqlx::SqlitePool;

async fn example(pool: &SqlitePool) -> anyhow::Result<()> {
    // Create a new task
    let task = Task::new("Refactor authentication", TaskSource::Manual)
        .with_priority(7)
        .with_category(TaskCategory::Refactor)
        .with_source_file("rustassistant", "src/auth.rs", Some(120))
        .with_context("Current auth flow is complex and hard to test");
    
    // Save to database
    create_task(pool, &task).await?;
    
    // Get pending tasks
    let pending = get_pending_tasks(pool, 50).await?;
    
    // Group them smartly
    let groups = group_tasks(pending, GroupingStrategy::Smart);
    
    // Get next group to work on
    if let Some(next) = get_next_group(&groups) {
        println!("{}", next.format_for_zed());
    }
    
    Ok(())
}
```

## Integration with Existing Code

### From Old Queue System

If you have existing queue items, you can migrate them:

```rust
// Migration helper (add to your migration script)
async fn migrate_queue_to_tasks(pool: &SqlitePool) -> anyhow::Result<()> {
    let old_items: Vec<OldQueueItem> = sqlx::query_as(
        "SELECT * FROM queue_items WHERE stage != 'Done'"
    )
    .fetch_all(pool)
    .await?;
    
    for item in old_items {
        let task = Task::new(&item.content, TaskSource::Scan)
            .with_priority(item.priority)
            .with_source_file(&item.repo, &item.file_path, item.line_number);
        
        create_task(pool, &task).await?;
    }
    
    Ok(())
}
```

### From Old Task Generator

The existing `src/tasks.rs` task generator can feed into the new system:

```rust
use rustassistant::tasks::TaskGenerator;
use rustassistant::task::{Task, TaskSource, create_task};

async fn integrate_old_tasks(pool: &SqlitePool) -> anyhow::Result<()> {
    let mut generator = TaskGenerator::new();
    
    // Generate tasks using existing logic
    generator.generate_from_tags(&tags);
    
    // Convert to new task model
    for old_task in generator.tasks() {
        let new_task = Task::new(&old_task.description, TaskSource::Scan)
            .with_priority(match old_task.priority {
                Priority::Critical => 9,
                Priority::High => 7,
                Priority::Medium => 5,
                Priority::Low => 3,
            })
            .with_category(map_category(&old_task.category));
        
        create_task(pool, &new_task).await?;
    }
    
    Ok(())
}
```

## Database Permissions Fix

### Development
```bash
# Database in local data directory
export RUSTASSISTANT_DB_PATH="./data/rustassistant.db"
mkdir -p ./data
chmod 755 ./data
```

### Production
```bash
# System directory with proper permissions
sudo mkdir -p /var/lib/rustassistant
sudo chown -R $USER:$USER /var/lib/rustassistant
sudo chmod 775 /var/lib/rustassistant

export RUSTASSISTANT_DB_PATH="/var/lib/rustassistant/rustassistant.db"
```

The `config.rs` module automatically:
- Creates parent directories if missing
- Sets correct permissions (0o700 for dirs, 0o664 for files)
- Handles WAL and SHM files
- Works across jordan/actions users

## Cost Tracking

Monitor LLM usage and costs:

```bash
# View token usage statistics
rustassistant task stats

# Check daily usage
sqlite3 data/rustassistant.db "SELECT * FROM v_daily_stats LIMIT 7"
```

Estimated costs with Grok 4.1 (~$0.20/M tokens):
- Light usage (10 tasks/day): ~$0.01/day = $0.30/month
- Medium usage (50 tasks/day): ~$0.02/day = $0.60/month
- Heavy usage (200 tasks/day): ~$0.08/day = $2.40/month

## Workflow Examples

### Daily Solo Dev Workflow

```bash
# Morning: Check what's ready
rustassistant task next --min-priority 7 --copy
# Paste into Zed, work on tasks

# Mark completed
rustassistant task done next

# Afternoon: Review backlog
rustassistant task list review

# End of day: Check stats
rustassistant task stats
```

### GitHub TODO Scanning Integration

```bash
# Scan repos for TODOs (using existing scanner)
rustassistant scan todos rustassistant

# This creates tasks automatically
# Then group and prioritize
rustassistant task groups --group-by file

# Work through high-priority file groups
rustassistant task next --format zed --copy
```

### Batch LLM Analysis

```rust
// Process multiple tasks in one LLM call
async fn batch_analyze_tasks(pool: &SqlitePool, llm: &GrokClient) -> anyhow::Result<()> {
    let pending = get_pending_tasks(pool, 10).await?;
    
    // Combine into one prompt
    let combined = pending.iter()
        .enumerate()
        .map(|(i, t)| format!("Task {}: {}", i+1, t.content))
        .collect::<Vec<_>>()
        .join("\n\n");
    
    let prompt = format!(
        "Analyze these tasks and provide for each:\n\
         1. Priority (1-10)\n\
         2. Category (bug/refactor/feature/docs)\n\
         3. Brief suggestion\n\
         \n\
         Respond in JSON array format.\n\n{}",
        combined
    );
    
    let response = llm.generate(&prompt).await?;
    let analyses: Vec<TaskAnalysis> = serde_json::from_str(&response)?;
    
    // Update tasks
    for (task, analysis) in pending.iter().zip(analyses.iter()) {
        update_task_analysis(
            pool,
            &task.id,
            analysis.priority,
            &analysis.category,
            Some(&analysis.suggestion),
            Some(response.len() as i32 / pending.len() as i32),
        ).await?;
    }
    
    Ok(())
}
```

## Troubleshooting

### Database Permission Errors

```bash
# Error: unable to open database file
# Solution: Check directory permissions
ls -la data/
chmod 755 data/
chmod 664 data/rustassistant.db*

# Or use environment-based path
export RUSTASSISTANT_DB_PATH="./dev.db"
```

### Migration Conflicts

```bash
# If you get migration conflicts, check current schema
sqlx migrate info

# Revert if needed
sqlx migrate revert

# Or start fresh (CAUTION: loses data)
rm data/rustassistant.db*
sqlx migrate run
```

### Missing Task Commands

```bash
# Error: unrecognized subcommand 'task'
# Solution: Rebuild after adding TaskCommands
cargo clean
cargo build --release
```

## Next Steps

1. **Integrate with Web UI**: Add task management to the existing web interface
2. **LLM Batch Processing**: Implement batch analysis for cost efficiency
3. **Priority Auto-Adjustment**: Use completion time to refine priority scoring
4. **Context Enhancement**: Automatically pull relevant file context for tasks
5. **IDE Plugins**: Create Zed/VSCode extensions for direct integration

## API Reference

See the module documentation for full API details:
- `rustassistant::task::models` - Task types and database operations
- `rustassistant::task::grouping` - Grouping strategies and filters
- `rustassistant::db::config` - Database configuration
- `rustassistant::cli::task_commands` - CLI interface

## Contributing

When adding new features:
1. Keep the task model simple - resist adding columns
2. Use task metadata (context, llm_suggestion) for extensibility
3. Add grouping strategies as functions, not database logic
4. Format outputs for IDE consumption (not just CLI)

## License

MIT - Same as rustassistant project