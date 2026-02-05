# Quick Integration Guide - Simplified Task System

This is a quick-start guide to get the new simplified task system running in your rustassistant project.

## What Changed?

✅ **Added** - New simplified task management system  
✅ **Added** - Smart task grouping for IDE integration  
✅ **Added** - Database configuration with proper permissions  
✅ **Kept** - All existing functionality (queue, scanner, etc.)  

The new system lives alongside your existing code - nothing was removed or broken.

---

## 5-Minute Setup

### 1. Run the Migration

```bash
cd rustassistant

# Install sqlx-cli if you don't have it
cargo install sqlx-cli --no-default-features --features sqlite

# Run the migration
sqlx migrate run

# Verify it worked
sqlite3 data/rustassistant.db "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;"
# Should see: llm_usage, repositories, task_groups, tasks
```

### 2. Update Your CLI Binary

Edit `src/bin/cli.rs` and add the task commands:

```rust
use rustassistant::cli::{
    handle_queue_command, handle_report_command, handle_scan_command,
    handle_task_command, // ADD THIS
    QueueCommands, ReportCommands, ScanCommands,
    TaskCommands, // ADD THIS
};

#[derive(Parser)]
#[command(name = "rustassistant")]
#[command(about = "Developer workflow management", long_about = None)]
enum Commands {
    // ... your existing commands ...
    
    /// Task management (new simplified system)
    #[command(subcommand)]
    Task(TaskCommands),
}

// In your match statement for commands:
async fn run() -> Result<()> {
    match cli.command {
        // ... existing matches ...
        
        Commands::Task(cmd) => {
            let pool = init_db().await?;
            handle_task_command(&pool, cmd).await?;
        }
    }
    Ok(())
}
```

### 3. Rebuild

```bash
cargo build --release
```

### 4. Test It

```bash
# Add a test task
./target/release/rustassistant task add "Test the new task system" -p 5

# List it
./target/release/rustassistant task list pending

# View stats
./target/release/rustassistant task stats

# Success! 🎉
```

---

## Daily Usage

### Quick Commands

```bash
# Add a task manually
rustassistant task add "Fix memory leak" -p 8 -c bug -f src/processor.rs

# See what needs doing
rustassistant task list pending

# Get next task group for Zed (copies to clipboard)
rustassistant task next --copy
# Then paste into Zed AI chat

# Mark done
rustassistant task done <task-id>
# or mark the next ready task
rustassistant task done next

# Check your progress
rustassistant task stats
```

### Integration with Existing Scanner

Your TODO scanner can now create tasks directly:

```rust
// In your scanner code:
use rustassistant::task::{Task, TaskSource, create_task};

// When you find a TODO:
let task = Task::new(&todo_content, TaskSource::Todo)
    .with_priority(7)
    .with_source_file(repo_name, file_path, Some(line_number))
    .with_context(&surrounding_code);

create_task(&pool, &task).await?;
```

---

## Environment Setup (Optional but Recommended)

Copy the example config:

```bash
cp config/env.example.txt .env
```

Key settings in `.env`:

```bash
# Where to store the database
RUSTASSISTANT_DB_PATH=./data/rustassistant.db

# Auto-run migrations on startup
RUSTASSISTANT_AUTO_MIGRATE=true

# Your Grok API key (for LLM features)
XAI_API_KEY=your_key_here
```

---

## Zed IDE Workflow

1. **Morning**: Get your top-priority tasks
   ```bash
   rustassistant task next --min-priority 7 --copy
   ```

2. **Paste into Zed**: The formatted output is ready for Zed's AI chat

3. **Work through tasks**: Let Zed help you implement the fixes

4. **Mark complete**: 
   ```bash
   rustassistant task done next
   ```

5. **Repeat**: Get the next group

---

## Task Grouping Strategies

The system automatically groups related tasks:

```bash
# Smart grouping (default): by file, then category
rustassistant task groups

# Group by file (all tasks in src/main.rs together)
rustassistant task groups --group-by file

# Group by category (all bugs together)
rustassistant task groups --group-by category

# Only show high-priority groups
rustassistant task groups --min-priority 7
```

---

## Common Patterns

### Pattern 1: Daily Standup

```bash
# What's pending?
rustassistant task list pending | head -20

# What's done?
rustassistant task list done | tail -10

# Overall health
rustassistant task stats
```

### Pattern 2: Feature Sprint

```bash
# Create tasks for a feature
rustassistant task add "Add user auth endpoint" -p 8 -c feature
rustassistant task add "Write auth tests" -p 7 -c test
rustassistant task add "Document auth flow" -p 5 -c docs

# Group them
rustassistant task groups --group-by category

# Work through them
rustassistant task next --copy
```

### Pattern 3: Bug Triage

```bash
# Scan for issues
rustassistant scan todos myproject

# Review what was found (tasks auto-created)
rustassistant task list pending

# Prioritize the critical ones
# (Use task next to get highest priority automatically)
rustassistant task next --min-priority 8 --copy
```

---

## Database Permissions (If You Have Issues)

### Development

```bash
mkdir -p ./data
chmod 755 ./data
export RUSTASSISTANT_DB_PATH="./data/rustassistant.db"
```

### Production/Server

```bash
sudo mkdir -p /var/lib/rustassistant
sudo chown $USER:$USER /var/lib/rustassistant
sudo chmod 775 /var/lib/rustassistant
export RUSTASSISTANT_DB_PATH="/var/lib/rustassistant/rustassistant.db"
```

The `db::config` module handles permissions automatically, but these commands help if you run into issues.

---

## What About My Existing Queue System?

**Don't worry!** The new task system doesn't replace your queue - it complements it.

- **Old queue system** (`src/queue/`) - Still works for background processing
- **New task system** (`src/task/`) - Better for manual task tracking and IDE integration

You can use both, or gradually migrate to the new system.

### Migration Script (Optional)

If you want to migrate old queue items to tasks:

```bash
# Connect to your database
sqlite3 data/rustassistant.db

# Check what's in the old queue
SELECT COUNT(*) FROM queue_items WHERE stage != 'Done';

# Exit sqlite and create a migration script
```

See `docs/TASK_SYSTEM_INTEGRATION.md` for the full migration code.

---

## Troubleshooting

### "No such table: tasks"

Run the migration:
```bash
sqlx migrate run
```

### "unrecognized subcommand 'task'"

Rebuild the CLI:
```bash
cargo clean
cargo build --release
```

### Permission denied on database

Check permissions:
```bash
ls -la data/
chmod 664 data/rustassistant.db*
```

Or change the database path:
```bash
export RUSTASSISTANT_DB_PATH="./my-dev.db"
```

---

## Next Steps

1. ✅ **Try it out** - Add a few tasks manually
2. ✅ **Integrate with scanner** - Auto-create tasks from TODOs
3. ✅ **Use with Zed** - Copy task groups to your IDE
4. ✅ **Track costs** - Monitor LLM usage with `task stats`
5. 📖 **Read full docs** - See `docs/TASK_SYSTEM_INTEGRATION.md`

---

## Cost Tracking

With Grok 4.1 (~$0.20 per million tokens):

- **10 tasks/day**: ~$0.01/day = **$0.30/month**
- **50 tasks/day**: ~$0.02/day = **$0.60/month**
- **200 tasks/day**: ~$0.08/day = **$2.40/month**

Check your usage:
```bash
rustassistant task stats
```

---

## Files Added

```
rustassistant/
├── migrations/
│   └── 001_simplified_tasks.sql    # Database schema
├── src/
│   ├── task/
│   │   ├── mod.rs                  # Module exports
│   │   ├── models.rs               # Task types & DB ops
│   │   └── grouping.rs             # Grouping strategies
│   ├── db/
│   │   └── config.rs               # DB configuration
│   └── cli/
│       └── task_commands.rs        # CLI commands
├── .github/workflows/
│   └── ci.yml                      # Updated CI/CD
└── config/
    └── env.example.txt             # Environment template
```

---

## Questions?

- 📖 Full docs: `docs/TASK_SYSTEM_INTEGRATION.md`
- 🐛 Issues: Open a GitHub issue
- 💬 Discussion: Check the chatlog in `merge/chatlog.md`

**Happy coding!** 🚀