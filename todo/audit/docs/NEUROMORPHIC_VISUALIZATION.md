# Neuromorphic Architecture Visualization

A powerful visualization system for mapping the JANUS neuromorphic trading system to biological brain regions and generating interactive flowcharts.

## Overview

The neuromorphic visualization feature transforms your code architecture into intuitive brain-inspired diagrams, making it easy to:

- **Understand System Architecture**: See how components map to biological brain regions
- **Visualize Data Flow**: Track the path of market data through sensory, executive, and motor systems
- **Document Design Decisions**: Explain the neuromorphic architecture to stakeholders
- **Identify Missing Components**: Quickly see which brain regions are implemented

## Biological Metaphor

JANUS models a high-frequency trading system after the mammalian brain's decision-making pathways:

### Brain Regions

| Region | Biological Function | JANUS Equivalent | Code Location |
|--------|---------------------|------------------|---------------|
| **Sensory** | Process external stimuli | Market data ingestion and filtering | `src/data`, `src/dsp`, `neuromorphic/thalamus` |
| **Memory** | Spatial and temporal mapping | State encoding and prediction | `neuromorphic/hippocampus` |
| **Executive** | Planning and strategy | Strategy selection and working memory | `neuromorphic/cortex`, `neuromorphic/prefrontal` |
| **Limbic** | Emotion and risk regulation | Risk management and homeostasis | `neuromorphic/amygdala`, `neuromorphic/hypothalamus` |
| **Action** | Action selection | Go/NoGo trading decisions | `neuromorphic/basal_ganglia` |
| **Output** | Motor execution | Order execution and visualization | `src/execution`, `src/clients` |
| **Meta** | Self-reflection | Performance analysis and learning | `src/audit` |

### Signal Pathways

**Path A: Exteroception** (The "Thinking" Loop)
```
Exchange → DSP → Thalamus → Hippocampus → Basal Ganglia → Execution
```
Primary alpha generation - detects patterns and executes profitable trades.

**Path B: Interoception** (The "Body Awareness" Loop)
```
Prometheus → Insula → Hypothalamus
```
Monitors system health (latency, CPU, inventory) and adjusts risk.

**Path C: Nociception** (The "Pain" Loop)
```
AlertManager → Amygdala → Hyperdirect Pathway → STOP
```
Survival reflex - instantly neutralizes risk on critical alerts.

## Usage

### CLI

#### Generate Full Neuromorphic Map

```bash
# Scan current directory
cargo run --bin audit-cli -- visualize .

# Scan specific project path
cargo run --bin audit-cli -- visualize /path/to/janus

# Save to file
cargo run --bin audit-cli -- visualize . -o brain_map.mmd

# JSON output
cargo run --bin audit-cli -- visualize . --format json
```

**Output:**
```
================================================================================
🧠 Neuromorphic Architecture Analysis
================================================================================
Total Modules Detected: 8
Active Brain Regions: 5

Modules by Region:

  Sensory:
    - External Data Feeds
    - Signal Processing (DSP)
    - Thalamus (TRN Gating)

  Memory:
    - Hippocampus (Time Cells/SR)

  Action:
    - Basal Ganglia (Go/NoGo)

  Output:
    - Motor Cortex (Execution)

  Meta:
    - LLM Audit (Meta-Cognition)

================================================================================
📊 Mermaid Diagram (neuromorphic type)
================================================================================

graph TD
%% Neuromorphic Architecture - JANUS Trading System
%% Style Definitions
classDef sensory fill:#f9f,stroke:#333,stroke-width:2px,color:black;
...
```

#### Generate Component Diagram

```bash
# Visualize a specific component
cargo run --bin audit-cli -- visualize . --diagram-type component --component src/execution

# Save component diagram
cargo run --bin audit-cli -- visualize . -t component -c src/dsp -o dsp_diagram.mmd
```

### API

#### Neuromorphic Architecture Endpoint

```bash
curl -X POST http://localhost:8080/api/visualize/neuromorphic \
  -H "Content-Type: application/json" \
  -d '{
    "path": "."
  }'
```

**Response:**
```json
{
  "diagram_type": "neuromorphic",
  "mermaid": "graph TD\n%% Neuromorphic Architecture...",
  "summary": {
    "total_modules": 8,
    "detected_regions": 5,
    "modules_by_region": {
      "Sensory": [
        "External Data Feeds",
        "Signal Processing (DSP)",
        "Thalamus (TRN Gating)"
      ],
      "Memory": [
        "Hippocampus (Time Cells/SR)"
      ],
      ...
    }
  }
}
```

#### Component Visualization Endpoint

```bash
curl -X POST http://localhost:8080/api/visualize/component \
  -H "Content-Type: application/json" \
  -d '{
    "path": ".",
    "component": "src/execution"
  }'
```

**Response:**
```json
{
  "diagram_type": "component",
  "mermaid": "graph TD\n    src_execution[src/execution]\n...",
  "summary": null
}
```

## Mermaid Integration

The generated diagrams use [Mermaid](https://mermaid.js.org/) syntax, which can be:

### Viewed Online
1. Copy the Mermaid code
2. Visit [mermaid.live](https://mermaid.live/)
3. Paste and interact with the diagram

### Embedded in Markdown
```markdown
```mermaid
graph TD
    SENSORY['External Data Feeds']
    DSP['Signal Processing (DSP)']
    SENSORY -->|Raw Data| DSP
```
```

### Integrated in Documentation Sites
- GitHub (automatic rendering in `.md` files)
- GitLab (automatic rendering)
- MkDocs with `mkdocs-mermaid2-plugin`
- Docusaurus with `@docusaurus/theme-mermaid`

### Exported to Images
Using [mermaid-cli](https://github.com/mermaid-js/mermaid-cli):
```bash
# Install
npm install -g @mermaid-js/mermaid-cli

# Convert to PNG
mmdc -i brain_map.mmd -o brain_map.png

# Convert to SVG
mmdc -i brain_map.mmd -o brain_map.svg -b transparent
```

## Color Scheme

Each brain region has a distinct color for easy identification:

- **Sensory** (#f9f): Pink - Input processing
- **Memory** (#bfb): Green - Information storage
- **Executive** (#bbf): Blue - Planning and control
- **Limbic** (#fbb): Red - Risk and emotion
- **Action** (#ddd): Gray (thick border) - Decision point
- **Output** (#fb9): Orange - Execution
- **Meta** (#eee): Light gray (dashed) - Self-reflection

## Example Diagrams

### Full Neuromorphic Architecture

```mermaid
graph TD
    subgraph Sensory
        SENSORY['External Data Feeds']
        DSP['Signal Processing (DSP)']
        THALAMUS['Thalamus (TRN Gating)']
    end
    
    subgraph Memory
        HIPPOCAMPUS['Hippocampus (Time Cells/SR)']
    end
    
    subgraph Executive
        CORTEX['Cortex (Strategy/Planning)']
        PFC['Prefrontal (Working Memory)']
    end
    
    subgraph Limbic
        AMYGDALA['Amygdala (Fear/Risk Veto)']
        HYPOTHALAMUS['Hypothalamus (Homeostasis/Kelly)']
    end
    
    subgraph Action
        BG['Basal Ganglia (Go/NoGo)']
    end
    
    subgraph Output
        MOTOR['Motor Cortex (Execution)']
        CLIENTS['Frontend (Visualization)']
    end
    
    subgraph Meta
        AUDIT['LLM Audit (Meta-Cognition)']
    end
    
    SENSORY -->|Raw Data| DSP
    DSP -->|Cleaned Signal| THALAMUS
    THALAMUS -->|State Encoding (Grid Cells)| HIPPOCAMPUS
    THALAMUS -->|Fast Threat Detection| AMYGDALA
    THALAMUS -->|Bursts/Attention| CORTEX
    HIPPOCAMPUS -->|Successor Representation| PFC
    HIPPOCAMPUS -->|Context State| BG
    PFC -->|Strategy Selection| BG
    CORTEX -->|Action Proposals| BG
    AMYGDALA -->|Inhibition (NoGo/Fear)| BG
    HYPOTHALAMUS -->|Motivation (Dopamine)| BG
    HYPOTHALAMUS -->|Risk Appetite/Kelly| AMYGDALA
    BG -->|Gated Execution| MOTOR
    MOTOR -->|Updates| CLIENTS
    MOTOR -->|Post-Trade Logs| AUDIT
    AUDIT -->|Policy Updates (Meta-Learning)| CORTEX
    
    class SENSORY,DSP,THALAMUS sensory
    class HIPPOCAMPUS memory
    class CORTEX,PFC executive
    class AMYGDALA,HYPOTHALAMUS limbic
    class BG action
    class MOTOR,CLIENTS output
    class AUDIT meta
```

### Minimal Example (Only Detected Modules)

If you only have `src/data`, `src/dsp`, and `src/execution`:

```mermaid
graph TD
    subgraph Sensory
        SENSORY['External Data Feeds']
        DSP['Signal Processing (DSP)']
    end
    
    subgraph Output
        MOTOR['Motor Cortex (Execution)']
    end
    
    SENSORY -->|Raw Data| DSP
    
    class SENSORY,DSP sensory
    class MOTOR output
```

## Module Detection

The system scans your codebase for specific path patterns:

| Pattern | Module ID | Label |
|---------|-----------|-------|
| `src/data` | SENSORY | External Data Feeds |
| `src/dsp` | DSP | Signal Processing (DSP) |
| `neuromorphic/thalamus` | THALAMUS | Thalamus (TRN Gating) |
| `neuromorphic/hippocampus` | HIPPOCAMPUS | Hippocampus (Time Cells/SR) |
| `neuromorphic/cortex` | CORTEX | Cortex (Strategy/Planning) |
| `neuromorphic/prefrontal` | PFC | Prefrontal (Working Memory) |
| `neuromorphic/cerebellum` | CEREBELLUM | Cerebellum (Error Correction) |
| `neuromorphic/amygdala` | AMYGDALA | Amygdala (Fear/Risk Veto) |
| `neuromorphic/hypothalamus` | HYPOTHALAMUS | Hypothalamus (Homeostasis/Kelly) |
| `neuromorphic/basal_ganglia` | BG | Basal Ganglia (Go/NoGo) |
| `src/execution` | MOTOR | Motor Cortex (Execution) |
| `src/clients` | CLIENTS | Frontend (Visualization) |
| `src/audit` | AUDIT | LLM Audit (Meta-Cognition) |

## Customization

### Adding New Modules

Edit `src/audit/src/neuromorphic_mapper.rs`:

```rust
// In NeuromorphicMap::new()
modules.push(ModuleConfig {
    id: "INSULA".to_string(),
    label: "Insula (Interoception)".to_string(),
    region: BrainRegion::Limbic,
    path_pattern: "neuromorphic/insula".to_string(),
});
```

### Adding New Connections

```rust
// In NeuromorphicMap::new()
connections.push(Connection {
    from: "PROMETHEUS".to_string(),
    to: "INSULA".to_string(),
    label: "Metrics".to_string(),
});
```

### Custom Color Schemes

Modify the CSS classes in `generate_mermaid()`:

```rust
output.push(
    "classDef custom fill:#abc,stroke:#333,stroke-width:3px;".to_string()
);
```

## Integration with Documentation

### MkDocs Example

```yaml
# mkdocs.yml
plugins:
  - search
  - mermaid2

markdown_extensions:
  - pymdownx.superfences:
      custom_fences:
        - name: mermaid
          class: mermaid
          format: !!python/name:pymdownx.superfences.fence_code_format
```

Then in your docs:

```markdown
# Architecture

```mermaid
[paste generated Mermaid code here]
```
```

### Automated Updates

Add to your CI pipeline:

```yaml
# .github/workflows/docs.yml
- name: Generate Architecture Diagram
  run: |
    cd src/audit
    cargo run --bin audit-cli -- visualize ../.. -o ../../docs/architecture.mmd
    
- name: Convert to PNG
  run: |
    npx @mermaid-js/mermaid-cli -i docs/architecture.mmd -o docs/architecture.png
```

## Use Cases

### 1. Onboarding New Developers

Show the big picture:
```bash
cargo run --bin audit-cli -- visualize . > docs/ARCHITECTURE.mmd
```

New team members can see:
- How data flows through the system
- Which components are "thinking" vs "executing"
- Risk management pathways
- Meta-learning loops

### 2. Architecture Review

Generate before major refactoring:
```bash
# Before
cargo run --bin audit-cli -- visualize . -o before_refactor.mmd

# After changes
cargo run --bin audit-cli -- visualize . -o after_refactor.mmd

# Compare visually
```

### 3. Stakeholder Presentations

The biological metaphor makes complex systems accessible:
- "The Amygdala acts as our kill switch when drawdown exceeds limits"
- "The Hippocampus predicts future market states using Time Cells"
- "The Basal Ganglia implements Go/NoGo decision gating"

### 4. System Monitoring

Check which components are actually deployed:
```bash
# Production server
ssh prod "cd /opt/janus && audit-cli visualize ." | grep "Total Modules"
```

### 5. Research Documentation

Include in research papers:
```latex
\begin{figure}
  \includegraphics{architecture.pdf}
  \caption{JANUS neuromorphic trading architecture}
\end{figure}
```

## Advanced Features

### Programmatic Access

```rust
use audit::NeuromorphicMap;

let mut map = NeuromorphicMap::new();
map.scan_directory(&Path::new("."))?;

// Get summary
let summary = map.summary();
println!("Detected {} modules in {} regions",
    summary.total_modules,
    summary.detected_regions
);

// Generate diagram
let mermaid = map.generate_mermaid();
std::fs::write("diagram.mmd", mermaid)?;
```

### Filtering by Region

```rust
// Only show sensory pathway
let sensory_modules: Vec<_> = map.modules.iter()
    .filter(|m| m.region == BrainRegion::Sensory)
    .collect();
```

### Custom Export Formats

Extend the system to support other formats:
- GraphViz DOT
- PlantUML
- D3.js JSON
- Cytoscape.js JSON

## Troubleshooting

### No Modules Detected

**Problem**: "Total Modules Detected: 0"

**Solutions**:
1. Check you're running from the correct directory
2. Verify path patterns match your directory structure
3. Ensure modules exist on disk

### Diagram Too Large

**Problem**: Mermaid diagram is cluttered

**Solutions**:
1. Use component diagrams for focused views
2. Filter by region
3. Increase diagram size in Mermaid Live settings
4. Export to SVG and zoom in external viewer

### Styling Not Applied

**Problem**: Colors not showing

**Solutions**:
1. Ensure class definitions are before `class` statements
2. Check Mermaid version compatibility
3. Try a different Mermaid renderer

## Future Enhancements

Planned features:
- [ ] Interactive web UI for diagram exploration
- [ ] Live updates during development
- [ ] Dependency graph overlay
- [ ] Performance metrics integration
- [ ] Time-series architecture evolution
- [ ] Automatic gap analysis (missing modules)
- [ ] Export to Graphviz, PlantUML, D3.js
- [ ] Integration with OpenTelemetry for runtime flow visualization

## References

- [Mermaid Documentation](https://mermaid.js.org/)
- [JANUS Neuromorphic Architecture Research](./NEUROMORPHIC_ARCHITECTURE_RESEARCH.md)
- [Project JANUS Paper](https://github.com/nuniesmith/technical_papers/blob/main/project_janus/janus.tex)
- [Brain-Inspired Computing](https://www.frontiersin.org/articles/10.3389/fnins.2021.665565/full)

## License

Same as parent project (see root LICENSE file).