//! Janus Framework Integration Example
//!
//! This example demonstrates how to use the Janus theoretical framework
//! structures in the audit service.

use fks_audit::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    println!("=== Janus Framework Integration Example ===\n");

    // Initialize Janus orchestrator
    let config = JanusConfig {
        hippocampus_capacity: 1000,
        swr_batch_size: 32,
        decision_threshold: 0.5,
        enable_amygdala: true,
        llm_provider: "xai".to_string(),
    };

    let mut orchestrator = JanusOrchestrator::new(config);

    println!("✓ Initialized Janus Orchestrator");
    println!("  - Hippocampus capacity: 1000");
    println!("  - SWR batch size: 32");
    println!("  - Decision threshold: 0.5\n");

    // Define constraints (LTN)
    add_audit_constraints(&mut orchestrator);

    // Simulate forward pass (real-time analysis)
    println!("=== Forward Pass (Real-Time Analysis) ===\n");
    forward_analysis_example(&mut orchestrator);

    // Simulate backward pass (consolidation)
    println!("\n=== Backward Pass (Consolidation) ===\n");
    backward_consolidation_example(&mut orchestrator);

    println!("\n=== Completed ===");
}

fn add_audit_constraints(orchestrator: &mut JanusOrchestrator) {
    println!("Adding LTN constraints:");

    // Constraint 1: Frozen code protection
    orchestrator.add_constraint(LTNConstraint {
        id: "frozen_code".to_string(),
        predicate: "frozen_code".to_string(),
        weight: 10.0,
        satisfaction: None,
        variables: vec!["file_path".to_string(), "tags".to_string()],
    });
    println!("  ✓ frozen_code (weight: 10.0)");

    // Constraint 2: Security critical files
    orchestrator.add_constraint(LTNConstraint {
        id: "security_critical".to_string(),
        predicate: "security".to_string(),
        weight: 5.0,
        satisfaction: None,
        variables: vec!["category".to_string(), "security_score".to_string()],
    });
    println!("  ✓ security_critical (weight: 5.0)");

    // Constraint 3: Type safety
    orchestrator.add_constraint(LTNConstraint {
        id: "type_safety".to_string(),
        predicate: "type_safety".to_string(),
        weight: 3.0,
        satisfaction: None,
        variables: vec!["type_safety".to_string()],
    });
    println!("  ✓ type_safety (weight: 3.0)\n");
}

fn forward_analysis_example(orchestrator: &mut JanusOrchestrator) {
    // Example 1: High-quality code
    println!("1. Analyzing high-quality code:");
    let state1 = create_sample_state(
        "src/core/engine.rs",
        FeatureVector {
            loc: 250,
            complexity: 5.0,
            doc_coverage: 0.95,
            test_coverage: 0.90,
            security_score: 0.98,
            type_safety: 0.95,
            async_safety: 0.92,
            custom_metrics: HashMap::new(),
        },
    );

    let decision1 = orchestrator.forward_pass(state1);
    print_decision_result(&decision1, "engine.rs");

    // Example 2: Low-quality code
    println!("\n2. Analyzing low-quality code:");
    let state2 = create_sample_state(
        "src/utils/helper.rs",
        FeatureVector {
            loc: 500,
            complexity: 15.0,
            doc_coverage: 0.20,
            test_coverage: 0.10,
            security_score: 0.45,
            type_safety: 0.60,
            async_safety: 0.55,
            custom_metrics: HashMap::new(),
        },
    );

    let decision2 = orchestrator.forward_pass(state2);
    print_decision_result(&decision2, "helper.rs");

    // Example 3: Critical file
    println!("\n3. Analyzing critical kill_switch:");
    let state3 = create_sample_state(
        "src/amygdala/kill_switch.rs",
        FeatureVector {
            loc: 150,
            complexity: 3.0,
            doc_coverage: 1.0,
            test_coverage: 1.0,
            security_score: 1.0,
            type_safety: 1.0,
            async_safety: 1.0,
            custom_metrics: HashMap::new(),
        },
    );

    let decision3 = orchestrator.forward_pass(state3);
    print_decision_result(&decision3, "kill_switch.rs");
}

fn backward_consolidation_example(orchestrator: &mut JanusOrchestrator) {
    println!("Running schema consolidation...");

    // Trigger backward pass
    orchestrator.backward_pass();

    // Display memory statistics
    println!("\nMemory Hierarchy Statistics:");
    println!(
        "  - Hippocampus: {} episodes",
        orchestrator.memory.hippocampus.episodes.len()
    );
    println!(
        "  - SWR Buffer: {} prioritized experiences",
        orchestrator.memory.swr_buffer.experiences.len()
    );
    println!(
        "  - Neocortex: {} schemas",
        orchestrator.memory.neocortex.schemas.len()
    );

    // Demonstrate Łukasiewicz logic
    println!("\n=== Łukasiewicz Logic Examples ===");
    demonstrate_lukasiewicz_logic();
}

fn create_sample_state(path: &str, features: FeatureVector) -> ForwardState {
    ForwardState {
        timestamp: chrono::Utc::now(),
        features,
        observation: Observation {
            path: PathBuf::from(path),
            content_hash: "abc123".to_string(),
            ast_summary: Some("function definitions: 5, imports: 3".to_string()),
            dependencies: vec!["tokio".to_string(), "serde".to_string()],
        },
        context: ContextMetadata {
            repository: "fks".to_string(),
            branch: "main".to_string(),
            commit: Some("abc123def".to_string()),
            ci_context: None,
            volatility: 0.5,
            spread: 0.1,
            volume: 100,
        },
    }
}

fn print_decision_result(decision: &DualPathwayDecision, filename: &str) {
    println!("  File: {}", filename);
    println!("  Direct pathway:   {:.3}", decision.direct_pathway);
    println!("  Indirect pathway: {:.3}", decision.indirect_pathway);
    println!("  Action score:     {:.3}", decision.action_score);
    println!(
        "  Decision: {}",
        if decision.should_act() {
            "✓ GENERATE TASK"
        } else {
            "○ Skip"
        }
    );
}

fn demonstrate_lukasiewicz_logic() {
    println!("  Conjunction (AND):");
    println!("    0.8 ∧ 0.7 = {:.1}", LukasiewiczLogic::and(0.8, 0.7));
    println!("    0.3 ∧ 0.4 = {:.1}", LukasiewiczLogic::and(0.3, 0.4));

    println!("\n  Disjunction (OR):");
    println!("    0.3 ∨ 0.4 = {:.1}", LukasiewiczLogic::or(0.3, 0.4));
    println!("    0.8 ∨ 0.7 = {:.1}", LukasiewiczLogic::or(0.8, 0.7));

    println!("\n  Negation (NOT):");
    println!("    ¬0.6 = {:.1}", LukasiewiczLogic::not(0.6));
    println!("    ¬0.2 = {:.1}", LukasiewiczLogic::not(0.2));

    println!("\n  Implication (IF-THEN):");
    println!("    0.8 ⇒ 0.5 = {:.1}", LukasiewiczLogic::implies(0.8, 0.5));
    println!("    0.2 ⇒ 0.9 = {:.1}", LukasiewiczLogic::implies(0.2, 0.9));

    println!("\n  Constraint Example:");
    println!("    Rule: IsCritical(f) ⇒ SecurityScore(f) ≥ 0.9");

    let is_critical = 1.0;
    let security_low = 0.5;
    let security_high = 0.95;

    let violation = LukasiewiczLogic::implies(is_critical, security_low);
    let satisfied = LukasiewiczLogic::implies(is_critical, security_high);

    println!(
        "    Critical file with low security:  {:.1} (VIOLATION)",
        violation
    );
    println!(
        "    Critical file with high security: {:.1} (SATISFIED)",
        satisfied
    );
}
