//! Full-pipeline scan of the `actions` repo (nuniesmith/actions).
//!
//! Demonstrates:
//!   1. Static pre-filter (skip / minimal / standard / deep-dive triage)
//!   2. Code chunking with dedup index
//!   3. Prompt routing & token estimation
//!   4. TODO/FIXME scanning with priority classification
//!   5. Savings report
//!
//! Run with:
//!   cargo run --example scan_actions_repo
//!
//! The repo must already be cloned to `repos/actions/` (relative to the
//! project root).  If it isn't there yet:
//!   git clone https://github.com/nuniesmith/actions.git repos/actions

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rustassistant::code_chunker::{CodeChunker, DedupIndex};
use rustassistant::prompt_router::PromptRouter;
use rustassistant::static_analysis::{
    analyze_batch, AnalysisRecommendation, StaticAnalyzer, StaticAnalyzerConfig,
};
use rustassistant::todo_scanner::TodoScanner;

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

/// Recursively collect every file under `dir`, respecting a basic ignore list.
fn collect_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_files_inner(dir, &mut out);
    out.sort();
    out
}

fn collect_files_inner(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();

        // Skip hidden dirs other than .github, and skip target/node_modules
        if path.is_dir() {
            if name == ".git" || name == "target" || name == "node_modules" {
                continue;
            }
            collect_files_inner(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// Try to read a file as UTF-8, returning None for binary files.
fn read_text(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    String::from_utf8(bytes).ok()
}

// ────────────────────────────────────────────────────────────────────────────
// Main
// ────────────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();

    // Locate repo
    let repo_root = PathBuf::from("repos/actions");
    if !repo_root.exists() {
        eprintln!("❌  Repo not found at repos/actions — clone it first:");
        eprintln!("    git clone https://github.com/nuniesmith/actions.git repos/actions");
        std::process::exit(1);
    }

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║         Rustassistant · Full Pipeline Scan · actions repo           ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    // ── 1. Collect files ────────────────────────────────────────────────────
    let files = collect_files(&repo_root);
    println!("📁  Files discovered: {}\n", files.len());

    // Load file contents (skip binary)
    let mut file_contents: Vec<(String, String)> = Vec::new();
    let mut binary_skipped = 0usize;
    for f in &files {
        let rel = f.strip_prefix(&repo_root).unwrap_or(f);
        let rel_str = rel.to_string_lossy().to_string();
        match read_text(f) {
            Some(content) => file_contents.push((rel_str, content)),
            None => binary_skipped += 1,
        }
    }
    println!(
        "📄  Text files loaded: {} (binary skipped: {})\n",
        file_contents.len(),
        binary_skipped
    );

    // ── 2. Static pre-filter ────────────────────────────────────────────────
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔬  Stage 1: Static Analysis Pre-Filter");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let analyzer = StaticAnalyzer::new();
    let batch = analyze_batch(&analyzer, &file_contents);

    println!("  Total files analysed : {}", batch.total_files);
    println!("  ┌────────────────────────────────────────┐");
    println!(
        "  │  Skip      : {:>4}  ({:>5.1}%)              │",
        batch.skip_count,
        (batch.skip_count as f64 / batch.total_files.max(1) as f64) * 100.0
    );
    println!(
        "  │  Minimal   : {:>4}  ({:>5.1}%)              │",
        batch.minimal_count,
        (batch.minimal_count as f64 / batch.total_files.max(1) as f64) * 100.0
    );
    println!(
        "  │  Standard  : {:>4}  ({:>5.1}%)              │",
        batch.standard_count,
        (batch.standard_count as f64 / batch.total_files.max(1) as f64) * 100.0
    );
    println!(
        "  │  Deep Dive : {:>4}  ({:>5.1}%)              │",
        batch.deep_dive_count,
        (batch.deep_dive_count as f64 / batch.total_files.max(1) as f64) * 100.0
    );
    println!("  └────────────────────────────────────────┘");
    println!(
        "  Estimated LLM savings : {:.1}%",
        batch.estimated_savings_percent
    );
    println!("  Static issues found   : {}", batch.total_static_issues);

    if !batch.skip_reasons.is_empty() {
        println!("\n  Skip reasons:");
        let mut reasons: Vec<_> = batch.skip_reasons.iter().collect();
        reasons.sort_by(|a, b| b.1.cmp(a.1));
        for (reason, count) in reasons {
            println!("    • {} (×{})", reason, count);
        }
    }

    // Per-file detail table
    println!("\n  ┌─────────────────────────────────────────────────────────────────────┐");
    println!("  │  File                                       │ Rec       │ Issues    │");
    println!("  ├─────────────────────────────────────────────┼───────────┼───────────┤");
    for r in &batch.results {
        let short = if r.file_path.len() > 44 {
            format!("…{}", &r.file_path[r.file_path.len() - 43..])
        } else {
            format!("{:<44}", r.file_path)
        };
        let rec_str = match r.recommendation {
            AnalysisRecommendation::Skip => "Skip     ",
            AnalysisRecommendation::Minimal => "Minimal  ",
            AnalysisRecommendation::Standard => "Standard ",
            AnalysisRecommendation::DeepDive => "DeepDive ",
        };
        println!(
            "  │  {} │ {} │ {:>3}       │",
            short, rec_str, r.static_issue_count
        );
    }
    println!("  └─────────────────────────────────────────────┴───────────┴───────────┘");

    // ── 3. Prompt Routing ───────────────────────────────────────────────────
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎯  Stage 2: Prompt Router – Tier Assignment & Token Estimates");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let router = PromptRouter::new();
    let mut tier_tokens: HashMap<String, u64> = HashMap::new();
    let mut tier_counts: HashMap<String, usize> = HashMap::new();

    for r in &batch.results {
        if matches!(r.recommendation, AnalysisRecommendation::Skip) {
            continue;
        }

        let content = file_contents
            .iter()
            .find(|(p, _)| p == &r.file_path)
            .map(|(_, c)| c.as_str())
            .unwrap_or("");

        // Use the router to build the actual prompt tier and get the token estimate
        let prompt_tier = router.route(&r.file_path, content, r);

        let label = format!("{:?}", prompt_tier.tier);
        *tier_tokens.entry(label.clone()).or_default() += prompt_tier.estimated_input_tokens as u64;
        *tier_counts.entry(label).or_default() += 1;
    }

    let total_tokens: u64 = tier_tokens.values().sum();
    println!("  Tier          │ Files │ Est. Tokens");
    println!("  ──────────────┼───────┼────────────");
    for tier_name in &["Minimal", "Standard", "DeepDive"] {
        let count = tier_counts.get(*tier_name).copied().unwrap_or(0);
        let tokens = tier_tokens.get(*tier_name).copied().unwrap_or(0);
        println!("  {:<14} │ {:>5} │ {:>10}", tier_name, count, tokens);
    }
    println!("  ──────────────┼───────┼────────────");
    let total_count: usize = tier_counts.values().sum();
    println!(
        "  {:<14} │ {:>5} │ {:>10}",
        "TOTAL", total_count, total_tokens
    );

    let skipped_files = batch.skip_count;
    let naive_tokens: u64 = file_contents
        .iter()
        .map(|(_, c)| c.len() as u64 / 4) // rough ~4 chars per token
        .sum();
    let saved_tokens = naive_tokens.saturating_sub(total_tokens);
    println!(
        "\n  📉  Naïve (all-files) token estimate : ~{}",
        naive_tokens
    );
    println!("  📉  After static triage              : ~{}", total_tokens);
    println!(
        "  💰  Tokens saved                     : ~{} ({:.1}%)",
        saved_tokens,
        (saved_tokens as f64 / naive_tokens.max(1) as f64) * 100.0
    );
    println!("  🚫  Files skipped entirely           : {}", skipped_files);

    // ── 4. Code Chunking + Dedup ────────────────────────────────────────────
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧩  Stage 3: Code Chunking & Content-Addressable Dedup");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let chunker = CodeChunker::new();
    let mut dedup = DedupIndex::new();
    let mut total_chunks = 0usize;
    let mut unique_chunks = 0usize;
    let mut dup_chunks = 0usize;
    let mut chunks_by_ext: HashMap<String, usize> = HashMap::new();
    let mut largest_file_chunks: (String, usize) = (String::new(), 0);

    let repo_id = "actions";

    for (rel_path, content) in &file_contents {
        let chunks = chunker.chunk_file(rel_path, content, repo_id);
        total_chunks += chunks.len();

        if chunks.len() > largest_file_chunks.1 {
            largest_file_chunks = (rel_path.clone(), chunks.len());
        }

        let ext = Path::new(rel_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("(none)")
            .to_string();
        *chunks_by_ext.entry(ext).or_default() += chunks.len();

        for chunk in &chunks {
            if dedup.contains(&chunk.content_hash) {
                dup_chunks += 1;
            } else {
                unique_chunks += 1;
                dedup.insert_or_link(chunk);
            }
        }
    }

    println!("  Total chunks produced  : {}", total_chunks);
    println!("  Unique chunks          : {}", unique_chunks);
    println!(
        "  Duplicate chunks       : {} ({:.1}% dedup rate)",
        dup_chunks,
        (dup_chunks as f64 / total_chunks.max(1) as f64) * 100.0
    );
    println!(
        "  Most chunked file      : {} ({} chunks)",
        largest_file_chunks.0, largest_file_chunks.1
    );
    println!("\n  Chunks by file type:");
    let mut ext_vec: Vec<_> = chunks_by_ext.iter().collect();
    ext_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (ext, count) in ext_vec {
        println!("    .{:<12} {:>4} chunks", ext, count);
    }

    // ── 5. TODO / FIXME Scanning ────────────────────────────────────────────
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📝  Stage 4: TODO / FIXME / HACK Scanner");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let todo_scanner = TodoScanner::new()?;
    let todo_items = todo_scanner.scan_directory(&repo_root)?;
    let todo_summary = todo_scanner.generate_summary(&todo_items);

    println!("  Total TODOs found : {}", todo_summary.total);
    println!("  Files with TODOs  : {}", todo_summary.files_with_todos);
    println!("  High priority     : {}", todo_summary.high_priority);
    println!("  Medium priority   : {}", todo_summary.medium_priority);
    println!("  Low priority      : {}", todo_summary.low_priority);

    // Group by category
    if !todo_summary.by_category.is_empty() {
        println!("\n  By category:");
        let mut cats: Vec<_> = todo_summary.by_category.iter().collect();
        cats.sort_by(|a, b| b.1.cmp(a.1));
        for (cat, count) in cats {
            println!("    {:?}: {}", cat, count);
        }
    }

    // Show first few TODOs
    let show_limit = 15;
    if !todo_items.is_empty() {
        println!("\n  Sample TODOs (up to {}):", show_limit);
        for item in todo_items.iter().take(show_limit) {
            let file_str = item.file.to_string_lossy();
            let short_file = if file_str.len() > 35 {
                format!("…{}", &file_str[file_str.len() - 34..])
            } else {
                file_str.to_string()
            };
            println!(
                "    [{:?}] {}:{} — {}",
                item.category,
                short_file,
                item.line,
                if item.text.len() > 55 {
                    format!("{}…", &item.text[..55])
                } else {
                    item.text.clone()
                }
            );
        }
    }

    // ── 6. Integrated analysis with TODOs ───────────────────────────────────
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔗  Stage 5: Static Analysis + TODO Integration (analyze_with_todos)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let config = StaticAnalyzerConfig::default();
    let rich_analyzer = StaticAnalyzer::with_config(config);
    let mut upgraded_count = 0usize;

    for (rel_path, content) in &file_contents {
        let base = rich_analyzer.analyze(rel_path, content);
        let with_todos = rich_analyzer.analyze_with_todos(rel_path, content, &todo_scanner);

        // Check if TODO integration changed the recommendation
        if with_todos.recommendation != base.recommendation {
            upgraded_count += 1;
            println!(
                "  ⬆  {} : {:?} → {:?}",
                rel_path, base.recommendation, with_todos.recommendation
            );
        }
    }

    if upgraded_count == 0 {
        println!("  (no recommendation changes from TODO integration)");
    } else {
        println!(
            "\n  {} file(s) had recommendation upgraded due to TODO density.",
            upgraded_count
        );
    }

    // ── Summary ─────────────────────────────────────────────────────────────
    let elapsed = started.elapsed();
    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                         Scan Summary                               ║");
    println!("╠══════════════════════════════════════════════════════════════════════╣");
    println!("║  Repo              : nuniesmith/actions                            ║");
    println!("║  Files scanned     : {:<47}║", file_contents.len());
    println!("║  Binary skipped    : {:<47}║", binary_skipped);
    println!(
        "║  LLM-skippable     : {:<47}║",
        format!(
            "{} ({:.0}%)",
            batch.skip_count, batch.estimated_savings_percent
        )
    );
    println!(
        "║  Chunks produced   : {:<47}║",
        format!("{} unique, {} dupes", unique_chunks, dup_chunks)
    );
    println!("║  TODOs found       : {:<47}║", todo_summary.total);
    println!(
        "║  Token savings est.: {:<47}║",
        format!(
            "~{} saved ({:.1}%)",
            saved_tokens,
            (saved_tokens as f64 / naive_tokens.max(1) as f64) * 100.0
        )
    );
    println!("║  Elapsed           : {:<47}║", format!("{:.2?}", elapsed));
    println!("╚══════════════════════════════════════════════════════════════════════╝");

    Ok(())
}
