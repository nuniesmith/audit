# 🎯 ACTION PLAN - Fix RustAssistant Deployment

**Current Status:** 🟡 Containers running but database error  
**Time to Fix:** ⏱️ 30 seconds  
**Difficulty:** ✅ Easy

---

## 🚨 IMMEDIATE ACTION REQUIRED (On Raspberry Pi)

Your containers are deployed and running, but there are TWO issues to fix:

1. **Database permissions** - Container crash-looping 
2. **Missing XAI_API_KEY** - Warning: "The XAI_API_KEY variable is not set"

### Fix 1: Database Permissions

```bash
# SSH into your Raspberry Pi
ssh jordan@rasp

# Navigate to project
cd ~/rustassistant

# OPTION A: Run the automated fix script
chmod +x fix-permissions.sh
./fix-permissions.sh

# OPTION B: Manual fix (if you prefer)
docker compose -f docker-compose.prod.yml down
mkdir -p data config
chmod 755 data config
docker compose -f docker-compose.prod.yml up -d

# Verify it's working
docker ps
docker logs rustassistant-web
```

### Fix 2: Add XAI API Key

```bash
# Still on your Raspberry Pi
cd ~/rustassistant

# OPTION A: Use the automated script
chmod +x add-api-key.sh
./add-api-key.sh
# (It will prompt you to enter your API key)

# OPTION B: Manually edit .env
nano .env
# Add this line:
# XAI_API_KEY=your_actual_key_here
# Save with Ctrl+O, exit with Ctrl+X

# Restart container to apply changes
docker compose -f docker-compose.prod.yml restart rustassistant-web

# Check it worked
docker logs rustassistant-web --tail=20
```

**Expected Result:** 
- Container status changes from "Restarting" to "Up" and healthy
- No warning about XAI_API_KEY
- LLM features will work

---

## 📝 LATER: Commit the Fixes (On Your Dev Machine)

After the Raspberry Pi is working, commit the fixes to prevent this in future deployments:

### On Your Development Machine

```bash
cd ~/github/rustassistant

# Review what changed
git status
git diff

# Commit the fixes
git add .github/workflows/ci-cd.yml fix-permissions.sh add-api-key.sh QUICKFIX.md ACTION_PLAN.md
git commit -m "fix: deployment pipeline, database permissions, and XAI API key

Changes:
- Fix CI/CD to use docker-compose.prod.yml explicitly
- Auto-create data/config directories in deployment
- Auto-configure XAI_API_KEY from GitHub secrets
- Add fix-permissions.sh for manual recovery
- Add add-api-key.sh for manual API key setup
- Add comprehensive documentation

Fixes:
- Deployment exit code 1 (wrong compose file)
- Database permissions error (missing directories)
- XAI_API_KEY not being set from GitHub secrets"

# Push to GitHub
git push origin main
```

This will trigger a new CI/CD run, which should now:
- ✅ Pull ARM64 images
- ✅ Create data/config directories
- ✅ Set XAI_API_KEY from GitHub secrets automatically
- ✅ Start containers successfully

---

## 🔍 What We Fixed

### Problem 1: Deployment Failed (Exit Code 1)
**Root Cause:** Workflow was calling `./run.sh start` which used `docker-compose.yml` instead of `docker-compose.prod.yml`

**Fix:** Updated workflow to explicitly use `docker-compose.prod.yml`

### Problem 2: Database Error
**Root Cause:** Container runs as UID 1000, needs `data` directory to exist with proper permissions

**Fix:** 
- Immediate: Run `fix-permissions.sh` on Raspberry Pi
- Long-term: Workflow now auto-creates directories

### Problem 3: Missing XAI_API_KEY
**Root Cause:** GitHub secret wasn't being passed to the .env file during deployment

**Fix:**
- Immediate: Run `add-api-key.sh` on Raspberry Pi or manually edit .env
- Long-term: Workflow now auto-sets from GitHub secrets

---

## ✅ Success Criteria

### On Raspberry Pi
- [x] Containers deployed successfully
- [ ] Both containers show "Up" status (not "Restarting")
- [ ] No database errors in logs
- [ ] No XAI_API_KEY warning in logs
- [ ] XAI_API_KEY set in .env file
- [ ] Can access http://localhost:3001/
- [ ] Health check passes

### On Dev Machine
- [ ] Workflow changes committed
- [ ] Changes pushed to GitHub
- [ ] CI/CD pipeline runs successfully
- [ ] All jobs green (test, build, deploy)

---

## 📊 Current State

### Working ✅
- Tailscale connection
- SSH authentication  
- Git clone/pull
- Docker image pull (ARM64)
- Redis container (running healthy)

### Needs Fix 🔧
- Database permissions (fix immediately on Pi)
- XAI_API_KEY configuration (fix immediately on Pi)
- Workflow improvements (commit later from dev machine)

---

## 🎯 Timeline

### NOW (5 minutes)
1. SSH to Raspberry Pi
2. Run `fix-permissions.sh`
3. Run `add-api-key.sh` (or manually add XAI_API_KEY to .env)
4. Verify containers are healthy
5. Test the application

### LATER TODAY (5 minutes)
1. Review changes on dev machine
2. Commit and push fixes
3. Watch CI/CD pipeline succeed
4. Celebrate! 🎉

---

## 🆘 Need Help?

### Container still restarting?
```bash
docker logs rustassistant-web --tail=50
ls -la ~/rustassistant/data
```

### Application not responding?
```bash
docker compose -f docker-compose.prod.yml ps
curl http://localhost:3001/health
```

### Want to start over?
```bash
cd ~/rustassistant
docker compose -f docker-compose.prod.yml down -v
rm -rf data config
./fix-permissions.sh
```

---

## 📚 Reference Documents

- `QUICKFIX.md` - Detailed fix instructions
- `fix-permissions.sh` - Automated database permissions fix
- `add-api-key.sh` - Automated XAI API key configuration
- `.github/workflows/ci-cd.yml` - Updated deployment workflow

---

## 🎉 Final Notes

The good news: **Your deployment actually worked!** The containers are running on your Raspberry Pi. You just need to fix two quick issues:

1. Database permissions (30 seconds)
2. XAI API key configuration (30 seconds)

After you fix these and commit the changes, future deployments will be fully automated and work perfectly - the workflow will automatically set the XAI_API_KEY from your GitHub secret.

**Priority:** Fix the Raspberry Pi NOW (2 minutes), commit changes LATER (2 minutes).

---

## 📝 Quick Command Summary

**On Raspberry Pi:**
```bash
cd ~/rustassistant
chmod +x fix-permissions.sh add-api-key.sh
./fix-permissions.sh
./add-api-key.sh
docker compose -f docker-compose.prod.yml ps
```

**On Dev Machine:**
```bash
cd ~/github/rustassistant
git add .github/workflows/ci-cd.yml fix-permissions.sh add-api-key.sh QUICKFIX.md ACTION_PLAN.md
git commit -m "fix: deployment pipeline, database permissions, and XAI API key"
git push origin main
```