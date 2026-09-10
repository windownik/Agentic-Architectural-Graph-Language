# AAGL (Agentic Architectural Graph Language)

**AAGL** is an open-source, LLM-native, and agent-first spatial data standard designed to replace legacy CAD formats (DXF/IFC). It provides a deterministic, high-performance, graph-based system specifically engineered to allow autonomous AI agents to perform reliable, concurrent spatial and architectural edits.

---

## 🚀 Key Architectural Principles

1. **Configurable Deterministic Coordinate System**: Uses native `i64` integers paired with customizable global and local unit scales (e.g., millimeters for architecture, nanometers for microchips) to completely eliminate floating-point precision drift.
2. **Containerized Format (`.aagl`)**: A lightweight ZIP-based container format that balances fast runtime access (via selective chunk loading) with clean version control.
3. **Optimistic Concurrency Control (OCC)**: Built-in version hashes (`state_hash` / `base_version`) in the manifest to ensure safe, conflict-free parallel modifications by multiple AI agents.
4. **Universal Graph**: A flexible base structure for general spatial data representation.
5. **High-Performance Rust Core**: State management, parsing, and atomic patching handled by a blazing-fast Rust engine with Python bindings for LLM orchestration.

---

## 📁 Repository Structure

```text
aagl/
├── SPECIFICATION.md          # Detailed technical specification of the AAGL standard
├── docs/                     # Architecture Decision Records (ADRs)
├── schemas/                  # JSON Schemas for Universal Graph and Strict CAD Profile
├── crates/                   # Rust Workspace (Core Engine)
│   ├── aagl-core/            # Graph engine, JSON Patch (RFC 6902), and validation
│   ├── aagl-container/       # .aagl ZIP container manager & chunk loader
│   └── aagl-cli/             # CLI utilities (pack/unpack, git-hooks helper)
├── bindings/                 # Language integrations (Python via PyO3)
├── examples/                 # Sample architectural .aagl projects
└── tests/                    # Integration and multi-agent concurrency tests
```
📦 The .aagl Container Format
Instead of massive monolithic files or unmanageable folder trees, an .aagl project is a single portable container:
```text
Plaintext
project.aagl (ZIP Container)
├── manifest.json             # Global routing table, spatial_index, version hashes, and base units
├── schema.json               # Active profile validation rules
├── chunks/                   # Sharded sub-graphs (50-100 KB chunks for efficient LLM context)
│   ├── floor_01.json
│   └── floor_02.json
└── assets/                   # Binary resources (textures, floor plan scans, point clouds)
```
🛠️ Tech Stack
Core Engine: Rust (Zero-cost abstractions, memory safety, high-speed parsing).

AI Orchestration: Python (Seamless integration with LLM frameworks via PyO3).

Interchange: JSON Patch (RFC 6902) for atomic, stateful updates.

📄 License
AAGL follows a Dual-Licensing Model:

Non-Commercial Use: This project is licensed under the Non-Commercial License — free for academic, research, personal, and evaluation purposes.

Commercial Use: For any commercial application, enterprise deployment, or integration into revenue-generating products, a Commercial License is required.

Please see the LICENSE file for full details or contact us for commercial inquiries.

🗺️ Roadmap
[ ] Core Rust engine prototype (Graph & JSON Patch implementation)

[ ] .aagl container packer/unpacker (aagl-cli)

[ ] Python bindings via PyO3 (pip install aagl-core)

[ ] Spatial indexing (R-Tree) and multi-agent OCC conflict resolution

[ ] Strict CAD Profile validation rules