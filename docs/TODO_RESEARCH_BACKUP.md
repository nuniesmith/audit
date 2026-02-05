# Research & Backup System Integration Checklist

**Created**: February 2024  
**For**: Jordan (@nuniesmith)  
**Project**: rustassistant - Research & Backup System Integration

---

## ✅ What I Did For You

I've integrated the parallel research system and Google Drive backup into your rustassistant project.

### Files Added
- ✅ `src/research/mod.rs` - Research models & DB operations
- ✅ `src/research/worker.rs` - Parallel worker orchestration
- ✅ `src/research/aggregator.rs` - Result synthesis
- ✅ `src/backup/mod.rs` - rclone-based Google Drive backup
- ✅ `src/llm/simple_client.rs` - Simple Grok API client
- ✅ `src/cli/research_backup_commands.rs` - CLI commands
- ✅ `scripts/setup-pi.sh` - Raspberry Pi setup script
- ✅ `docs/RESEARCH_BACKUP_INTEGRATION.md` - Full guide

### Files Modified
- ✅ `src/lib.rs` - Added backup and research modules
- ✅ `src/llm/mod.rs` - Added simple_client module
- ✅ `src/cli/mod.rs` - Added research_backup_commands

---

## 🚨 REQUIRED NEXT STEPS

### Step 1: Update CLI Binary

**Edit `src/bin/cli.rs`** and add:

```rust
// ADD TO IMPORTS:
use rustassistant::cli::{
    handle_backup_command,      // ADD THIS
    handle_research_command,    // ADD THIS
    BackupCommands,             // ADD THIS
    ResearchCommands,           // ADD THIS
    // ... your existing imports ...
};

// ADD TO Commands ENUM:
#[derive(Parser)]
enum Commands {
    // ... existing commands like Task, Queue, etc. ...
    
    /// Research topics with parallel LLM workers
    #[command(subcommand)]
    Research(ResearchCommands),  // ADD THIS
    
    /// Backup and restore data
    #[command(subcommand)]
    Backup(BackupCommands),      // ADD THIS
}

// ADD TO MATCH STATEMENT (in main or run function):
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
```

### Step 2: Rebuild

```bash
cargo build --release
```

If you get errors:
```bash
cargo clean
cargo build --release
```

### Step 3: Test Locally (5 minutes)

```bash
# Test research (quick mode)
./target/release/rustassistant research quick "What is the Actor model?"

# Test backup check (won't fail if rclone not configured)
./target/release/rustassistant backup setup
```

**If Step 3 works**: ✅ Integration successful!

---

## 📦 OPTIONAL: Raspberry Pi Deployment

If you want to deploy to your Raspberry Pi:

### A. Transfer Files to Pi

```bash
# On your dev machine:
rsync -avz --exclude target --exclude .git \
  ~/github/rustassistant/ pi@your-pi-ip:~/rustassistant/
```

### B. Run Setup Script

```bash
# SSH to your Pi
ssh pi@your-pi-ip

# Run setup (creates directories, installs rclone, sets up systemd)
cd ~/rustassistant
sudo bash scripts/setup-pi.sh
```

**What the script does**:
- Creates `/var/lib/rustassistant` directory
- Installs rclone
- Creates systemd service
- Sets up daily backup cron job (2 AM)
- Creates environment file at `/etc/rustassistant/rustassistant.env`

### C. Add Your API Key

```bash
# Edit the config
sudo nano /etc/rustassistant/rustassistant.env

# Set your Grok API key:
XAI_API_KEY=your_actual_api_key_here
```

### D. Configure Google Drive

```bash
# Run rclone configuration
rclone config

# Choose:
# n (new remote)
# name: gdrive
# storage: drive (Google Drive)
# (accept defaults for client_id, secret, scope)
# Use auto config? 
#   - y (if Pi has browser)
#   - n (if headless - see below)

# Test connection
rclone lsd gdrive:
```

**For headless Pi**:
```bash
# On a machine WITH a browser:
rclone authorize "drive"

# Copy the token it outputs
# Then on your Pi during 'rclone config':
# Choose 'n' for auto config
# Paste the token when prompted
```

### E. Build and Deploy

```bash
# Build on Pi (or cross-compile on dev machine)
cd ~/rustassistant
cargo build --release

# Install binaries
sudo cp target/release/rustassistant /usr/local/bin/
sudo cp target/release/rustassistant-server /usr/local/bin/

# Start service
sudo systemctl enable rustassistant
sudo systemctl start rustassistant

# Check status
sudo systemctl status rustassistant
```

### F. Test Backup

```bash
# Check configuration
rustassistant backup check

# Create a test backup
rustassistant backup create

# List backups
rustassistant backup list
```

---

## 🎯 What You Get

### Research Commands

```bash
# Quick research (2 workers, fast)
rustassistant research quick "What is the Actor model?"

# Standard research (4 workers)
rustassistant research start "Best practices for Rust async error handling"

# Deep research (6 workers, thorough)
rustassistant research start "Comparing Tokio vs async-std" --depth deep

# Code-focused research
rustassistant research start "How to optimize this query" \
  --type code \
  --repo rustassistant \
  --files src/db/query.rs

# List all research
rustassistant research list

# View specific research
rustassistant research view abc12345 --format markdown
rustassistant research view abc12345 --format zed
rustassistant research view abc12345 --format json
```

### Backup Commands

```bash
# Setup instructions
rustassistant backup setup

# Check configuration
rustassistant backup check

# Create backup manually
rustassistant backup create

# List backups
rustassistant backup list

# Restore from backup
rustassistant backup restore backup_20240201_020000
```

---

## 💰 Cost Estimates

With Grok 4.1 (~$0.20 per million tokens):

### Research Costs
| Activity | Tokens | Cost |
|----------|--------|------|
| 1 quick research | 20K | $0.004 |
| 1 standard research | 50K | $0.01 |
| 1 deep research | 100K | $0.02 |

### Daily Usage Example
- 2 quick researches: $0.008
- 1 standard research: $0.01
- 10 task analyses: $0.004
- **Daily total: ~$0.02**
- **Monthly: ~$0.60**

### Backup Costs
**FREE** - Google Drive gives you 15 GB free
- Each backup: ~20-50 MB
- 30 backups: ~1 GB (well under limit)

---

## 🔍 Verification Checklist

- [ ] CLI builds without errors (`cargo build --release`)
- [ ] Can run research command (`research quick "test"`)
- [ ] Can run backup check (`backup check`)
- [ ] (Pi only) Systemd service running
- [ ] (Pi only) rclone configured
- [ ] (Pi only) Can create backup
- [ ] (Pi only) Daily cron job setup

---

## 📖 Documentation Reference

- **Full Guide**: `docs/RESEARCH_BACKUP_INTEGRATION.md`
- **Task Integration**: `docs/TASK_SYSTEM_INTEGRATION.md`
- **Quick Start**: `docs/QUICK_INTEGRATION.md`
- **Setup Script**: `scripts/setup-pi.sh`

---

## 🐛 Common Issues

### Issue: "XAI_API_KEY not set"
**Solution**: 
```bash
export XAI_API_KEY=your_key_here
# Or add to .env or /etc/rustassistant/rustassistant.env
```

### Issue: "rclone not found"
**Solution**: 
```bash
curl https://rclone.org/install.sh | sudo bash
```

### Issue: Build errors about missing types
**Solution**: 
```bash
cargo clean
cargo build --release
```

### Issue: Research times out
**Solution**: Start with quick mode first:
```bash
rustassistant research quick "test question"
```

---

## 🚀 Quick Start Flow

### For Development Testing (5 minutes)
```bash
# 1. Update CLI (edit src/bin/cli.rs)
# 2. Rebuild
cargo build --release

# 3. Test research
./target/release/rustassistant research quick "What is Rust's ownership model?"

# 4. Check backup setup
./target/release/rustassistant backup setup
```

### For Pi Deployment (20 minutes)
```bash
# 1. Transfer code to Pi
rsync -avz ~/github/rustassistant/ pi@your-pi:/home/pi/rustassistant/

# 2. SSH to Pi and run setup
ssh pi@your-pi
cd rustassistant
sudo bash scripts/setup-pi.sh

# 3. Add API key
sudo nano /etc/rustassistant/rustassistant.env

# 4. Configure Google Drive
rclone config

# 5. Build and deploy
cargo build --release
sudo cp target/release/rustassistant* /usr/local/bin/

# 6. Start service
sudo systemctl enable rustassistant
sudo systemctl start rustassistant

# 7. Test
rustassistant research quick "test"
rustassistant backup create
```

---

## ✨ Features Summary

✅ **Parallel Research** - 2-6 LLM workers investigate topics simultaneously  
✅ **Smart Synthesis** - Findings aggregated into coherent reports  
✅ **IDE Integration** - Format for Zed, markdown, or JSON  
✅ **Simple Backup** - No Google API keys or service accounts needed  
✅ **Auto-Cleanup** - Keeps last 30 backups, deletes older ones  
✅ **Safe Restore** - SQLite-safe backup and restore  
✅ **Pi-Ready** - Complete deployment script included  
✅ **Cost-Effective** - ~$0.60/month for typical research usage  

---

## 🎉 You're Ready!

The research and backup systems are fully integrated. Just:

1. Update your CLI binary (5 minutes)
2. Rebuild and test locally (2 minutes)
3. (Optional) Deploy to Pi (20 minutes)

**Happy researching and backing up!** 🚀

---

## Need Help?

1. **Local testing**: Check `docs/RESEARCH_BACKUP_INTEGRATION.md`
2. **Pi deployment**: Review `scripts/setup-pi.sh`
3. **rclone setup**: Run `rustassistant backup setup`
4. **Research examples**: Try `rustassistant research quick "test"`
5. **Issues**: Open GitHub issue with error details