# JANUS Audit Quick Reference

> **One-page cheat sheet for auditing JANUS code**

---

## 🎯 Quick Links

- **Technical Paper**: https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex
- **Full Context**: [JANUS_CONTEXT.md](./JANUS_CONTEXT.md)
- **Paper Integration**: [TECHNICAL_PAPER_INTEGRATION.md](./TECHNICAL_PAPER_INTEGRATION.md)
- **LLM Audit Guide**: [LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md)

---

## 🧠 JANUS Architecture (30 seconds)

| Service | Function | Latency | Location |
|---------|----------|---------|----------|
| **Forward** | Real-time execution | < 10ms | `services/forward/` |
| **Backward** | Memory consolidation | Batch | `services/backward/` |
| **CNS** | Health monitoring | < 100ms | `services/cns/` |

**Brain Regions**: Visual Cortex, Hippocampus, Prefrontal Cortex, Basal Ganglia, Cerebellum, Amygdala

---

## 🔢 Critical Formulas (Must Verify!)

### GAF Normalization (Part 2, Eq. 1.1.1)
```rust
// ✅ CORRECT: Learnable with tanh
x_norm = (gamma * (x - mu) / sigma + beta).tanh();

// ❌ WRONG: Fixed min-max
x_norm = (x - min) / (max - min) * 2.0 - 1.0;
```

### LTN T-norms (Part 2, Eq. 2.3.1-2.3.3)
```rust
// ✅ INFERENCE: Łukasiewicz
fn and_inference(a: f64, b: f64) -> f64 {
    (a + b - 1.0).max(0.0)
}

// ✅ TRAINING: Product (for gradients)
fn and_training(a: f64, b: f64) -> f64 {
    a * b
}
```

### PER Priority (Part 3, Eq. 2.2.1)
```rust
// ✅ CORRECT: Include epsilon
priority = td_error.abs() + 1e-6;

// ❌ WRONG: Division by zero risk
priority = td_error.abs();
```

### Circuit Breaker (Part 4, Eq. 2.6.1)
```rust
// ✅ CORRECT: Mahalanobis distance
let d_m = ((state - mu).transpose() * cov_inv * (state - mu)).sqrt();
if d_m > 5.0 { emergency_stop(); }
```

---

## ✅ Quick Audit Checklist

### Mathematical Correctness
- [ ] GAF normalization ensures |x| < 1 (for arccos)
- [ ] LTN uses Łukasiewicz for inference, Product for training
- [ ] PER priorities include epsilon (10⁻⁶)
- [ ] Qdrant vectors are L2-normalized

### Safety & Compliance
- [ ] No hardcoded API keys (use `env::var()`)
- [ ] Circuit breakers properly configured
- [ ] Wash sale: no buy within 30 days of loss-sell
- [ ] Position limits: MAX 10%, DAILY LOSS 2%

### Performance
- [ ] Forward service: No blocking I/O in hot path
- [ ] Memory: Forward < 2GB, Backward < 8GB
- [ ] Async operations use Tokio correctly
- [ ] Pre-allocated buffers for hot loops

### Architecture
- [ ] Brain-region mapping documented
- [ ] Service boundary correct (Forward/Backward/CNS)
- [ ] Hot/cold path separation maintained
- [ ] Zero-copy Arrow IPC for forward→backward

---

## 🏷️ Audit Tags

```rust
// @audit-tag: neuromorphic
// Brain-inspired component

// @audit-tag: forward-path
// Hot path code (<10ms requirement)

// @audit-tag: mathematical
// Implements paper formula (cite equation)

// @audit-freeze
// Never modify (mathematical constants)

// @audit-security: circuit-breaker
// Critical safety mechanism

// @audit-todo: verify-formula Part 2, Eq. 1.1.3
// Needs validation against paper
```

---

## 🚀 Run Quick Audit

### Local CLI
```bash
cd src/audit

# Full audit with LLM
cargo run --release --bin audit-cli -- audit ../janus --llm

# Math-focused audit
cargo run --release --bin audit-cli -- audit ../janus/crates/vision \
  --llm --focus mathematical,performance

# Compliance check
cargo run --release --bin audit-cli -- audit ../janus/crates/logic \
  --llm --focus compliance,trading-safety
```

### GitHub Actions
1. Go to **Actions** → **🤖 LLM Audit**
2. Click **Run workflow**
3. Configure:
   - Provider: `xai` or `google`
   - Depth: `deep`
   - Focus: `mathematical,architecture,neuromorphic`
   - Include Tests: `true`
   - Batch Size: `5`

---

## 🔍 Common Issues

### Issue: `arccos domain error`
**Cause**: |x| ≥ 1 passed to arccos  
**Fix**: Use tanh normalization  
**Reference**: Part 2, Eq. 1.1.1

### Issue: `Wrong t-norm during inference`
**Cause**: Using Product logic instead of Łukasiewicz  
**Fix**: Switch to `max(0, a + b - 1)`  
**Reference**: Part 2, Eq. 2.3.1

### Issue: `Division by zero in PER`
**Cause**: Priority calculated without epsilon  
**Fix**: Add `+ 1e-6` to priority  
**Reference**: Part 3, Eq. 2.2.1

### Issue: `Qdrant similarity wrong`
**Cause**: Non-normalized vectors  
**Fix**: L2-normalize before insertion  
**Reference**: Part 3, Section 4

---

## 📊 Performance Targets

| Metric | Target | Critical? |
|--------|--------|-----------|
| Forward p99 latency | < 10ms | ⚠️ YES |
| Forward throughput | > 10K req/s | ⚠️ YES |
| QuestDB writes | > 100K/sec | YES |
| Qdrant search | < 10ms @ 1M | YES |
| Forward memory | < 2GB RSS | YES |
| Backward memory | < 8GB RSS | NO |

---

## 🛡️ Regulatory Rules

### Wash Sale (IRS)
```rust
// No buy within 30 days of loss-realizing sell
assert!(days_since_loss_sell >= 30);
```

### Position Limits
```rust
const MAX_POSITION_SIZE: f64 = 0.10;  // 10%
const DAILY_LOSS_LIMIT: f64 = 0.02;   // 2%
```

### PDT Rule
```rust
if day_trades_last_5_days >= 4 {
    assert!(account_value >= 25_000.0);
}
```

---

## 📖 Paper Section Guide

| Code Location | Paper Reference |
|---------------|-----------------|
| `crates/vision/` | Part 2, Section 1 (GAF, ViViT) |
| `crates/logic/` | Part 2, Section 2 (LTN) |
| `services/forward/src/fusion.rs` | Part 2, Section 3 (Fusion) |
| `services/forward/src/decision.rs` | Part 2, Section 4 (Basal Ganglia) |
| `crates/memory/src/hippocampus.rs` | Part 3, Section 1 (Hippocampus) |
| `crates/memory/src/per.rs` | Part 3, Section 2.2 (PER) |
| `services/backward/src/umap.rs` | Part 3, Section 3 (UMAP) |
| `crates/memory/src/qdrant.rs` | Part 3, Section 4 (Qdrant) |
| `services/forward/src/amygdala.rs` | Part 4, Section 2.6 (Amygdala) |

---

## 🎓 Issue Categories

- **ARCHITECTURE**: Neuromorphic design violations
- **MATHEMATICS**: Formula implementation errors
- **PERFORMANCE**: Latency/throughput violations
- **SAFETY**: Missing circuit breakers, hardcoded secrets
- **COMPLIANCE**: Regulatory constraint violations
- **LOGIC**: Business logic errors
- **CODE_QUALITY**: Documentation, testing gaps

---

## 💡 Pro Tips

1. **Always cite paper sections** in code comments
2. **Use audit tags** for categorization
3. **Run math audits** before merging formula changes
4. **Deep analysis** for architectural refactors
5. **Check performance** after hot-path modifications
6. **Validate compliance** for execution/risk changes

---

**Quick Start**: Read [JANUS_CONTEXT.md](./JANUS_CONTEXT.md) → Run audit → Fix issues  
**Full Guide**: [LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md)  
**Paper**: https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex