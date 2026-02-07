# Migration Guide: Web UI to API-Only Architecture

> **Migration from web interface to API-only audit service with GitHub Actions integration**

## 🎯 Overview

The FKS Audit service has been refactored from a web UI-based service to an **API-only architecture** with integrated GitHub Actions workflows for LLM-powered audits.

## 📋 What Changed

### Removed Features

❌ **Web UI Interface**
- HTML/CSS/JavaScript frontend
- Browser-based audit interface
- Static file serving
- `ServeDir` and static asset handling

### Added Features

✅ **API-Only Server**
- Pure REST API endpoints
- No static file dependencies
- Smaller Docker images
- Better security posture

✅ **Multi-Provider LLM Support**
- XAI Grok API
- Google Gemini API
- Automatic provider detection
- Environment-based configuration

✅ **GitHub Actions Integration**
- Manual LLM audit workflow
- Configurable depth and focus (defaults to all areas)
- Test files included by default for comprehensive analysis
- Automatic report generation
- Artifact uploads with 90-day retention

✅ **Enhanced CI/CD**
- Automated static audits on every push
- LLM audits triggered from GitHub web UI
- No local setup required for LLM audits

## 🔄 Migration Steps

### Step 1: Update Dependencies

**Before:** `Cargo.toml` with `fs` feature
```toml
tower-http = { version = "0.5", features = ["fs", "trace", "cors"] }
```

**After:** Remove `fs` feature
```toml
tower-http = { version = "0.5", features = ["trace", "cors"] }
```

Run:
```bash
cd src/audit
cargo clean
cargo build --release
```

### Step 2: Update Environment Variables

**Before:** Single API key
```env
XAI_API_KEY=xai-xxx
LLM_PROVIDER=grok
LLM_MODEL=grok-4-1-fast-reasoning
```

**After:** Provider-specific keys
```env
# Choose provider
LLM_PROVIDER=xai        # or: google

# Set appropriate API key
XAI_API_KEY=xai-xxx     # For XAI Grok
GOOGLE_API_KEY=AIza-xxx # For Google Gemini

# Model auto-selects based on provider
LLM_MODEL=grok-4-1-fast-reasoning     # or: gemini-2.0-flash-exp
```

### Step 3: Configure GitHub Secrets

Add repository secrets for CI/CD:

1. Go to **Settings** → **Secrets and variables** → **Actions**
2. Add secrets:
   - `XAI_API_KEY` - Your XAI Grok API key
   - `GOOGLE_API_KEY` - Your Google Gemini API key

### Step 4: Update API Calls

**Before:** Web UI access
```
http://localhost:8080/index.html
http://localhost:8080/dashboard.html
```

**After:** API endpoints only
```bash
# Health check
curl http://localhost:8080/health

# Create audit
curl -X POST http://localhost:8080/api/audit \
  -H "Content-Type: application/json" \
  -d '{"repository": "/path", "enable_llm": true}'

# Get results
curl http://localhost:8080/api/audit/{id}
```

### Step 5: Update Workflows

**Before:** Manual LLM setup and execution
```bash
# Had to run locally with API keys
export XAI_API_KEY="..."
cargo run --bin audit-cli -- audit . --llm
```

**After:** Use GitHub Actions workflow
```
1. Go to Actions → 🤖 LLM Audit
2. Click "Run workflow"
3. Select provider and options
4. Download artifacts
```

## 🔧 Code Changes Required

### Server Code

**Before:** `src/server.rs`
```rust
use tower_http::services::ServeDir;

// Serve static files
let static_dir = PathBuf::from("static");
let app = Router::new()
    .nest_service("/static", ServeDir::new(&static_dir))
    .fallback_service(ServeDir::new(&static_dir).append_index_html_on_directories(true))
```

**After:** `src/server.rs`
```rust
// No static file serving imports needed

// API-only router
let app = Router::new()
    .route("/health", get(health_check))
    .route("/api/audit", post(create_audit))
    // ... other API routes only
```

### LLM Client

**Before:** Single provider (XAI only)
```rust
let client = LlmClient::new(
    api_key,
    model,
    max_tokens,
    temperature,
)?;
```

**After:** Multi-provider support
```rust
let client = LlmClient::new_with_provider(
    api_key,
    provider,  // "xai" or "google"
    model,
    max_tokens,
    temperature,
)?;
```

### Configuration

**Before:** Hardcoded XAI
```rust
let api_key = std::env::var("XAI_API_KEY").ok();
```

**After:** Provider-based selection
```rust
let provider = std::env::var("LLM_PROVIDER").unwrap_or("xai".to_string());
let api_key = match provider.as_str() {
    "google" | "gemini" => std::env::var("GOOGLE_API_KEY").ok(),
    "xai" | "grok" => std::env::var("XAI_API_KEY").ok(),
    _ => std::env::var("XAI_API_KEY").ok(),
};
```

## 📊 Comparison Table

| Feature | Before (Web UI) | After (API-Only) |
|---------|----------------|------------------|
| **Interface** | HTML/Browser | REST API |
| **Static Files** | Yes, served | None |
| **LLM Providers** | XAI only | XAI + Google |
| **CI Integration** | Manual | Automated + Manual |
| **Deployment** | Server + Assets | Server only |
| **Dependencies** | tower-http[fs] | tower-http (minimal) |
| **Security** | Web attack surface | API-only |
| **Docker Size** | Larger | Smaller |
| **Configuration** | .env file | .env + GitHub Secrets |

## 🚀 New Workflows

### Automated Static Audits

**Trigger:** Every push to `main` or `develop`

**What it does:**
- Builds audit CLI
- Scans for tags
- Runs static analysis
- Generates tasks
- Uploads artifacts

**Configuration:** `.github/workflows/ci.yml` (already configured)

### Manual LLM Audits

**Trigger:** Manual from GitHub Actions UI

**What it does:**
- Accepts provider selection (XAI or Google)
- Runs deep LLM analysis on all focus areas
- Includes test files by default (for comprehensive analysis)
- Generates comprehensive reports
- Creates actionable tasks
- Uploads artifacts (90-day retention)

**Configuration:** `.github/workflows/llm-audit.yml`

**Usage:**
```
1. Actions → 🤖 LLM Audit → Run workflow
2. Select:
   - LLM Provider: xai | google
   - Analysis Depth: standard | deep
   - Focus Areas: security,logic,performance,compliance,architecture (all by default)
   - Include Tests: true (default - always include paired test files)
   - Batch Size: 10 (1-20)
3. Wait ~15-30 minutes
4. Download artifacts
```

## 🔐 Security Improvements

### Before

- Web UI exposed to network
- Static file serving vulnerability
- Larger attack surface
- Browser-based XSS risks

### After

- API-only endpoints
- No static file serving
- Minimal attack surface
- Server-side only
- API keys in GitHub Secrets (encrypted)

## 💰 Cost Optimization

### LLM API Usage

**Before:** Manual runs, hard to track
```bash
# Easy to accidentally run expensive LLM calls
cargo run --bin audit-cli -- audit . --llm
```

**After:** Controlled via GitHub Actions
- Manual trigger only (no accidental runs)
- Choose provider based on cost (Google has free tier)
- Configurable depth (standard vs deep)
- Batch size control
- Clear audit trail in Actions

### Provider Comparison

| Provider | Free Tier | Cost Model | Best For |
|----------|-----------|------------|----------|
| **XAI Grok** | Limited | Pay-per-token | Production |
| **Google Gemini** | 1,500 req/day | Generous free tier | Testing/Dev |

## 📝 Checklist

Use this checklist to ensure complete migration:

- [ ] Remove old static files (if any)
- [ ] Update `Cargo.toml` dependencies
- [ ] Rebuild with `cargo clean && cargo build --release`
- [ ] Update `.env` with provider-specific keys
- [ ] Add GitHub repository secrets (`XAI_API_KEY`, `GOOGLE_API_KEY`)
- [ ] Test static audit in CI (push to branch)
- [ ] Test manual LLM audit (run workflow)
- [ ] Update any custom scripts to use API endpoints
- [ ] Update documentation references
- [ ] Remove any web UI deployment configs (nginx, etc.)
- [ ] Update Docker configs (smaller images now)
- [ ] Test both XAI and Google providers
- [ ] Verify artifact uploads in GitHub Actions
- [ ] Review and adjust LLM audit frequency/costs

## 🆘 Troubleshooting

### Issue: "No static files found"

**Cause:** Old deployment looking for web UI

**Solution:** This is expected. Remove any web UI references:
```bash
# Remove old static directory (if exists)
rm -rf src/audit/static/

# Update deployment to not serve static files
# Server now runs API-only on port 8080
```

### Issue: "API key not found"

**Cause:** Missing provider-specific environment variable

**Solution:** Set the correct variable:
```bash
# For XAI
export XAI_API_KEY="xai-..."
export LLM_PROVIDER="xai"

# For Google
export GOOGLE_API_KEY="AIza..."
export LLM_PROVIDER="google"
```

### Issue: "LLM workflow fails in GitHub Actions"

**Cause:** Missing repository secrets

**Solution:**
1. Go to Settings → Secrets → Actions
2. Add `XAI_API_KEY` and/or `GOOGLE_API_KEY`
3. Re-run workflow

### Issue: "Can't access web interface"

**Cause:** Web UI removed in this version

**Solution:** Use API endpoints or GitHub Actions:
```bash
# Instead of browser, use curl
curl http://localhost:8080/health

# Or use GitHub Actions for LLM audits
# Actions → 🤖 LLM Audit
```

### Issue: "Wrong API format for provider"

**Cause:** Model name doesn't match provider

**Solution:** Use correct model for provider:
```bash
# XAI
LLM_PROVIDER=xai
LLM_MODEL=grok-4-1-fast-reasoning

# Google
LLM_PROVIDER=google
LLM_MODEL=gemini-2.0-flash-exp
```

## 📚 Additional Resources

- [LLM Audit Guide](./LLM_AUDIT_GUIDE.md) - Comprehensive LLM audit documentation
- [README](./README.md) - Updated service documentation
- [Quick Start](./QUICK_START.md) - Getting started guide
- [CI/CD Workflows](../../.github/workflows/) - GitHub Actions configs

## 🎓 Best Practices

### When to Use Each Approach

**Static Audits (Automated):**
- Every commit
- Fast feedback
- No API costs
- Pattern-based checks

**LLM Audits (Manual):**
- Weekly on `main`
- Before releases
- After security incidents
- Deep analysis needed
- Logic validation

### Cost Management

1. **Use free tiers for testing:**
   - Google Gemini: 1,500 requests/day free
   - Perfect for development

2. **Production with XAI:**
   - More consistent
   - Better for CI/CD
   - Predictable costs

3. **Optimize batch size:**
   - Larger batches = fewer API calls
   - Including tests adds valuable context
   - Sweet spot: 10-15 files

4. **Focus comprehensively:**
   - Use all focus areas by default
   - Tests are included for logic validation
   - Narrow focus only for specific investigations

## ✅ Verification

After migration, verify everything works:

```bash
# 1. Static audit in CI
git push origin feature-branch
# Check Actions tab for green checkmark

# 2. Manual LLM audit
# Go to Actions → 🤖 LLM Audit → Run workflow
# Select XAI provider, standard depth
# Wait for completion, download artifacts

# 3. API server
cd src/audit
export XAI_API_KEY="xai-..."
cargo run --release --bin audit-server &
curl http://localhost:8080/health
# Should return: {"status":"healthy","version":"0.1.0"}

# 4. CLI tool (includes tests by default)
./target/release/audit-cli tags ../..
./target/release/audit-cli static ../..
# Should output reports including test files without errors

# 5. Test exclusion (if needed)
./target/release/audit-cli static ../.. --exclude-tests
# Should skip test files
```

## 🎉 Migration Complete

If all verification steps pass, your migration is complete! You now have:

✅ API-only audit service  
✅ Automated static audits in CI  
✅ Manual LLM audits via GitHub Actions  
✅ Multi-provider LLM support  
✅ Better security and smaller deployments  
✅ Cost-effective AI-powered code analysis  

---

**Questions?** Open an issue or check [LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md)

**Last Updated:** 2024  
**Version:** 0.1.0 (API-only)