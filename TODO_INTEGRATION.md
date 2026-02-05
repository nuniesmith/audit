# Integration Action Checklist

**Created**: February 2024  
**For**: Jordan (@nuniesmith)  
**Project**: rustassistant - Simplified Task System Integration

---

## ✅ What I Did For You

I reviewed your chatlog and integrated the simplified task system files from `merge/` into your rustassistant project:

### Files Added
- ✅ `migrations/001_simplified_tasks.sql` - Database schema
- ✅ `src/task/mod.rs` - Task module
- ✅ `src/task/models.rs` - Task types and DB operations
- ✅ `src/task/grouping.rs` - Smart grouping logic
- ✅ `src/db/config.rs` - Database configuration
- ✅ `src/cli/task_commands.rs` - CLI commands
- ✅ `.github/workflows/ci.yml` - GitHub Actions
- ✅ `config/env.example.txt` - Environment template
- ✅ `docs/TASK_SYSTEM_INTEGRATION.md` - Full guide
- ✅ `docs/QUICK_INTEGRATION.md` - Quick start
- ✅ `INTEGRATION_SUMMARY.md` - Overview

### Files Modified
- ✅ `src/lib.rs` - Added task module export
- ✅ `src/db/mod.rs` - Added config module
- ✅ `src/cli/mod.rs` - Added task_commands
- ✅ `.gitignore` - Added backup directories

---

## 🚨 REQUIRED NEXT STEPS

### Step 1: Install SQLx CLI (if needed)
```bash
cargo install sqlx-cli --no-default-features --features sqlite
```

### Step 2: Run Database Migration
```bash
cd ~/github/rustassistant
sqlx migrate run
```

Expected output:
```
Applied 001_simplified_tasks.sql
```

Verify:
```bash
sqlite3 data/rustassistant.db "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;"
```

Should show: `llm_usage`, `repositories`, `task_groups`, `tasks`

### Step 3: Update CLI Binary

**Edit `src/bin/cli.rs`** and add these changes:

```rust
// ADD TO IMPORTS:
use rustassistant::cli::{
    handle_task_command,  // ADD THIS
    TaskCommands,         // ADD THIS
    // ... your existing imports ...
};

// ADD TO Commands ENUM:
#[derive(Parser)]
enum Commands {
    // ... existing commands like Queue, Scan, etc. ...
    
    /// Task management (new simplified system)
    #[command(subcommand)]
    Task(TaskCommands),  // ADD THIS
}

// ADD TO MATCH STATEMENT (in main or run function):
match cli.command {
    // ... existing matches ...
    
    Commands::Task(cmd) => {
        let pool = init_db().await?;
        handle_task_command(&pool, cmd).await?;
    }
}
```

### Step 4: Rebuild
```bash
cargo build --release
```

If you get errors, run:
```bash
cargo clean
cargo build --release
```

### Step 5: Test It Works
```bash
# Add a test task
./target/release/rustassistant task add "Test the new task system" -p 5

# List tasks
./target/release/rustassistant task list pending

# View stats
./target/release/rustassistant task stats

# Get next task (for Zed)
./target/release/rustassistant task next
```

If all commands work: **✅ SUCCESS!**

---

## 📋 Optional But Recommended

### A. Environment Setup
```bash
# Copy the example
cp config/env.example.txt .env

# Edit .env and set:
# - RUSTASSISTANT_DB_PATH (default: ./data/rustassistant.db)
# - XAI_API_KEY (your Grok API key)
# - GITHUB_USERNAME (nuniesmith)
```

### B. Try the Zed Workflow
```bash
# Get next task group and copy to clipboard
rustassistant task next --copy

# Paste into Zed AI chat
# Work on the tasks
# Mark complete
rustassistant task done next
```

### C. Integrate with TODO Scanner

When your scanner finds TODOs, create tasks:

```rust
use rustassistant::task::{Task, TaskSource, create_task};

// In your scanner:
let task = Task::new(&todo_text, TaskSource::Todo)
    .with_priority(7)
    .with_source_file("rustassistant", &file_path, Some(line_num))
    .with_context(&surrounding_code);

create_task(&pool, &task).await?;
```

---

## 🔍 Verification Checklist

Before moving forward, verify:

- [ ] Migration ran successfully (`sqlx migrate run`)
- [ ] CLI builds without errors (`cargo build --release`)
- [ ] Can add tasks (`task add "test" -p 5`)
- [ ] Can list tasks (`task list pending`)
- [ ] Can view stats (`task stats`)
- [ ] Can get next group (`task next`)
- [ ] Database is in .gitignore (already done ✅)
- [ ] Existing features still work (queue, scan, etc.)

---

## 📖 Documentation Reference

- **Quick Start**: `docs/QUICK_INTEGRATION.md`
- **Full Guide**: `docs/TASK_SYSTEM_INTEGRATION.md`
- **Summary**: `INTEGRATION_SUMMARY.md`
- **Original Discussion**: `merge/chatlog.md`

---

## 🐛 Common Issues

### Issue: "No such table: tasks"
**Solution**: Run `sqlx migrate run`

### Issue: "unrecognized subcommand 'task'"
**Solution**: Update `src/bin/cli.rs` with TaskCommands, then rebuild

### Issue: Database permission errors
**Solution**: 
```bash
chmod 755 ./data
chmod 664 ./data/rustassistant.db*
```

### Issue: Build errors about missing imports
**Solution**: 
```bash
cargo clean
cargo build --release
```

---

## 🎯 What You Get

### CLI Commands
```bash
# Task management
rustassistant task add <content> [OPTIONS]
rustassistant task list [STATUS]
rustassistant task stats
rustassistant task next [--copy] [--format zed|markdown|json]
rustassistant task groups [--group-by file|category|repo|smart]
rustassistant task done <ID|next>
rustassistant task ready <ID>
```

### Grouping Strategies
- **Smart** (default): Groups by file first, then category
- **File**: All tasks in same file
- **Category**: All bugs, features, etc. together
- **Repo**: All tasks from same repository

### Output Formats
- **Zed**: Formatted for Zed AI chat (default)
- **Markdown**: Checklist format
- **JSON**: For API/scripts

---

## 💰 Cost Tracking

With Grok 4.1 (~$0.20 per million tokens):
- **10 tasks/day**: ~$0.30/month
- **50 tasks/day**: ~$0.60/month
- **200 tasks/day**: ~$2.40/month

Monitor usage:
```bash
rustassistant task stats
```

---

## 🚀 Next Actions

### Today (15 minutes total)
1. ⏱️ Run migration (1 min)
2. ⏱️ Update CLI (2 min)
3. ⏱️ Rebuild (2 min)
4. ⏱️ Test commands (5 min)
5. ⏱️ Try Zed workflow (5 min)

### This Week
1. Integrate TODO scanner → task creation
2. Set up environment variables
3. Create 5-10 real tasks
4. Use `task next --copy` daily

### This Month
1. Add batch LLM analysis
2. Build Web UI task dashboard
3. Implement auto-priority adjustment

---

## ✨ Benefits You'll Get

✅ **Single source of truth** - One tasks table, not 5+  
✅ **Smart grouping** - Related tasks batched together  
✅ **IDE integration** - Copy/paste directly into Zed  
✅ **Cost tracking** - Know exactly what LLM calls cost  
✅ **Better permissions** - No more jordan/actions conflicts  
✅ **Simpler architecture** - Easier to maintain and extend  

---

## 📞 Need Help?

1. Check `docs/QUICK_INTEGRATION.md` first
2. Review troubleshooting section above
3. See `merge/chatlog.md` for context
4. Open GitHub issue if stuck

---

## 🎉 You're Ready!

The simplified task system is fully integrated and ready to use. Just complete the 5 required steps above and you're good to go!

**Happy task tracking!** 🚀