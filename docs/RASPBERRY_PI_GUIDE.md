# Raspberry Pi Deployment Guide

**Status:** ✅ Production Ready  
**Date:** February 3, 2026  
**Target:** Raspberry Pi 4/5 with Debian/Raspbian OS

---

## Overview

RustAssistant can be deployed on a Raspberry Pi to provide automated repository scanning and analysis from a low-power, always-on device. This guide covers installation, configuration, and remote management via the web interface.

---

## Prerequisites

### Hardware
- **Raspberry Pi 4 or 5** (4GB+ RAM recommended)
- **32GB+ SD card** (for database and cache storage)
- **Network connection** (WiFi or Ethernet)
- **Power supply** (official Raspberry Pi power adapter recommended)

### Software
- **Raspbian/Debian 11+** or Ubuntu Server 22.04+
- **Rust toolchain** (1.70+)
- **Git** (for cloning repositories)
- **Optional:** SSH enabled for remote access

---

## Installation

### 1. Install Rust

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload shell
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Clone RustAssistant

```bash
# Create workspace
mkdir -p ~/projects
cd ~/projects

# Clone repository
git clone https://github.com/nuniesmith/rustassistant.git
cd rustassistant
```

### 3. Configure Environment

```bash
# Create .env file
cat > .env << 'EOF'
# Database
DATABASE_URL=sqlite:data/rustassistant.db

# XAI/Grok API Key (required for analysis)
XAI_API_KEY=your-api-key-here

# GitHub Token (optional, for higher rate limits)
GITHUB_TOKEN=your-github-token-here

# Server Configuration
HOST=0.0.0.0
PORT=3000
EOF

# Make data directory
mkdir -p data
```

### 4. Build RustAssistant

```bash
# Build release binaries (optimized for Pi)
# Note: This may take 20-40 minutes on Raspberry Pi 4
cargo build --release

# Verify binaries
ls -lh target/release/rustassistant*
```

### 5. Initialize Database

```bash
# Run CLI to initialize database
./target/release/rustassistant repo list

# This will create the database if it doesn't exist
```

---

## Running the Server

### Manual Start

```bash
# Start the server
./target/release/rustassistant-server

# Server will listen on http://0.0.0.0:3000
```

### Systemd Service (Recommended)

Create a systemd service for auto-start on boot:

```bash
# Create service file
sudo tee /etc/systemd/system/rustassistant.service << 'EOF'
[Unit]
Description=RustAssistant Analysis Server
After=network.target

[Service]
Type=simple
User=pi
WorkingDirectory=/home/pi/projects/rustassistant
Environment="DATABASE_URL=sqlite:data/rustassistant.db"
EnvironmentFile=/home/pi/projects/rustassistant/.env
ExecStart=/home/pi/projects/rustassistant/target/release/rustassistant-server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable rustassistant.service
sudo systemctl start rustassistant.service

# Check status
sudo systemctl status rustassistant.service
```

### View Logs

```bash
# Follow service logs
sudo journalctl -u rustassistant.service -f

# View last 100 lines
sudo journalctl -u rustassistant.service -n 100
```

---

## Web Interface Access

### From Same Network

1. **Find Raspberry Pi IP:**
   ```bash
   hostname -I
   # Example output: 192.168.1.100
   ```

2. **Access web interface:**
   - Open browser on any device on same network
   - Navigate to: `http://192.168.1.100:3000/static/repos.html`

### From Internet (Port Forwarding)

1. **Configure router port forwarding:**
   - External port: 3000
   - Internal IP: Your Pi's IP (e.g., 192.168.1.100)
   - Internal port: 3000

2. **Find external IP:**
   ```bash
   curl ifconfig.me
   ```

3. **Access remotely:**
   - Navigate to: `http://YOUR_EXTERNAL_IP:3000/static/repos.html`

**Security Note:** Consider using a reverse proxy with HTTPS (nginx + Let's Encrypt) for production internet access.

---

## Using the Web Interface

### Repository Scanner Dashboard

**URL:** `http://YOUR_PI_IP:3000/static/repos.html`

#### Features

1. **Scan Repositories**
   - Click "🔄 Scan Repositories" to sync from GitHub
   - Automatically discovers all your repositories
   - Updates the tracked repository list

2. **View Repository List**
   - See all tracked repositories
   - Status (active/inactive)
   - Last analyzed timestamp
   - Repository metadata

3. **Queue Status**
   - Click "📊 Queue Status" to check processing queue
   - Shows items in inbox, pending, and analyzing stages

4. **Auto-Refresh**
   - Dashboard refreshes every 30 seconds
   - Manual refresh with "♻️ Refresh List"

5. **Statistics Cards**
   - Total repositories
   - Active repositories
   - Queue items
   - Last scan time

---

## Command-Line Operations

### Repository Management

```bash
# List all repositories
./target/release/rustassistant repo list

# Add a repository manually
./target/release/rustassistant repo add ~/github/myproject --name myproject

# Scan GitHub repositories
./target/release/rustassistant scan repos
```

### Cache Management

```bash
# Initialize cache in a repository
./target/release/rustassistant cache init --path ~/github/myproject

# Check cache status
./target/release/rustassistant cache status --path ~/github/myproject

# Clear cache
./target/release/rustassistant cache clear --path ~/github/myproject --all
```

### Analysis Commands

```bash
# Analyze a file for refactoring
./target/release/rustassistant refactor analyze src/main.rs

# Generate documentation
./target/release/rustassistant docs module src/lib.rs

# Scan for TODOs
./target/release/rustassistant scan todos ~/github/myproject
```

### Batch Scanning

```bash
# Scan repository and cache results (Python script)
python3 scripts/batch_cache.py ~/github/myproject --limit 50

# With auto-commit
python3 scripts/batch_cache.py ~/github/myproject --limit 50 --commit
```

---

## Automation with Cron

### Nightly Repository Scan

```bash
# Edit crontab
crontab -e

# Add this line (scan at 2 AM daily)
0 2 * * * cd /home/pi/projects/rustassistant && ./target/release/rustassistant scan repos >> /home/pi/logs/rustassistant-scan.log 2>&1
```

### Hourly Cache Updates

```bash
# Update cache for tracked repos every hour
0 * * * * cd /home/pi/projects/rustassistant && python3 scripts/batch_cache.py ~/github/fks --limit 10 >> /home/pi/logs/rustassistant-cache.log 2>&1
```

---

## Performance Optimization for Raspberry Pi

### 1. Enable Swap (for compilation)

```bash
# Increase swap size for building
sudo dphys-swapfile swapoff
sudo nano /etc/dphys-swapfile
# Set CONF_SWAPSIZE=2048

sudo dphys-swapfile setup
sudo dphys-swapfile swapon
```

### 2. Use Release Builds

Always use `--release` flag for production:
- Faster execution
- Lower memory usage
- Better for limited Pi resources

### 3. Limit Concurrent Analysis

When using batch scanner, limit parallelism:

```bash
# Don't use --parallel on Pi, stick to sequential
python3 scripts/batch_cache.py ~/github/myproject --limit 20
```

### 4. Database Optimization

```bash
# Vacuum database monthly
sqlite3 data/rustassistant.db "VACUUM;"

# Analyze database for query optimization
sqlite3 data/rustassistant.db "ANALYZE;"
```

---

## Monitoring & Maintenance

### Check Service Health

```bash
# Service status
sudo systemctl status rustassistant.service

# Is server responding?
curl http://localhost:3000/health

# Database size
du -sh data/rustassistant.db

# Cache size
du -sh .rustassistant/
```

### Log Rotation

```bash
# Create logrotate config
sudo tee /etc/logrotate.d/rustassistant << 'EOF'
/var/log/rustassistant/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
}
EOF
```

### Backup Strategy

```bash
# Backup database
cp data/rustassistant.db data/rustassistant.db.backup

# Backup configuration
tar -czf rustassistant-backup-$(date +%Y%m%d).tar.gz .env data/ .rustassistant/

# Sync backups to external storage
rsync -av data/ /mnt/external/rustassistant-backups/
```

---

## Troubleshooting

### Server Won't Start

```bash
# Check logs
sudo journalctl -u rustassistant.service -n 50

# Common issues:
# 1. Port already in use
sudo lsof -i :3000

# 2. Database locked
rm data/rustassistant.db-shm data/rustassistant.db-wal

# 3. Missing API key
grep XAI_API_KEY .env
```

### Out of Memory During Build

```bash
# Use cargo with limited jobs
cargo build --release -j 2

# Or build on another machine and copy binary
scp target/release/rustassistant* pi@raspberry-pi:~/rustassistant/target/release/
```

### Cannot Access Web Interface

```bash
# Check firewall
sudo ufw status
sudo ufw allow 3000/tcp

# Check server is listening
sudo netstat -tlnp | grep 3000

# Test from Pi itself
curl http://localhost:3000/health
```

### Slow Analysis Performance

```bash
# Check CPU/memory usage
htop

# Reduce concurrent operations
# Use --limit flag with smaller numbers

# Enable cache to avoid re-analysis
./target/release/rustassistant cache init
```

---

## Security Best Practices

### 1. Firewall Configuration

```bash
# Enable firewall
sudo ufw enable

# Allow SSH (if using)
sudo ufw allow ssh

# Allow RustAssistant only from local network
sudo ufw allow from 192.168.1.0/24 to any port 3000
```

### 2. API Key Security

```bash
# Restrict .env file permissions
chmod 600 .env

# Never commit .env to git
echo ".env" >> .gitignore
```

### 3. Regular Updates

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Update RustAssistant
cd ~/projects/rustassistant
git pull
cargo build --release
sudo systemctl restart rustassistant.service
```

### 4. HTTPS with Nginx (Optional)

```bash
# Install nginx and certbot
sudo apt install nginx certbot python3-certbot-nginx

# Configure reverse proxy
# See docs/NGINX_SETUP.md for details
```

---

## Example: Complete Setup from Scratch

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. Clone and setup
git clone https://github.com/nuniesmith/rustassistant.git
cd rustassistant
mkdir -p data

# 3. Configure
cat > .env << 'EOF'
DATABASE_URL=sqlite:data/rustassistant.db
XAI_API_KEY=xai-your-key-here
GITHUB_TOKEN=ghp_your-token-here
HOST=0.0.0.0
PORT=3000
EOF

# 4. Build (grab coffee, this takes time)
cargo build --release

# 5. Initialize
./target/release/rustassistant repo list

# 6. Start server
./target/release/rustassistant-server &

# 7. Open web interface
# From your laptop/phone browser:
# http://YOUR_PI_IP:3000/static/repos.html

# 8. Scan repositories
# Click "Scan Repositories" in web UI
# Or run: ./target/release/rustassistant scan repos
```

---

## Resource Usage

**Typical Usage on Raspberry Pi 4 (4GB):**

- **RAM:** 200-400 MB (server running)
- **CPU:** 5-15% (idle), 80-100% (during analysis)
- **Disk:** 500 MB (binaries + database)
- **Network:** Minimal (<1 MB/day for API calls)

**Power Consumption:** ~3-5W (with official power supply)

---

## Next Steps

1. ✅ **Set up systemd service** for auto-start
2. ✅ **Access web interface** from your network
3. ✅ **Scan your repositories** using web UI
4. ✅ **Schedule automated scans** with cron
5. ✅ **Monitor queue status** regularly
6. ⏭️ **Implement HTTPS** for internet access (optional)
7. ⏭️ **Set up backups** to external storage

---

## Support & Documentation

- **Main Docs:** `docs/CACHE_INTEGRATION.md`
- **Deployment:** `docs/CACHE_DEPLOYMENT.md`
- **API Reference:** Check `/api/*` endpoints in `src/server.rs`
- **Issues:** GitHub Issues

---

## Summary

You now have RustAssistant running on your Raspberry Pi! You can:

✅ **Scan repositories** from the web interface  
✅ **View repository status** in real-time  
✅ **Monitor queue** for pending analyses  
✅ **Cache results** to save API costs  
✅ **Access remotely** from any device on your network  
✅ **Run 24/7** with low power consumption  

The Raspberry Pi setup is perfect for continuous monitoring and analysis of your repositories without tying up your development machine! 🚀🥧