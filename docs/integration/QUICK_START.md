# 🚀 Quick Start Guide - Rustassistant

Get up and running in 5 minutes!

---

## Step 1: Build the Project (1 minute)

```bash
cd /home/jordan/github/rustassistant
cargo build --release
```

This creates optimized binaries:
- `target/release/rustassistant` (CLI)
- `target/release/rustassistant-server` (REST API)

---

## Step 2: Verify Configuration (30 seconds)

Check your `.env` file exists with:

```bash
cat .env
```

Should contain:
```
DATABASE_URL=sqlite:/home/jordan/github/rustassistant/data/rustassistant.db
XAI_API_KEY=xai-your-key-here
HOST=127.0.0.1
PORT=3000
```

The database will be created automatically on first use.

---

## Step 3: Try the CLI (2 minutes)

### Add your first note
```bash
./target/release/rustassistant note add "My first note!" --tags test,getting-started
```

Output:
```
✓ Note created
  ID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
  Content: My first note!
  Tags: test,getting-started
```

### List all notes
```bash
./target/release/rustassistant note list
```

### Check statistics
```bash
./target/release/rustassistant stats
```

Output:
```
📊 Rustassistant Statistics

  Total notes: 1
  Inbox notes: 1
  Repositories: 0
  Total tasks: 0
  Pending tasks: 0
```

### Search notes
```bash
./target/release/rustassistant note search "first"
```

---

## Step 4: Add a Repository (1 minute)

Track your current project:

```bash
./target/release/rustassistant repo add /home/jordan/github/rustassistant --name rustassistant
```

List repositories:
```bash
./target/release/rustassistant repo list
```

---

## Step 5: Start the Server (Optional)

In one terminal:
```bash
./target/release/rustassistant-server
```

Output:
```
🚀 Rustassistant server starting on http://127.0.0.1:3000
```

In another terminal, test the API:
```bash
# Health check
curl http://localhost:3000/health

# Get stats
curl http://localhost:3000/api/stats

# Create a note via API
curl -X POST http://localhost:3000/api/notes \
  -H "Content-Type: application/json" \
  -d '{"content":"Created via API!","tags":"api,test"}'

# List notes
curl http://localhost:3000/api/notes
```

---

## Daily Workflow Examples

### Capture Quick Thoughts
```bash
# Add a note
rustassistant note add "Remember to refactor the auth module" --tags todo,code

# Add a note to a specific project
rustassistant note add "New feature idea: dark mode" --project myapp --tags feature,ui
```

### Task Management
```bash
# Create a task (will add this feature in Week 4)
# For now, use notes with task tags
rustassistant note add "Fix database connection pooling" --tags task,priority-high,backend

# List high-priority items
rustassistant note search "priority-high"
```

### Repository Analysis (Coming Soon)
```bash
# Analyze a repository (Week 2-3 feature)
rustassistant repo analyze rustassistant

# Get next recommended task
rustassistant next
```

---

## Common Commands Reference

### Notes
```bash
# Add
rustassistant note add "content" [--tags tag1,tag2] [--project name]

# List
rustassistant note list [--limit 20] [--status inbox|processed|archived]

# Search
rustassistant note search "keyword"
```

### Repositories
```bash
# Add
rustassistant repo add <path> [--name name]

# List
rustassistant repo list

# Remove
rustassistant repo remove <id>
```

### Stats
```bash
rustassistant stats
```

---

## Tips & Tricks

### 1. Create Shell Aliases
Add to your `~/.bashrc` or `~/.zshrc`:

```bash
alias ra='/home/jordan/github/rustassistant/target/release/rustassistant'
alias ran='ra note add'
alias ras='ra stats'
```

Now you can:
```bash
ran "Quick note!" --tags idea
ras
```

### 2. Quick Note Script
Create `~/bin/qnote`:

```bash
#!/bin/bash
/home/jordan/github/rustassistant/target/release/rustassistant note add "$*" --tags quick
```

Make it executable:
```bash
chmod +x ~/bin/qnote
```

Use it:
```bash
qnote "This is a quick thought"
```

### 3. Daily Review Script
Create `~/bin/daily-review`:

```bash
#!/bin/bash
echo "=== Today's Notes ==="
/home/jordan/github/rustassistant/target/release/rustassistant note list --limit 10

echo ""
echo "=== Statistics ==="
/home/jordan/github/rustassistant/target/release/rustassistant stats
```

### 4. Use Tags Effectively
```bash
# Work vs Personal
ran "Meeting notes" --tags work,meetings
ran "Grocery list" --tags personal

# Priority levels
ran "Critical bug fix needed" --tags work,priority-critical
ran "Nice-to-have feature" --tags work,priority-low

# Project-specific
ran "API v2 ideas" --project myapp --tags api,ideas
```

---

## Troubleshooting

### Database Error
If you see "unable to open database file":

```bash
# Make sure the data directory exists
mkdir -p /home/jordan/github/rustassistant/data

# Check .env has correct DATABASE_URL
cat .env | grep DATABASE_URL
```

### Command Not Found
If `rustassistant` isn't found:

```bash
# Use full path
/home/jordan/github/rustassistant/target/release/rustassistant

# Or add to PATH
export PATH=$PATH:/home/jordan/github/rustassistant/target/release
```

### Server Won't Start
Check if port 3000 is already in use:

```bash
lsof -i :3000

# Use a different port
PORT=3001 cargo run --bin rustassistant-server
```

---

## What's Next?

Now that you're up and running, check out:

1. **INTEGRATION_COMPLETE.md** - Full feature list and API documentation
2. **merge/rustassistant_work_plan.md** - Long-term roadmap
3. Start using the system daily to capture notes and ideas!

### Coming Soon (Week 2-4)
- Repository analysis with Grok AI
- Automatic task generation from code TODOs
- Smart task prioritization
- `rustassistant next` command for workflow guidance

---

## Getting Help

- Check the logs: Server outputs to stdout
- Review test output: `cargo test --lib db::tests`
- See full docs: `INTEGRATION_COMPLETE.md`

---

**You're all set! Start capturing notes and building your workflow! 🎉**