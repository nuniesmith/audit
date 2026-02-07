//! Neuromorphic Architecture Mapper
//!
//! Generates Mermaid flowcharts of the JANUS neuromorphic trading system.
//! Maps code modules to biological brain regions and visualizes the "connectome"
//! of data flow and decision pathways.

use crate::error::{AuditError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

/// Biological brain regions used as metaphor for system architecture
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrainRegion {
    /// Sensory input processing (Thalamus, DSP)
    Sensory,
    /// Memory and spatial reasoning (Hippocampus)
    Memory,
    /// Executive control and planning (Cortex, PFC)
    Executive,
    /// Emotional/risk regulation (Amygdala, Hypothalamus)
    Limbic,
    /// Action selection and execution (Basal Ganglia)
    Action,
    /// Motor output and visualization (Execution, Clients)
    Output,
    /// Meta-cognition and learning (Audit)
    Meta,
}

impl BrainRegion {
    /// Get CSS color for this brain region
    pub fn color(&self) -> &'static str {
        match self {
            BrainRegion::Sensory => "#f9f",
            BrainRegion::Memory => "#bfb",
            BrainRegion::Executive => "#bbf",
            BrainRegion::Limbic => "#fbb",
            BrainRegion::Action => "#ddd",
            BrainRegion::Output => "#fb9",
            BrainRegion::Meta => "#eee",
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            BrainRegion::Sensory => "Sensory",
            BrainRegion::Memory => "Memory",
            BrainRegion::Executive => "Executive",
            BrainRegion::Limbic => "Limbic",
            BrainRegion::Action => "Action",
            BrainRegion::Output => "Output",
            BrainRegion::Meta => "Meta",
        }
    }
}

/// Configuration for a biological/functional module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    /// Unique identifier for the node
    pub id: String,
    /// Human-readable label
    pub label: String,
    /// Biological region classification
    pub region: BrainRegion,
    /// File path pattern to detect this module
    pub path_pattern: String,
}

/// A connection (synapse) between two modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Source module ID
    pub from: String,
    /// Target module ID
    pub to: String,
    /// Description of the data/signal flow
    pub label: String,
}

/// The complete neuromorphic architecture map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuromorphicMap {
    /// All module configurations
    pub modules: Vec<ModuleConfig>,
    /// All connections between modules
    pub connections: Vec<Connection>,
    /// Modules detected in the scanned codebase
    pub detected_modules: HashSet<String>,
}

impl NeuromorphicMap {
    /// Create a new neuromorphic map with default JANUS architecture
    pub fn new() -> Self {
        let modules = vec![
            // --- SENSORY SYSTEM ---
            ModuleConfig {
                id: "SENSORY".to_string(),
                label: "External Data Feeds".to_string(),
                region: BrainRegion::Sensory,
                path_pattern: "src/data".to_string(),
            },
            ModuleConfig {
                id: "DSP".to_string(),
                label: "Signal Processing (DSP)".to_string(),
                region: BrainRegion::Sensory,
                path_pattern: "src/dsp".to_string(),
            },
            ModuleConfig {
                id: "THALAMUS".to_string(),
                label: "Thalamus (TRN Gating)".to_string(),
                region: BrainRegion::Sensory,
                path_pattern: "neuromorphic/thalamus".to_string(),
            },
            // --- EXECUTIVE & PLANNING ---
            ModuleConfig {
                id: "CORTEX".to_string(),
                label: "Cortex (Strategy/Planning)".to_string(),
                region: BrainRegion::Executive,
                path_pattern: "neuromorphic/cortex".to_string(),
            },
            ModuleConfig {
                id: "PFC".to_string(),
                label: "Prefrontal (Working Memory)".to_string(),
                region: BrainRegion::Executive,
                path_pattern: "neuromorphic/prefrontal".to_string(),
            },
            ModuleConfig {
                id: "CEREBELLUM".to_string(),
                label: "Cerebellum (Error Correction)".to_string(),
                region: BrainRegion::Executive,
                path_pattern: "neuromorphic/cerebellum".to_string(),
            },
            // --- MEMORY & SPACE-TIME ---
            ModuleConfig {
                id: "HIPPOCAMPUS".to_string(),
                label: "Hippocampus (Time Cells/SR)".to_string(),
                region: BrainRegion::Memory,
                path_pattern: "neuromorphic/hippocampus".to_string(),
            },
            // --- LIMBIC / RISK SYSTEM ---
            ModuleConfig {
                id: "AMYGDALA".to_string(),
                label: "Amygdala (Fear/Risk Veto)".to_string(),
                region: BrainRegion::Limbic,
                path_pattern: "neuromorphic/amygdala".to_string(),
            },
            ModuleConfig {
                id: "HYPOTHALAMUS".to_string(),
                label: "Hypothalamus (Homeostasis/Kelly)".to_string(),
                region: BrainRegion::Limbic,
                path_pattern: "neuromorphic/hypothalamus".to_string(),
            },
            // --- ACTION SELECTION ---
            ModuleConfig {
                id: "BG".to_string(),
                label: "Basal Ganglia (Go/NoGo)".to_string(),
                region: BrainRegion::Action,
                path_pattern: "neuromorphic/basal_ganglia".to_string(),
            },
            // --- MOTOR / OUTPUT ---
            ModuleConfig {
                id: "MOTOR".to_string(),
                label: "Motor Cortex (Execution)".to_string(),
                region: BrainRegion::Output,
                path_pattern: "src/execution".to_string(),
            },
            ModuleConfig {
                id: "CLIENTS".to_string(),
                label: "Frontend (Visualization)".to_string(),
                region: BrainRegion::Output,
                path_pattern: "src/clients".to_string(),
            },
            // --- META / EXTERNAL ---
            ModuleConfig {
                id: "AUDIT".to_string(),
                label: "LLM Audit (Meta-Cognition)".to_string(),
                region: BrainRegion::Meta,
                path_pattern: "src/audit".to_string(),
            },
        ];

        let connections = vec![
            // Sensory Flow
            Connection {
                from: "SENSORY".to_string(),
                to: "DSP".to_string(),
                label: "Raw Data".to_string(),
            },
            Connection {
                from: "DSP".to_string(),
                to: "THALAMUS".to_string(),
                label: "Cleaned Signal".to_string(),
            },
            Connection {
                from: "THALAMUS".to_string(),
                to: "HIPPOCAMPUS".to_string(),
                label: "State Encoding (Grid Cells)".to_string(),
            },
            Connection {
                from: "THALAMUS".to_string(),
                to: "AMYGDALA".to_string(),
                label: "Fast Threat Detection".to_string(),
            },
            Connection {
                from: "THALAMUS".to_string(),
                to: "CORTEX".to_string(),
                label: "Bursts/Attention".to_string(),
            },
            // Memory & Context
            Connection {
                from: "HIPPOCAMPUS".to_string(),
                to: "PFC".to_string(),
                label: "Successor Representation".to_string(),
            },
            Connection {
                from: "HIPPOCAMPUS".to_string(),
                to: "BG".to_string(),
                label: "Context State".to_string(),
            },
            // Executive Control
            Connection {
                from: "PFC".to_string(),
                to: "BG".to_string(),
                label: "Strategy Selection".to_string(),
            },
            Connection {
                from: "CORTEX".to_string(),
                to: "BG".to_string(),
                label: "Action Proposals".to_string(),
            },
            Connection {
                from: "CEREBELLUM".to_string(),
                to: "MOTOR".to_string(),
                label: "Fine Tuning/Almgren-Chriss".to_string(),
            },
            // Limbic Regulation
            Connection {
                from: "HYPOTHALAMUS".to_string(),
                to: "AMYGDALA".to_string(),
                label: "Risk Appetite/Kelly".to_string(),
            },
            Connection {
                from: "AMYGDALA".to_string(),
                to: "BG".to_string(),
                label: "Inhibition (NoGo/Fear)".to_string(),
            },
            Connection {
                from: "HYPOTHALAMUS".to_string(),
                to: "BG".to_string(),
                label: "Motivation (Dopamine)".to_string(),
            },
            // Action & Output
            Connection {
                from: "BG".to_string(),
                to: "MOTOR".to_string(),
                label: "Gated Execution".to_string(),
            },
            Connection {
                from: "MOTOR".to_string(),
                to: "CLIENTS".to_string(),
                label: "Updates".to_string(),
            },
            // Meta/Audit Loop
            Connection {
                from: "MOTOR".to_string(),
                to: "AUDIT".to_string(),
                label: "Post-Trade Logs".to_string(),
            },
            Connection {
                from: "AUDIT".to_string(),
                to: "CORTEX".to_string(),
                label: "Policy Updates (Meta-Learning)".to_string(),
            },
        ];

        Self {
            modules,
            connections,
            detected_modules: HashSet::new(),
        }
    }

    /// Scan a directory tree to detect which modules exist
    pub fn scan_directory(&mut self, root_path: &Path) -> Result<()> {
        self.detected_modules.clear();

        for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let path_str = path.to_string_lossy();

            for module in &self.modules {
                if path_str.contains(&module.path_pattern) {
                    self.detected_modules.insert(module.id.clone());
                }
            }
        }

        Ok(())
    }

    /// Generate Mermaid flowchart syntax
    pub fn generate_mermaid(&self) -> String {
        // Header and styling
        let mut output = vec![
            "graph TD".to_string(),
            "%% Neuromorphic Architecture - JANUS Trading System".to_string(),
            "%% Style Definitions".to_string(),
            "classDef sensory fill:#f9f,stroke:#333,stroke-width:2px,color:black;".to_string(),
            "classDef executive fill:#bbf,stroke:#333,stroke-width:2px,color:black;".to_string(),
            "classDef memory fill:#bfb,stroke:#333,stroke-width:2px,color:black;".to_string(),
            "classDef limbic fill:#fbb,stroke:#333,stroke-width:2px,color:black;".to_string(),
            "classDef action fill:#ddd,stroke:#333,stroke-width:4px,color:black;".to_string(),
            "classDef output fill:#fb9,stroke:#333,stroke-width:2px,color:black;".to_string(),
            "classDef meta fill:#eee,stroke:#333,stroke-dasharray:5 5,color:black;".to_string(),
            String::new(),
        ];

        // Group modules by region
        let mut modules_by_region: HashMap<String, Vec<&ModuleConfig>> = HashMap::new();
        for module in &self.modules {
            if self.detected_modules.contains(&module.id) {
                modules_by_region
                    .entry(module.region.name().to_string())
                    .or_default()
                    .push(module);
            }
        }

        // Define subgraphs in priority order
        let priority_order = vec![
            "Sensory",
            "Memory",
            "Executive",
            "Limbic",
            "Action",
            "Output",
            "Meta",
        ];

        for region_name in priority_order {
            if let Some(modules) = modules_by_region.get(region_name) {
                output.push(format!("subgraph {}", region_name));
                for module in modules {
                    output.push(format!("    {}['{}']", module.id, module.label));
                }
                output.push("end".to_string());
            }
        }

        output.push(String::new());

        // Add connections (only if both endpoints are detected)
        for conn in &self.connections {
            if self.detected_modules.contains(&conn.from)
                && self.detected_modules.contains(&conn.to)
            {
                output.push(format!("{} -->|{}| {}", conn.from, conn.label, conn.to));
            }
        }

        output.push(String::new());

        // Apply styles
        let style_map: HashMap<_, _> = [
            ("Sensory", "sensory"),
            ("Executive", "executive"),
            ("Memory", "memory"),
            ("Limbic", "limbic"),
            ("Action", "action"),
            ("Output", "output"),
            ("Meta", "meta"),
        ]
        .iter()
        .cloned()
        .collect();

        for module in &self.modules {
            if self.detected_modules.contains(&module.id) {
                let style = style_map.get(module.region.name()).unwrap_or(&"executive");
                output.push(format!("class {} {}", module.id, style));
            }
        }

        output.join("\n")
    }

    /// Generate a summary of detected modules
    pub fn summary(&self) -> ModuleSummary {
        let mut by_region: HashMap<String, Vec<String>> = HashMap::new();

        for module in &self.modules {
            if self.detected_modules.contains(&module.id) {
                by_region
                    .entry(module.region.name().to_string())
                    .or_default()
                    .push(module.label.clone());
            }
        }

        ModuleSummary {
            total_modules: self.detected_modules.len(),
            detected_regions: by_region.len(),
            modules_by_region: by_region,
        }
    }
}

impl Default for NeuromorphicMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of detected modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSummary {
    pub total_modules: usize,
    pub detected_regions: usize,
    pub modules_by_region: HashMap<String, Vec<String>>,
}

/// Generate a simplified component-level flowchart
pub fn generate_component_diagram(root_path: &Path, component: &str) -> Result<String> {
    let component_path = root_path.join(component);

    if !component_path.exists() {
        return Err(AuditError::other(format!(
            "Component not found: {}",
            component
        )));
    }

    let mut files = Vec::new();
    let mut directories = Vec::new();

    for entry in WalkDir::new(&component_path)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name() {
                files.push(name.to_string_lossy().to_string());
            }
        } else if path.is_dir() && path != component_path {
            if let Some(name) = path.file_name() {
                directories.push(name.to_string_lossy().to_string());
            }
        }
    }

    let mut output = Vec::new();
    output.push("graph TD".to_string());
    output.push(format!("    {}[{}]", sanitize_id(component), component));

    for (i, dir) in directories.iter().enumerate() {
        let id = format!("DIR{}", i);
        output.push(format!("    {}[{}]", id, dir));
        output.push(format!("    {} --> {}", sanitize_id(component), id));
    }

    Ok(output.join("\n"))
}

/// Sanitize a string to be a valid Mermaid ID
fn sanitize_id(s: &str) -> String {
    s.replace(['/', '.', '-', ' '], "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_region_colors() {
        assert_eq!(BrainRegion::Sensory.color(), "#f9f");
        assert_eq!(BrainRegion::Memory.color(), "#bfb");
    }

    #[test]
    fn test_default_map() {
        let map = NeuromorphicMap::new();
        assert!(!map.modules.is_empty());
        assert!(!map.connections.is_empty());
    }

    #[test]
    fn test_mermaid_generation() {
        let mut map = NeuromorphicMap::new();
        map.detected_modules.insert("SENSORY".to_string());
        map.detected_modules.insert("DSP".to_string());

        let mermaid = map.generate_mermaid();
        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("SENSORY"));
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(sanitize_id("src/data"), "src_data");
        assert_eq!(sanitize_id("my-component"), "my_component");
    }
}
