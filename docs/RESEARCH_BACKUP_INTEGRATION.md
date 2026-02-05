# Research & Backup System Integration Guide

**Date**: February 2024  
**Status**: ✅ Ready for Integration  
**Impact**: Additive - No breaking changes to existing code

---

## Overview

This guide covers the integration of two major new features into rustassistant:

1. **Parallel Research System** - Spawn multiple LLM workers to research topics in depth
2. **Google Drive Backup** - Simple rclone-based backups (no API key needed!)

Both systems are designed for your Raspberry Pi deployment and complement the existing task management system.

---

## What Was Added

### 📁 New Directories & Files

```
rustassistant/
├── src/
│   ├── research/                          # Parallel research system
│   │   ├── mod.rs                         # Research models & DB operations
│   │   ├── worker.rs                      # Parallel worker orchestration
│   │   └── aggregator.rs                  # Result synthesis
│   │
│   ├── backup/                            # Google Drive backup
│   │   └── mod.rs                         # rclone-based backup
│   │
│   ├── llm/
│   │   └── simple_client.rs               # Simple Grok API client
│   │
│   └── cli/
│       └── research_backup_commands.rs    # CLI commands
│
├── scripts/
│   └── setup-pi.sh                        # Raspberry Pi setup script
│
└── docs/
    └── RESEARCH_BACKUP_INTEGRATION.md     # This file
```

### 📝 Modified Files

- `src/lib.rs` - Added backup and research modules
- `src/llm/mod.rs` - Added simple_client module
- `src/cli/mod.rs` - Added research_backup_commands

---

## Architecture

### 1. Research System

```
┌─────────────────────────────────────────────────────────┐
│                 Research Request                        │
│                 "Best Rust practices"                   │
└────────────────────┬────────────────────────────────────┘
                     │
          ┌──────────┴──────────┐
          │   LLM Generates     │
          │   Subtopics         │
          └──────────┬──────────┘
                     │
          ┌──────────┴──────────┐
          │                     │
    ┌─────▼─────┐         ┌─────▼─────┐
    │ Worker 1  │   ...   │ Worker N  │
    │ Subtopic A│         │ Subtopic N│
    └─────┬─────┘         └─────┬─────┘
          │                     │
          └──────────┬──────────┘
                     │
          ┌──────────▼──────────┐
          │   LLM Aggregates    │
          │   All Findings      │
          └──────────┬──────────┘
                     │
          ┌──────────▼──────────┐
          │  Research Report    │
          │  (markdown/json)    │
          └─────────────────────┘
```

**Key Features**:
- Multiple parallel workers (2-6 depending on depth)
- Automatic subtopic generation
- RAG integration ready (placeholder for now)
- Results synthesized into coherent reports
- Formatted for Zed IDE, markdown, or JSON

### 2. Backup System

```
┌──────────────────────────────────────────┐
│     rustassistant Data Directory         │
│  (/var/lib/rustassistant/)               │
│                                          │
│  ├── rustassistant.db                    │
│  ├── cache/                              │
│  └── config files                        │
└──────────────┬───────────────────────────┘
               │
               │ rclone copy
               │
               ▼
┌──────────────────────────────────────────┐
│        Google Drive                      │
│  (gdrive:rustassistant-backups/)         │
│                                          │
│  ├── backup_20240201_020000/             │
│  ├── backup_20240202_020000/             │
│  └── backup_20240203_020000/             │
│                                          │
│  (Keeps last 30, auto-cleanup)           │
└──────────────────────────────────────────┘
```

**Key Features**:
- Uses rclone (no Google API key or service account needed)
- Safe SQLite backup using `.backup` command
- Automatic cleanup of old backups
- Daily cron job at 2 AM
- Easy restore on fresh server

---

## Integration Steps

### Step 1: Update Your CLI Binary (5 minutes)

Edit `src/bin/cli.rs` and add the new commands:

```rust
use rustassistant::cli::{
    handle_backup_command,
    handle_research_command,
    handle_task_command,
    BackupCommands,
    ResearchCommands,
    TaskCommands,
    // ... your existing imports
};

#[derive(Parser)]
enum Commands {
    // ... existing commands ...
    
    /// Research topics with parallel LLM workers
    #[command(subcommand)]
    Research(ResearchCommands),
    
    /// Backup and restore data
    #[command(subcommand)]
    Backup(BackupCommands),
}

// In your match statement:
async fn run() -> Result<()> {
    match cli.command {
        // ... existing matches ...
        
        Commands::Research(cmd) => {
            let pool = init_db().await?;
            handle_research_command(&pool, cmd).await?;
        }
        
        Commands::Backup(cmd) => {
            handle_backup_command(cmd).await?;
        }
    }
    Ok(())
}
```

### Step 2: Rebuild (2 minutes)

```bash
cargo build --release
```

### Step 3: Raspberry Pi Setup (10 minutes)

If deploying to your Raspberry Pi:

```bash
# On your Pi, run the setup script
sudo bash scripts/setup-pi.sh

# This will:
# - Create /var/lib/rustassistant directory
# - Install rclone
# - Create systemd service
# - Setup cron for daily backups
# - Create config files
```

### Step 4: Configure Google Drive (5 minutes)

```bash
# Configure rclone (interactive)
rclone config

# Choose:
# - n (new remote)
# - Name: gdrive
# - Storage: drive (Google Drive)
# - Accept defaults for client_id, secret, scope
# - Auto config: y (if browser available)

# Test connection
rclone lsd gdrive:
```

**For headless Pi**:
```bash
# On a machine with a browser:
rclone authorize "drive"

# Copy the token output
# Then on your Pi during 'rclone config':
# Choose 'n' for auto config and paste the token
```

### Step 5: Environment Variables

Edit `/etc/rustassistant/rustassistant.env` (or your `.env`):

```bash
# Existing vars...
XAI_API_KEY=your_grok_api_key_here

# Backup configuration (optional - these are defaults)
BACKUP_REMOTE_NAME=gdrive
BACKUP_REMOTE_PATH=rustassistant-backups
BACKUP_RETENTION_COUNT=30
```

---

## Usage Examples

### Research System

#### Quick Research (2 workers, fast)
```bash
rustassistant research quick "What is the Actor model in Rust?"
```

#### Standard Research (4 workers)
```bash
rustassistant research start "Best practices for async Rust error handling"
```

#### Deep Research (6 workers)
```bash
rustassistant research start "Comparing Tokio vs async-std" \
  --depth deep \
  --type comparison
```

#### Code-Focused Research
```bash
rustassistant research start "How to optimize this database query" \
  --type code \
  --repo rustassistant \
  --files src/db/queue.rs
```

#### View Past Research
```bash
# List all research
rustassistant research list

# View specific research (use first 8 chars of ID)
rustassistant research view abc12345

# Format for Zed IDE
rustassistant research view abc12345 --format zed

# Export as JSON
rustassistant research view abc12345 --format json
```

### Research Depths

| Depth | Workers | Tokens (est) | Cost (est) | Use Case |
|-------|---------|--------------|------------|----------|
| `quick` | 2 | 20K | $0.004 | Simple questions |
| `standard` | 4 | 50K | $0.01 | Most research tasks |
| `deep` | 6 | 100K | $0.02 | Complex topics |

### Backup System

#### Manual Backup
```bash
# Create a backup now
rustassistant backup create

# Output:
# ✓ Backup created successfully!
# Name: backup_20240201_143022
# Size: 2048576 bytes
# Path: gdrive:rustassistant-backups/backup_20240201_143022
```

#### List Backups
```bash
rustassistant backup list

# Output:
# 📦 Available Backups:
# 
#   backup_20240203_020000 (2024-02-03T02:00:00Z)
#   backup_20240202_020000 (2024-02-02T02:00:00Z)
#   backup_20240201_020000 (2024-02-01T02:00:00Z)
```

#### Restore from Backup
```bash
# Stop the service first
sudo systemctl stop rustassistant

# Restore
rustassistant backup restore backup_20240201_020000

# Restart service
sudo systemctl start rustassistant
```

#### Check Configuration
```bash
rustassistant backup check

# Output:
# 🔧 Backup Configuration:
# 
#   Data directory: /var/lib/rustassistant
#   Remote name: gdrive
#   Remote path: rustassistant-backups
#   Retention: 30 backups
# 
# 🔍 Checking rclone...
#   ✓ rclone configured correctly
#   ✓ 15 existing backups
```

#### Setup Instructions
```bash
rustassistant backup setup

# Shows full rclone setup instructions
```

---

## Integration with Existing Features

### Research + Tasks
```bash
# Research a topic
rustassistant research start "Implementing retry logic in Rust" \
  --depth standard

# View the research
rustassistant research view abc12345

# Create tasks based on findings
rustassistant task add "Implement exponential backoff" -p 8 -c feature
rustassistant task add "Add jitter to retry delays" -p 7 -c feature
rustassistant task add "Test retry behavior" -p 6 -c test
```

### Backup + Task Database
The backup system automatically includes:
- SQLite database (all tasks, research, queue items)
- Cache files (LLM responses)
- Configuration files

So your entire workflow is backed up!

---

## Raspberry Pi Deployment

### Systemd Service
After running `setup-pi.sh`, rustassistant runs as a systemd service:

```bash
# Start/stop
sudo systemctl start rustassistant
sudo systemctl stop rustassistant

# Enable on boot
sudo systemctl enable rustassistant

# View logs
journalctl -u rustassistant -f

# Check status
sudo systemctl status rustassistant
```

### Automated Backups
A cron job runs daily at 2 AM:

```bash
# View cron jobs
crontab -l

# Should show:
# 0 2 * * * /usr/local/bin/rustassistant-backup

# Check backup logs
tail -f /var/log/rustassistant/backup.log
```

### Fresh Server Restore
If you need to restore on a new Pi:

```bash
# 1. Run setup
sudo bash scripts/setup-pi.sh

# 2. Configure rclone (same Google account)
rclone config

# 3. Build and install
cargo build --release
sudo cp target/release/rustassistant* /usr/local/bin/

# 4. Check available backups
rustassistant backup list

# 5. Restore latest
rustassistant backup restore backup_YYYYMMDD_HHMMSS

# 6. Start service
sudo systemctl start rustassistant
```

---

## Cost Estimates

Using Grok 4.1 at ~$0.20 per million tokens:

### Research Costs
| Activity | Tokens | Cost |
|----------|--------|------|
| 1 quick research | 20K | $0.004 |
| 1 standard research | 50K | $0.01 |
| 1 deep research | 100K | $0.02 |
| 10 standard researches | 500K | $0.10 |

### Daily Usage Estimate
- 2 quick researches: $0.008
- 1 standard research: $0.01
- 10 task analyses: $0.004
- **Daily total: ~$0.02**
- **Monthly total: ~$0.60**

### Backup Costs
**Google Drive is FREE** for personal use (15 GB included). Your backups will be tiny:
- Database: ~1-5 MB
- Cache: ~10-50 MB
- Total per backup: ~20 MB
- 30 backups: ~600 MB (well under the 15 GB limit)

---

## Troubleshooting

### Research Issues

**Problem**: "XAI_API_KEY not set"
```bash
# Solution: Set your API key
export XAI_API_KEY=your_key_here
# Or add to /etc/rustassistant/rustassistant.env
```

**Problem**: Research fails with timeout
```bash
# Solution: Reduce depth or increase timeout
# Edit WorkerConfig in research_backup_commands.rs
# Change timeout_secs from 120 to 300
```

**Problem**: Workers return empty results
```bash
# Solution: Check API quota and model availability
curl -H "Authorization: Bearer $XAI_API_KEY" \
  https://api.x.ai/v1/models
```

### Backup Issues

**Problem**: "rclone not found"
```bash
# Solution: Install rclone
curl https://rclone.org/install.sh | sudo bash
```

**Problem**: "Remote 'gdrive' not configured"
```bash
# Solution: Run rclone config
rclone config

# Then test:
rclone lsd gdrive:
```

**Problem**: "Permission denied" on restore
```bash
# Solution: Stop the service first
sudo systemctl stop rustassistant

# Then restore
rustassistant backup restore <name>

# Then start
sudo systemctl start rustassistant
```

**Problem**: Backup fails on Pi
```bash
# Check logs
tail -f /var/log/rustassistant/backup.log

# Test manually
/usr/local/bin/rustassistant-backup

# Check rclone connectivity
rclone ls gdrive:rustassistant-backups
```

---

## Advanced Usage

### Custom Research Types

You can add custom research types by modifying the research logic:

```rust
// In your code:
let request = ResearchRequest::new("Your topic", "custom_type")
    .with_depth(ResearchDepth::Deep)
    .with_description("Specific instructions for workers")
    .with_context(Some("rustassistant".to_string()), Some("src/".to_string()));
```

### RAG Integration (Future)

The research system has placeholders for RAG integration:

```rust
// In worker.rs:
pub async fn search_rag_context(
    pool: &SqlitePool,
    query: &str,
    limit: usize,
) -> Result<Vec<RagResult>> {
    // TODO: Integrate with LanceDB vector search
    // This is where you'd query your embeddings
}
```

To integrate:
1. Set up LanceDB with your codebase embeddings
2. Implement `search_rag_context` to query vectors
3. Results will automatically be included in worker prompts

### Custom Backup Schedules

Edit the cron job:
```bash
crontab -e

# Change from 2 AM daily to 6 PM daily:
0 18 * * * /usr/local/bin/rustassistant-backup

# Or run every 6 hours:
0 */6 * * * /usr/local/bin/rustassistant-backup

# Or weekly on Sunday at 3 AM:
0 3 * * 0 /usr/local/bin/rustassistant-backup
```

---

## File Locations

### Development
```
./data/                    # Local data directory
├── rustassistant.db       # Database
├── cache/                 # LLM cache
└── backups/               # Local backup snapshots (temp)
```

### Production (Raspberry Pi)
```
/var/lib/rustassistant/    # Data directory
├── rustassistant.db
├── cache/
└── backups/

/etc/rustassistant/        # Configuration
└── rustassistant.env

/var/log/rustassistant/    # Logs
├── rustassistant.log
├── rustassistant-error.log
└── backup.log

/usr/local/bin/            # Binaries
├── rustassistant
├── rustassistant-server
└── rustassistant-backup
```

---

## Next Steps

### Immediate (This Week)
1. ✅ Integrate research and backup commands into CLI
2. ✅ Deploy to Raspberry Pi using setup script
3. ✅ Configure Google Drive with rclone
4. ✅ Run test research and backup

### Short Term (This Month)
1. 🔄 Connect research system to RAG/vector DB
2. 🔄 Add research → task workflow
3. 🔄 Implement automatic backup verification
4. 🔄 Create Web UI for research management

### Long Term (Future)
1. 📋 Multi-LLM support (OpenAI, Anthropic, local models)
2. 📋 Research templates for common patterns
3. 📋 Incremental backups for large datasets
4. 📋 Backup to multiple cloud providers

---

## Summary

✅ **Integrated** - Research and backup systems ready to use  
✅ **Non-Breaking** - All existing code still works  
✅ **Cost-Effective** - ~$0.60/month for typical research usage  
✅ **Simple Backup** - No Google API keys needed  
✅ **Pi-Ready** - Complete deployment script included  

The research system gives you parallel LLM workers to investigate any topic, while the backup system ensures your data is safe on Google Drive with zero configuration hassle.

**Status**: Ready for production use! 🚀

---

## Questions or Issues?

1. Check this guide for common scenarios
2. Review `scripts/setup-pi.sh` for deployment details
3. Test with `rustassistant research quick "test"`
4. Test backup with `rustassistant backup create`
5. Open GitHub issue if problems persist

**Happy researching and backing up!** 🎉