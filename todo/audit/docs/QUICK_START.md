# Audit Service - Quick Reference

## 🚀 Quick Start

### 1. Setup

```bash
# Create reports directory
mkdir -p docs/audit

# Add Grok API key to .env (optional - for LLM features)
echo "XAI_API_KEY=your-grok-api-key-here" >> .env
echo "AUDIT_LLM_ENABLED=true" >> .env
```

### 2. Start Service

```bash
# Start audit service
docker compose up -d audit

# Check health
curl http://localhost:8080/health
```

### 3. Run First Audit

```bash
# Quick security scan (no LLM, ~5 seconds)
curl -X POST http://localhost:8080/api/scan/static \
  -H "Content-Type: application/json" \
  -d '{"path": "/app/src/janus", "focus": ["security"]}'

# Or use the helper script
./scripts/audit-fks.sh security
```

## 📁 What Gets Audited

The service analyzes your local FKS source code:

- **`./src/clients`** → `/app/src/clients` (read-only)
- **`./src/janus`** → `/app/src/janus` (read-only)
- **`./src/monitor`** → `/app/src/monitor` (read-only)

Reports are saved to: **`./docs/audit/`**

## 🛠️ Using the Helper Script

```bash
cd fks

# Complete audit (tags + security + all services)
./scripts/audit-fks.sh all

# Scan for TODOs and audit tags
./scripts/audit-fks.sh tags

# Security scan
./scripts/audit-fks.sh security

# Audit specific service
./scripts/audit-fks.sh service forward
./scripts/audit-fks.sh service gateway

# Full audit with LLM
./scripts/audit-fks.sh full /app/src/janus

# Check service health
./scripts/audit-fks.sh health

# View all reports
./scripts/audit-fks.sh reports
```

## 📡 API Examples

### Scan for Audit Tags

```bash
curl -X POST http://localhost:8080/api/scan/tags \
  -H "Content-Type: application/json" \
  -d '{"path": "/app/src/janus"}'
```

### Security Analysis (Static - Fast)

```bash
curl -X POST http://localhost:8080/api/scan/static \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/app/src",
    "focus": ["security"],
    "include_tests": false
  }'
```

### Full Audit with LLM (Deep - Slow)

```bash
curl -X POST http://localhost:8080/api/audit \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/app/src/janus/services/forward",
    "enable_llm": true,
    "focus": ["security", "performance"],
    "include_tests": false
  }'
```

### Audit Specific Service

```bash
# Forward service
curl -X POST http://localhost:8080/api/audit \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/app/src/janus/services/forward",
    "enable_llm": true
  }'

# Gateway service
curl -X POST http://localhost:8080/api/audit \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/app/src/janus/services/gateway",
    "enable_llm": true
  }'
```

## 🏷️ Audit Tags in Code

Add these to your Rust/Python code:

```rust
// @audit-tag: experimental | deprecated | new | old
// @audit-todo: Task description
// @audit-freeze
// @audit-security: Security concern
// @audit-review: Review notes
```

Example:

```rust
// @audit-tag: experimental
// @audit-security: Validate all input before processing
pub async fn handle_order(order: Order) -> Result<OrderResponse> {
    // @audit-todo: Add input validation
    // @audit-todo: Add rate limiting
    process_order(order).await
}

// @audit-freeze
const MAX_POSITION_SIZE: f64 = 100000.0;
```

## 📊 Output & Reports

All reports saved to `./docs/audit/`:

```bash
# List reports
ls -lh docs/audit/

# View latest JSON report
cat docs/audit/audit-*.json | jq .

# View security issues
cat docs/audit/security-*.json | jq '.issues[] | select(.severity=="High")'

# View all TODOs
cat docs/audit/tags-*.json | jq '.tags[] | select(.type=="todo")'
```

## 🎯 Focus Areas

Available focus areas:

- `security` - Security vulnerabilities, unsafe patterns
- `performance` - Performance bottlenecks, inefficiencies  
- `architecture` - Design patterns, code organization
- `complexity` - Code complexity, maintainability
- `deprecated` - Deprecated code, tech debt
- `type-safety` - Type safety, error handling

```bash
# Multiple focus areas
curl -X POST http://localhost:8080/api/audit \
  -H "Content-Type: application/json" \
  -d '{
    "path": "/app/src/janus",
    "focus": ["security", "performance", "architecture"]
  }'
```

## ⚡ Performance

| Operation | Time | LLM |
|-----------|------|-----|
| Tag scan | ~1-2s | No |
| Static analysis | ~5-10s | No |
| LLM audit (service) | ~30-60s | Yes |
| LLM audit (full) | ~2-5min | Yes |

**Tip:** Use static analysis for CI/CD (fast), LLM for deep reviews (slow but thorough).

## 🐛 Troubleshooting

### Service won't start

```bash
# Check logs
docker compose logs audit

# Verify source mounts
docker compose exec audit ls -la /app/src/janus

# Check permissions
ls -la src/
```

### Port conflict (8080 in use)

```bash
# Use different port - edit docker-compose.yml
ports:
  - "8081:8080"

# Then use localhost:8081
curl http://localhost:8081/health
```

### LLM not working

```bash
# Check API key
docker compose exec audit env | grep XAI_API_KEY

# Test API key manually
curl -X POST https://api.x.ai/v1/chat/completions \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"grok-4-1-fast-reasoning","messages":[{"role":"user","content":"test"}]}'
```

### Reports directory missing

```bash
mkdir -p docs/audit
docker compose restart audit
```

## 🔍 Common Workflows

### Daily Development

```bash
# Quick tag scan
./scripts/audit-fks.sh tags

# Check for new security issues
./scripts/audit-fks.sh security
```

### Pre-commit

```bash
# Fast static check
./scripts/audit-fks.sh static /app/src/janus
```

### Weekly Review

```bash
# Full audit with LLM
./scripts/audit-fks.sh all
```

### Before Production Deploy

```bash
# Comprehensive security scan
./scripts/audit-fks.sh security

# Audit critical services
./scripts/audit-fks.sh service forward
./scripts/audit-fks.sh service gateway
```

## 📚 More Info

- **Full Guide**: `docs/audit/README.md`
- **FKS Integration**: `docs/AUDIT_INTEGRATION_SUMMARY.md`
- **Service Docs**: `src/audit/README.md`
- **Deployment**: `docs/AUDIT_DEPLOYMENT_CHECKLIST.md`

## 🌐 Service Info

- **URL**: http://localhost:8080
- **Health**: GET `/health`
- **API Docs**: See `docs/audit/README.md`
- **Container**: `fks_audit`
- **Image**: `fks-audit:latest` (dev) or `nuniesmith/fks:audit-latest` (prod)

## 💡 Tips

1. **Start simple**: Use static analysis first (fast, no API key needed)
2. **Focus your audits**: Target specific services/areas for faster results
3. **Use tags**: Mark code with `@audit-*` comments for tracking
4. **Regular scans**: Run static analysis on every commit, LLM weekly
5. **Check reports**: Review `docs/audit/` regularly for findings

---

**Need help?** Run `./scripts/audit-fks.sh help`
