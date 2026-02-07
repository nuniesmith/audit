# JANUS Audit Documentation Index

> **Complete guide to navigating JANUS audit documentation**

---

## 📚 Documentation Map

### 🎯 Quick Navigation

| I Want To... | Read This |
|--------------|-----------|
| **Get started quickly** | [QUICK_START.md](./QUICK_START.md) |
| **Understand JANUS** | [JANUS_CONTEXT.md](./JANUS_CONTEXT.md) |
| **Run an LLM audit** | [LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md) |
| **Verify a formula** | [TECHNICAL_PAPER_INTEGRATION.md](./TECHNICAL_PAPER_INTEGRATION.md) |
| **Quick reference** | [JANUS_AUDIT_QUICK_REF.md](./JANUS_AUDIT_QUICK_REF.md) |
| **Customize LLM prompts** | [JANUS_LLM_SYSTEM_PROMPT.md](./JANUS_LLM_SYSTEM_PROMPT.md) |
| **See what changed** | [AUDIT_UPDATE_SUMMARY.md](./AUDIT_UPDATE_SUMMARY.md) |

---

## 📖 Reading Paths

### Path 1: New to JANUS & Audit System (Complete Journey)

**Time**: ~2 hours

1. **[README.md](./README.md)** _(15 min)_
   - What is the audit service?
   - Core features and architecture
   - Installation and setup

2. **[JANUS_CONTEXT.md](./JANUS_CONTEXT.md)** _(30 min)_
   - Project JANUS overview
   - Brain-inspired architecture
   - Mathematical specifications
   - Performance requirements

3. **[QUICK_START.md](./QUICK_START.md)** _(10 min)_
   - Run your first audit
   - Hands-on examples
   - Common commands

4. **[LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md)** _(45 min)_
   - Complete LLM audit workflow
   - GitHub Actions integration
   - Configuration options
   - Examples and troubleshooting

5. **[JANUS_AUDIT_QUICK_REF.md](./JANUS_AUDIT_QUICK_REF.md)** _(5 min)_
   - Keep this handy for daily work
   - Critical formulas
   - Quick checklists

---

### Path 2: Technical Reviewer (Focus on Verification)

**Time**: ~1.5 hours

1. **[JANUS_CONTEXT.md](./JANUS_CONTEXT.md)** _(30 min)_
   - Mathematical specifications
   - Brain-region mappings
   - Audit checklists

2. **[TECHNICAL_PAPER_INTEGRATION.md](./TECHNICAL_PAPER_INTEGRATION.md)** _(45 min)_
   - Equation-to-code mapping
   - Verification patterns
   - Common error examples
   - Issue report format

3. **[JANUS_AUDIT_QUICK_REF.md](./JANUS_AUDIT_QUICK_REF.md)** _(5 min)_
   - Critical formulas
   - Performance targets
   - Paper section guide

4. **[JANUS Technical Paper](https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex)** _(ongoing reference)_
   - Original mathematical specifications
   - Architecture rationale

---

### Path 3: LLM Integration Developer (Prompt Engineering)

**Time**: ~1 hour

1. **[JANUS_LLM_SYSTEM_PROMPT.md](./JANUS_LLM_SYSTEM_PROMPT.md)** _(30 min)_
   - Production system prompt
   - JANUS context for AI
   - Customization examples

2. **[LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md)** _(20 min)_
   - Provider configuration (XAI, Google)
   - API integration
   - Best practices

3. **[src/llm.rs](./src/llm.rs)** _(10 min)_
   - Prompt implementation
   - Response parsing
   - Provider abstraction

---

### Path 4: Quick Start for Experienced Users

**Time**: ~15 minutes

1. **[JANUS_AUDIT_QUICK_REF.md](./JANUS_AUDIT_QUICK_REF.md)** _(5 min)_
   - Essential formulas
   - Quick commands
   - Common issues

2. **[QUICK_REFERENCE.md](./QUICK_REFERENCE.md)** _(5 min)_
   - CLI commands
   - API endpoints
   - Configuration

3. **[LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md)** - Section 11 _(5 min)_
   - JANUS-specific examples
   - Ready-to-use commands

---

## 📁 File Reference

### Core Documentation (Must Read)

| File | Lines | Purpose | Audience |
|------|-------|---------|----------|
| **[README.md](./README.md)** | 620 | Service overview | Everyone |
| **[JANUS_CONTEXT.md](./JANUS_CONTEXT.md)** | 633 | Technical reference | Reviewers, LLMs |
| **[LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md)** | 750+ | Complete workflow | Audit users |
| **[JANUS_AUDIT_QUICK_REF.md](./JANUS_AUDIT_QUICK_REF.md)** | 253 | Cheat sheet | Developers |

### Technical Integration (For Reviewers)

| File | Lines | Purpose | Audience |
|------|-------|---------|----------|
| **[TECHNICAL_PAPER_INTEGRATION.md](./TECHNICAL_PAPER_INTEGRATION.md)** | 533 | Paper verification | Technical reviewers |
| **[JANUS_LLM_SYSTEM_PROMPT.md](./JANUS_LLM_SYSTEM_PROMPT.md)** | 369 | LLM prompt template | AI integration devs |
| **[AUDIT_UPDATE_SUMMARY.md](./AUDIT_UPDATE_SUMMARY.md)** | 532 | v2.0 changes | Maintainers |

### Quick Reference (Keep Handy)

| File | Lines | Purpose | Audience |
|------|-------|---------|----------|
| **[QUICK_START.md](./QUICK_START.md)** | ~100 | Installation & first run | New users |
| **[QUICK_REFERENCE.md](./QUICK_REFERENCE.md)** | ~150 | Command cheatsheet | All users |
| **[JANUS_AUDIT_INDEX.md](./JANUS_AUDIT_INDEX.md)** | ~350 | This file - navigation | Everyone |

### Supporting Documentation

| File | Lines | Purpose | Audience |
|------|-------|---------|----------|
| **[WHY_TESTS_AND_ALL_AREAS.md](./WHY_TESTS_AND_ALL_AREAS.md)** | ~100 | Testing philosophy | Contributors |
| **[MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)** | ~150 | Service migration | DevOps |

---

## 🎯 Use Case Index

### "I need to audit a mathematical component"

1. Read: [TECHNICAL_PAPER_INTEGRATION.md](./TECHNICAL_PAPER_INTEGRATION.md)
2. Find: Equation reference in paper (Part X, Section Y, Eq. Z)
3. Check: [JANUS_CONTEXT.md](./JANUS_CONTEXT.md) - Mathematical Specifications
4. Verify: Compare code against formula
5. Tag: Use `@audit-tag: mathematical` with paper reference

**Example Command**:
```bash
cargo run --bin audit-cli -- audit ../janus/crates/vision \
  --llm --focus mathematical,performance --batch-size 5
```

---

### "I need to verify brain-region architecture"

1. Read: [JANUS_CONTEXT.md](./JANUS_CONTEXT.md) - Brain-Region Mapping
2. Check: Component location matches architecture
3. Verify: Service boundary (Forward/Backward/CNS)
4. Validate: Performance characteristics (hot/cold path)

**Example Command**:
```bash
cargo run --bin audit-cli -- audit ../janus \
  --llm --focus architecture,neuromorphic --exclude-tests
```

---

### "I need to check regulatory compliance"

1. Read: [JANUS_CONTEXT.md](./JANUS_CONTEXT.md) - Safety & Compliance
2. Check: Wash sale rules, position limits, PDT
3. Verify: Circuit breaker implementations
4. Validate: Audit logging and data retention

**Example Command**:
```bash
cargo run --bin audit-cli -- audit ../janus/crates/execution \
  --llm --focus compliance,trading-safety,security
```

---

### "I need to run a comprehensive audit before release"

1. Read: [LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md) - Section 11
2. Configure: Deep analysis, all focus areas, include tests
3. Run: Via GitHub Actions for artifact retention
4. Review: All Critical and High severity issues
5. Document: Fixes with paper references

**GitHub Actions**:
```yaml
Provider: xai
Depth: deep
Focus Areas: security,logic,compliance,performance,architecture,mathematical,neuromorphic
Include Tests: true
Batch Size: 5
```

---

### "I need to customize the LLM system prompt"

1. Read: [JANUS_LLM_SYSTEM_PROMPT.md](./JANUS_LLM_SYSTEM_PROMPT.md)
2. Copy: Base prompt template
3. Customize: Add specific focus (e.g., "Extra attention to vision crate")
4. Test: Run on small codebase subset
5. Measure: False positive rate, issue quality

**Customization Example**:
```
# ADDITIONAL FOCUS: Vision Crate Mathematical Validation

For this audit, pay EXTRA attention to:
- GAF normalization (Part 2, Eq. 1.1.1)
- Polar transformation domain checking
- ViViT attention weight normalization
- Numerical stability in all trigonometric operations
```

---

## 🔄 Documentation Lifecycle

### When to Read

- **Before coding**: [JANUS_CONTEXT.md](./JANUS_CONTEXT.md) + Paper
- **While coding**: [JANUS_AUDIT_QUICK_REF.md](./JANUS_AUDIT_QUICK_REF.md)
- **Before PR**: [TECHNICAL_PAPER_INTEGRATION.md](./TECHNICAL_PAPER_INTEGRATION.md)
- **During review**: [LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md)
- **Before release**: All Critical checks from [JANUS_CONTEXT.md](./JANUS_CONTEXT.md)

### When to Update

- **Paper changes**: Update all mathematical references
- **Architecture refactor**: Update brain-region mappings
- **New requirements**: Update checklists and constraints
- **Performance changes**: Update target metrics
- **LLM improvements**: Update system prompts

---

## 📊 Documentation Statistics

### Coverage

- **Total Documentation**: ~3,500 lines across 12 files
- **Mathematical Specifications**: 5 core algorithms documented
- **Brain Regions Mapped**: 6 components
- **Audit Checklists**: 5 categories, 40+ items
- **Code Examples**: 50+ snippets
- **Paper References**: 20+ equation citations

### Quality Metrics

- **Completeness**: ✅ All JANUS components documented
- **Accuracy**: ✅ Verified against technical paper
- **Usability**: ✅ Multiple reading paths provided
- **Maintainability**: ✅ Clear update guidelines

---

## 🆘 Getting Help

### Documentation Issues

- **Missing content**: Open GitHub issue with `documentation` label
- **Unclear section**: Ask in GitHub Discussions
- **Technical questions**: Reference paper + context docs

### Audit Issues

- **False positives**: See [LLM_AUDIT_GUIDE.md](./LLM_AUDIT_GUIDE.md) - Troubleshooting
- **Missing issues**: Increase analysis depth, reduce batch size
- **Performance problems**: Check [QUICK_REFERENCE.md](./QUICK_REFERENCE.md)

---

## 🎓 Learning Resources

### External References

1. **JANUS Technical Paper** (Primary source)
   - https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex

2. **Gramian Angular Fields**
   - Paper: https://arxiv.org/abs/1506.00327
   - See: [JANUS_CONTEXT.md](./JANUS_CONTEXT.md) - Section 4.1

3. **Logic Tensor Networks**
   - Paper: https://arxiv.org/abs/2012.13635
   - See: [JANUS_CONTEXT.md](./JANUS_CONTEXT.md) - Section 4.2

4. **Prioritized Experience Replay**
   - Paper: https://arxiv.org/abs/1511.05952
   - See: [JANUS_CONTEXT.md](./JANUS_CONTEXT.md) - Section 4.3

5. **UMAP**
   - Paper: https://arxiv.org/abs/1802.03426
   - See: [JANUS_CONTEXT.md](./JANUS_CONTEXT.md) - Section 4.5

### Internal Resources

- [JANUS Project README](../../janus/README.md)
- [CNS Architecture](../../janus/docs/CNS_ARCHITECTURE.md)
- [FKS Main README](../../../README.md)

---

## ✅ Documentation Checklist

Before submitting a PR that adds/modifies JANUS code:

- [ ] Read relevant sections of [JANUS_CONTEXT.md](./JANUS_CONTEXT.md)
- [ ] Verify formulas against [TECHNICAL_PAPER_INTEGRATION.md](./TECHNICAL_PAPER_INTEGRATION.md)
- [ ] Add appropriate audit tags (`@audit-tag`, `@audit-freeze`, etc.)
- [ ] Reference paper equations in code comments
- [ ] Run targeted LLM audit on changed files
- [ ] Address all Critical and High severity issues
- [ ] Update documentation if architecture/formulas changed

---

**Quick Start**: Read [QUICK_START.md](./QUICK_START.md) → Run first audit → Bookmark [JANUS_AUDIT_QUICK_REF.md](./JANUS_AUDIT_QUICK_REF.md)  
**Full Learning**: Follow "Path 1: New to JANUS & Audit System" above  
**Daily Reference**: Keep [JANUS_AUDIT_QUICK_REF.md](./JANUS_AUDIT_QUICK_REF.md) open

---

**Last Updated**: 2025-01-XX  
**Version**: 2.0  
**Maintainer**: Jordan Smith