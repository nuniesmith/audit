# 🔍 Auto-Scanner Setup Guide

## ✅ Current Status

The auto-scanner is **ACTIVE and RUNNING**!

```
🔍 Starting auto-scanner (interval: 60 minutes)
Starting auto-scanner with 60 minute intervals
Checking 1 enabled repositories
Scanning repository: fks (/home/jordan/github/fks)
No changes detected in fks
```

---

## 📋 What is Auto-Scanner?

The auto-scanner automatically monitors enabled repositories for changes and re-analyzes modified files using AI (Grok). It:

- ✅ Runs in the background as part of the API service
- ✅ Checks repositories at configurable intervals (default: 60 minutes)
- ✅ Detects changed files using `git status`
- ✅ Analyzes only Rust, Python, JS/TS files
- ✅ Caches results to avoid redundant API calls
- ✅ Respects concurrent scan limits

---

## 🛠️ Configuration

### Environment Variables

Add to `docker-compose.yml` or `.env`:

```yaml
environment:
  - AUTO_SCAN_ENABLED=true              # Enable/disable scanner
  - AUTO_SCAN_INTERVAL=60               # Check interval in minutes
  - AUTO_SCAN_MAX_CONCURRENT=2          # Max parallel scans
```

### Current Configuration

| Setting | Value | Description |
|---------|-------|-------------|
| **Enabled** | `true` | Auto-scanner is active |
| **Interval** | `60 minutes` | Checks repos every hour |
| **Max Concurrent** | `2` | Scans up to 2 repos at once |

---

## 📁 Repository Setup

### Prerequisites

1. **Local Repository Access**
   - Repos must be mounted in the container
   - Already configured: `/home/jordan/github:/home/jordan/github:ro`

2. **Git Installed**
   - ✅ Already added to Dockerfile
   - Version: `git version 2.39.5`

3. **Database Entry**
   - Repository must exist in the database
   - Auto-scan must be enabled for that repo

---

## 🚀 Enable Auto-Scan for a Repository

### Method 1: Using the API

```bash
# Get repository ID
curl -s http://localhost:3000/api/repos | jq '.data[] | select(.name=="fks") | .id'

# Enable auto-scan (60 minute interval)
curl -X POST http://localhost:3000/api/repos/enable-auto-scan \
  -H "Content-Type: application/json" \
  -d '{"repo_id":"YOUR_REPO_ID","interval_minutes":60}'
```

### Method 2: Using SQL

```bash
# Enable auto-scan for fks repo
sqlite3 data/rustassistant.db <<EOF
UPDATE repositories 
SET auto_scan_enabled = 1, scan_interval_minutes = 60 
WHERE name = 'fks';
EOF
```

### Method 3: Using CLI (if available)

```bash
docker compose exec api rustassistant repo enable-auto-scan --repo fks --interval 60
```

---

## 📊 Currently Enabled Repositories

```
Repository: fks
Path: /home/jordan/github/fks
Status: ✅ Auto-scan enabled
Interval: 60 minutes
Last Check: Active
```

---

## 🔧 How It Works

### 1. Background Loop

```
Every 60 seconds → Check which repos need scanning
  ├─ Has enough time passed since last check?
  ├─ Is repo enabled for auto-scan?
  └─ Scan if both conditions met
```

### 2. Change Detection

```
Run git status --porcelain in repo directory
  ├─ Parse output for modified files
  ├─ Filter for .rs, .py, .js, .ts, .tsx files
  └─ Skip deleted files
```

### 3. File Analysis

```
For each changed file:
  ├─ Check cache (avoid redundant analysis)
  ├─ If cache miss → Analyze with Grok API
  ├─ Store results in cache
  └─ Update last_analyzed timestamp
```

---

## 📈 Monitoring

### View Scanner Logs

```bash
# Real-time logs
docker compose logs -f api | grep auto_scanner

# Recent activity
docker compose logs api --tail=100 | grep -i "scanning\|changed"
```

### Check Repository Status

```bash
# View all repos and their auto-scan status
curl -s http://localhost:3000/api/repos | jq '.data[] | {
  name: .name,
  auto_scan: .auto_scan_enabled,
  interval: .scan_interval_minutes,
  last_check: .last_scan_check
}'
```

### Database Queries

```sql
-- View enabled repos
SELECT name, path, scan_interval_minutes, last_scan_check 
FROM repositories 
WHERE auto_scan_enabled = 1;

-- Reset last scan check (force immediate scan)
UPDATE repositories 
SET last_scan_check = NULL 
WHERE name = 'fks';

-- View last analyzed time
SELECT name, datetime(last_analyzed, 'unixepoch') as last_analyzed_time
FROM repositories 
WHERE auto_scan_enabled = 1;
```

---

## ⚙️ Configuration Examples

### High-Frequency Scanning (15 minutes)

```sql
UPDATE repositories 
SET scan_interval_minutes = 15 
WHERE name = 'fks';
```

### Low-Frequency Scanning (Daily)

```sql
UPDATE repositories 
SET scan_interval_minutes = 1440 
WHERE name = 'fks';
```

### Disable Auto-Scan

```sql
UPDATE repositories 
SET auto_scan_enabled = 0 
WHERE name = 'fks';
```

---

## 🔬 Testing

### Force an Immediate Scan

```bash
# Reset the last_scan_check to trigger immediate scan
sqlite3 data/rustassistant.db "UPDATE repositories SET last_scan_check = NULL WHERE name = 'fks';"

# Wait 60 seconds for next scanner cycle
sleep 65

# Check logs
docker compose logs api --tail=20 | grep fks
```

### Create a Test File to Detect

```bash
# Make a change in the fks repo
cd /home/jordan/github/fks
echo "// Test file for auto-scanner" > test_scan.rs
git status

# Wait for next scan cycle (up to 60 minutes)
# Or force immediate scan with SQL above
```

---

## 📝 Log Output Examples

### Successful Scan with Changes

```
INFO rustassistant::auto_scanner: Scanning repository: fks (/home/jordan/github/fks)
INFO rustassistant::auto_scanner: Found 3 changed files in fks
INFO rustassistant::auto_scanner: Analyzing /home/jordan/github/fks/src/main.rs
INFO rustassistant::auto_scanner: Cached analysis for src/main.rs
```

### No Changes Detected

```
INFO rustassistant::auto_scanner: Scanning repository: fks (/home/jordan/github/fks)
DEBUG rustassistant::auto_scanner: No changes detected in fks
```

### Skipped Due to Recent Scan

```
DEBUG rustassistant::auto_scanner: Skipping fks - scanned 45 seconds ago
```

---

## 🎯 Best Practices

### Interval Selection

- **Active Development**: 15-30 minutes
- **Moderate Activity**: 60 minutes (default)
- **Stable Projects**: 2-4 hours (120-240 minutes)
- **Archived Projects**: Disable auto-scan

### Resource Management

- Limit concurrent scans to avoid API rate limits
- Use caching to reduce redundant LLM calls
- Monitor token usage in cost tracker
- Consider scan intervals based on API budget

### Repository Organization

Enable auto-scan for:
- ✅ Active projects you're currently working on
- ✅ Projects with frequent commits
- ✅ Critical codebases needing continuous analysis

Disable for:
- ❌ Large repos with many files (configure manually)
- ❌ Archived/inactive projects
- ❌ Third-party dependencies

---

## 🔐 Security Notes

- Repositories mounted **read-only** (`:ro`)
- Scanner runs as non-root user (`rustassistant`)
- No write access to repository files
- Only reads git status and file contents

---

## 🐛 Troubleshooting

### Scanner Not Running

**Check logs:**
```bash
docker compose logs api | grep -i "auto-scanner\|Starting"
```

**Expected output:**
```
INFO rustassistant_server: 🔍 Starting auto-scanner (interval: 60 minutes)
INFO rustassistant::auto_scanner: Starting auto-scanner with 60 minute intervals
```

### Repository Not Being Scanned

**Verify enabled:**
```sql
SELECT name, auto_scan_enabled, scan_interval_minutes 
FROM repositories 
WHERE name = 'fks';
```

**Force scan:**
```sql
UPDATE repositories SET last_scan_check = NULL WHERE name = 'fks';
```

### Git Errors

**Test git access:**
```bash
docker compose exec -T api sh -c "cd /home/jordan/github/fks && git status"
```

**Check mount:**
```bash
docker compose exec -T api ls -la /home/jordan/github/
```

### Permission Errors

**Ensure mount is readable:**
```yaml
volumes:
  - /home/jordan/github:/home/jordan/github:ro
```

---

## 📊 Performance Metrics

### Resource Usage

- **CPU**: Minimal (background checks every 60s)
- **Memory**: ~50MB additional for scanner
- **API Calls**: Only on detected changes with cache misses
- **Network**: Only for LLM API requests

### Typical Scan Times

- **Small repo** (10 files): ~5 seconds
- **Medium repo** (100 files): ~30 seconds  
- **Large repo** (1000+ files): ~5 minutes

---

## 🎉 Success Indicators

You know auto-scanning is working when you see:

1. ✅ **Startup logs** show scanner initialization
2. ✅ **Periodic checks** appear every 60 seconds
3. ✅ **Repository scans** happen at configured intervals
4. ✅ **No errors** in git status or file analysis
5. ✅ **Cache hits** for unchanged files

---

## 📚 Related Documentation

- `docker/README.md` - Docker setup and deployment
- `DOCKER_REFACTOR_SUMMARY.md` - Architecture overview
- `WEB_UI_STATUS.md` - Alternative interfaces
- `src/auto_scanner.rs` - Source code

---

## 🔄 Next Steps

1. **Monitor First Scan Cycle** - Wait 60 minutes and check logs
2. **Make Test Changes** - Modify a file in fks and verify detection
3. **Review Analysis Results** - Check database for cached analyses
4. **Tune Intervals** - Adjust based on your workflow
5. **Enable More Repos** - Add other active projects

---

**Status**: ✅ Fully Configured and Running  
**Repository**: fks  
**Interval**: 60 minutes  
**Last Updated**: February 5, 2026