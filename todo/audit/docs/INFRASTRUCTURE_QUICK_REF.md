# Infrastructure Category - Quick Reference

## Overview
The Infrastructure category now includes all DevOps, configuration, documentation, and tooling files in your codebase.

## Detected File Patterns

### Directories
- `config/` - Configuration files
- `docker/` - Docker-related files
- `docs/` - Documentation
- `scripts/` - Shell scripts and automation
- `.githooks/` - Git hooks
- `.github/` - GitHub Actions/workflows

### Top-Level Files
- `.dockerignore`
- `.gitattributes`
- `.gitignore`
- `compose`, `compose.yml`, `compose.yaml`
- `compose.prod`, `compose.prod.yml`
- `pyproject.toml`
- `README.md`, `README`
- `run.sh`
- Any `Dockerfile`

### File Extensions
- `.sh` - Shell scripts
- `.yml`, `.yaml` - YAML configuration
- `.toml` - TOML configuration

## Security Checks

### Critical Severity
- **Hardcoded Secrets**: Detects passwords, API keys, secrets in code
  - Pattern: `password=`, `api_key=`, `secret=`
  - Recommendation: Use environment variables or secrets manager

### High Severity
- **Dockerfile Root User**: Running containers as root
  - Pattern: `USER root`
  - Recommendation: Create and use non-root user

- **Compose Privileged Mode**: Containers with privileged flag
  - Pattern: `privileged: true`
  - Recommendation: Use specific capabilities instead

- **Dangerous Shell Commands**: Use of `rm -rf`
  - Pattern: `rm -rf` in shell scripts
  - Recommendation: Validate and quote paths properly

### Medium Severity
- **Latest Tags**: Using `:latest` in images
  - Pattern: `:latest` in Dockerfile or compose
  - Recommendation: Pin to specific versions

- **Missing Error Handling**: Shell scripts without `set -e`
  - Pattern: Missing `set -e` or `set -euo pipefail`
  - Recommendation: Add error handling at top of script

## Usage Examples

### Audit a Repository
```bash
# Basic audit (includes infrastructure by default)
cargo run --bin audit-cli -- scan /path/to/repo

# Focus on infrastructure
cargo run --bin audit-cli -- scan /path/to/repo --focus infra
```

### API Request
```json
{
  "repository": "/path/to/repo",
  "branch": "main",
  "enable_llm": false,
  "focus": ["infra"],
  "include_tests": false
}
```

### Expected Output
```json
{
  "system_map": {
    "files_by_category": {
      "infra": 15
    }
  },
  "files": [
    {
      "path": "docker/Dockerfile",
      "category": "infra",
      "priority": "medium",
      "issues": [
        {
          "severity": "medium",
          "category": "code-quality",
          "message": "Using :latest tag - not reproducible",
          "suggestion": "Pin to specific version"
        }
      ]
    }
  ]
}
```

## Best Practices

### Dockerfiles
- Use specific version tags, not `:latest`
- Run as non-root user
- Multi-stage builds for smaller images
- No hardcoded secrets

### Docker Compose
- Pin all image versions
- Avoid `privileged: true`
- Use secrets management
- Define resource limits

### Shell Scripts
- Start with `#!/bin/bash` or `#!/bin/sh`
- Add `set -e` or `set -euo pipefail` early
- Quote all variable expansions
- Validate paths before destructive operations
- Use shellcheck for validation

### Configuration Files
- No secrets in config files
- Use environment variables
- Document all configuration options
- Version control safe defaults

## Integration with Other Categories

Infrastructure files can reference:
- **Python**: Deploy scripts for Python services
- **Rust**: Dockerfiles for Rust applications
- **Documentation**: Setup guides in docs/
- **Tests**: CI/CD workflows running tests

All infrastructure issues contribute to the overall audit report and security rating.

## Testing Your Infrastructure

```bash
# Run infrastructure-specific tests
cargo test --lib scanner::tests::test_infra_category_detection
cargo test --lib scanner::tests::test_check_compose
cargo test --lib scanner::tests::test_check_shell
```

## Common Issues and Fixes

### Issue: Secret detected in config file
```yaml
# ❌ Bad
database:
  password: "mypassword123"

# ✅ Good
database:
  password: ${DB_PASSWORD}
```

### Issue: Latest tag in Dockerfile
```dockerfile
# ❌ Bad
FROM python:latest

# ✅ Good
FROM python:3.11-slim
```

### Issue: Privileged container
```yaml
# ❌ Bad
services:
  app:
    privileged: true

# ✅ Good
services:
  app:
    cap_add:
      - NET_ADMIN
```

### Issue: Unsafe shell script
```bash
# ❌ Bad
#!/bin/bash
rm -rf $BUILD_DIR

# ✅ Good
#!/bin/bash
set -euo pipefail
BUILD_DIR="${BUILD_DIR:-/tmp/build}"
if [[ -n "$BUILD_DIR" && -d "$BUILD_DIR" ]]; then
  rm -rf "$BUILD_DIR"
fi
```

## See Also
- `INFRASTRUCTURE_CATEGORY_UPDATE.md` - Full technical details
- `LLM_AUDIT_GUIDE.md` - Using LLM for deeper analysis
- `QUICK_REFERENCE.md` - Overall audit system guide