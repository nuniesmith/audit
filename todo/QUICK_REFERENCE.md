# Quick Reference Card

**Phase 2 Status:** 75% Complete  
**Time to Done:** 8-10 hours

---

## 🚀 RUN THESE COMMANDS NOW

### 1. Verify What Works (30 min)
```bash
cd ~/github/rustassistant

# Make script executable (if not already)
chmod +x scripts/verify_phase2.sh

# Run verification
./scripts/verify_phase2.sh

# Should see: ✓ Passed: X, ✗ Failed: Y
```

---

### 2. Build Latest Version
```bash
cargo build --release
```

---

### 3. Test Queue Commands
```bash
# Set environment
export DATABASE_URL="sqlite:data/rustassistant.db"
export XAI_API_KEY="your_key_here"
export GITHUB_TOKEN="your_github_token"

# Test queue
./target/release/rustassistant queue add "test thought" --source thought
./target/release/rustassistant queue status
./target/release/rustassistant queue list inbox

# Test scan
./target/release/rustassistant scan repos --token $GITHUB_TOKEN
./target/release/rustassistant scan todos .
./target/release/rustassistant scan tree . --depth 3

# Test reports
./target/release/rustassistant report todos --priority 2
./target/release/rustassistant report health .
```

---

## 📝 WHAT TO BUILD NEXT

### Priority 1: Refactor CLI (2 hours)

**File to edit:** `src/bin/cli.rs`

**Add this enum:**
```rust
#[derive(Subcommand)]
enum RefactorAction {
    Analyze { file: String },
    Plan { file: String, #[arg(short, long)] smell: Option<String> },
}
```

**Add to Commands:**
```rust
Refactor { #[command(subcommand)] action: RefactorAction },
```

**Add handler:** (See `todo/rustassistant_action_plan.md` line 180)

**Test:**
```bash
cargo build --release
./target/release/rustassistant refactor analyze src/server.rs
```

---

### Priority 2: Doc Generator (6 hours)

**Follow:** `todo/IMPLEMENT_DOC_GENERATOR.md`

**Summary:**
1. Create `src/doc_generator.rs`
2. Add types: `ModuleDoc`, `ReadmeContent`
3. Implement `DocGenerator` struct
4. Add CLI commands
5. Test

**When done:**
```bash
./target/release/rustassistant docs module src/db.rs
./target/release/rustassistant docs readme .
```

---

## 📚 DOCUMENTATION FILES

| File | Purpose |
|------|---------|
| `todo/START_HERE.md` | Begin here, executive summary |
| `todo/PHASE2_PROGRESS.md` | Visual progress tracking |
| `todo/rustassistant_action_plan.md` | Complete implementation guide |
| `todo/rustassistant_checklist.md` | Detailed task checklist |
| `todo/IMPLEMENT_DOC_GENERATOR.md` | Step-by-step doc generator guide |

---

## 🔧 DAILY WORKFLOW

### Morning (2 min)
```bash
rustassistant queue status
rustassistant next
```

### Capture Ideas (30 sec)
```bash
rustassistant queue add "your brilliant idea"
```

### Evening (5 min)
```bash
rustassistant queue process --batch-size 5
rustassistant report todos --priority 2
git add . && git commit -m "progress" && git push
```

---

## 🐛 TROUBLESHOOTING

### If commands fail:
```bash
# Check database exists
ls -la data/rustassistant.db

# Check tables
sqlite3 data/rustassistant.db ".tables"

# Rebuild
cargo clean
cargo build --release
```

### If API calls fail:
```bash
# Check API key
echo $XAI_API_KEY

# Test connection
curl https://api.x.ai/v1/chat/completions \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model": "grok-beta", "messages": [{"role": "user", "content": "hi"}]}'
```

### If deployment fails:
```bash
# Check CI/CD
# Go to: https://github.com/YOUR_USERNAME/rustassistant/actions

# Check Pi logs
ssh actions@YOUR_TAILSCALE_IP
docker compose -f docker-compose.prod.yml logs -f
```

---

## ✅ COMPLETION CHECKLIST

### This Week
- [ ] Run `./scripts/verify_phase2.sh`
- [ ] Wire up refactor CLI (2 hours)
- [ ] Implement doc generator (6 hours)
- [ ] Test all features end-to-end
- [ ] Tag v0.2.0-beta

### Next Week
- [ ] Add integration tests
- [ ] Set up database migrations
- [ ] Add Prometheus metrics
- [ ] Update documentation
- [ ] Tag v0.2.0 (RELEASE!)

---

## 🎯 SUCCESS METRICS

**Phase 2 DONE = Can run these without errors:**
```bash
rustassistant queue add "test"
rustassistant scan all
rustassistant report todos
rustassistant refactor analyze src/db.rs
rustassistant docs module src/db.rs
```

---

## 📊 CURRENT STATUS

```
✅ Queue System         [████████████████████████] 100%
✅ Code Review         [████████████████████████] 100%
✅ Test Generator      [████████████████████████] 100%
⏳ Refactor CLI        [████████████░░░░░░░░░░░░]  50%
❌ Doc Generator       [░░░░░░░░░░░░░░░░░░░░░░░░]   0%

Overall:                [██████████████████░░░░░░]  75%
```

**Remaining:** 8-10 hours = Phase 2 SHIPPED! 🚀

---

## 💡 REMEMBER

- Queue system is DONE (just test it!)
- Refactor code exists (just needs CLI)
- Only doc generator is truly new
- You're 75% there already!

**One focused weekend = Phase 2 complete!**

---

## 🚀 START NOW

```bash
# Step 1: Verify what works
./scripts/verify_phase2.sh

# Step 2: Read the plan
cat todo/START_HERE.md

# Step 3: Pick your 2-hour block
# (Wire refactor CLI OR start doc generator)

# Step 4: Ship it!
git push origin main
```

**You got this!** 🎉