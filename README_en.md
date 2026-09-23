# AAGL (Agentic Architectural Graph Language)

**AAGL** is an open-source, LLM-native, and agent-first spatial data standard designed to replace legacy CAD formats (DXF/IFC). It provides a deterministic, high-performance, graph-based system specifically engineered to allow autonomous AI agents to perform reliable, concurrent spatial and architectural edits.

---

## ✅ Current Implementation Status (v0.1.0 — Core Engine MVP)

- [x] **Container Layer (§ Container Layer)**: `.aagl` ZIP format **and** plain Folder mode (git-friendly dual-backend)
- [x] **Graph Layer (§ Graph Layer)**: Universal Graph — 4-field Node (`parent_id`, `coords: Vec<i64>`, `relations`, `metadata`), chunks, default Genesis cube with bounds ±500_000
- [x] **Clean Architecture (4 layers)**:
    - `common/` — constants (`SPEC_VERSION`, `CREATE_AS_FOLDER`, `DEFAULT_GENESIS_SPACE_SIZE` …) + unified error enum `AaglError` + `Result<T>`
    - `domain/` — pure data models `Manifest`, `Chunk`, `Node`, `Relation`, `Theme`, `PatchContext` (no logic, no IO)
    - `application/` — use cases: `create_empty_project`, **3 working mutations** `add_object` / `delete_object` / `update_object` (single-chunk v1) + 25+ Anchor-style command stubs (todo!)
    - `infrastructure/` — `hash::sha256_hex / sha256_canonical_json`, Storage trait + FolderBackend / ZipBackend (enum `AnyStorage` static dispatch), `open_storage` helper auto-detects backend by file type
- [x] **Public API facade (lib.rs)**: flat Anchor-style calls (`create_db_file`, `add_object`, …) — database path is **always the first argument**
- [x] **FFI-ready**: `crate-type = [cdylib, rlib, staticlib]` — Python ctypes integration (next step)
- [x] **OCC Anchors §3.2**: `manifest.state_hash` computed via "zero field → sha256 → write back" rule; `chunk.version` and `manifest.global_version` monotonically increment on every mutation
- [x] **25/25 integration+unit tests** passing:
    - 2 unit (initializer: genesis bounds, hash format)
    - 10 create_db_file: file tree, manifest hashes, default 1e6 cube, custom 10e6 cube, theme struct parse, schema.json present, ZIP build, rejects size 0/1, overwrite folder, overwrite zip
    - 13 node_crud: add×4, delete×4, update×4, ZIP roundtrip

---

## 🚀 Key Architectural Principles

1. **Configurable Deterministic Coordinate System**: Uses native `i64` integers paired with customizable global and local unit scales (e.g., millimeters for architecture, nanometers for microchips) to completely eliminate floating-point precision drift.
2. **Containerized Format (`.aagl`)**: A lightweight ZIP-based container format that balances fast runtime access (via selective chunk loading) with clean version control. Folder-mode available for development (see the `CREATE_AS_FOLDER` toggle in `common/constants.rs`).
3. **Optimistic Concurrency Control (OCC)**: Built-in version hashes (`state_hash` / `base_version`) in the manifest to ensure safe, conflict-free parallel modifications by multiple AI agents.
4. **Universal Graph**: A flexible base structure for general spatial data representation.
5. **High-Performance Rust Core**: State management, parsing, and atomic patching handled by a blazing-fast Rust engine with Python bindings for LLM orchestration.

---

## 🏗️ Rust Core Structure (`/core/`)

The DB engine lives in `/core/` as a standalone `aagl-core` crate. Split by Clean Architecture rules.

```text
core/src/
├── lib.rs                         # FACADE ONLY — modules + flat re-exports + thin create_db_file* wrappers
├── common/                        # Dependency root — constants + errors (0 imports from sibling layers!)
│   ├── constants.rs               # SPEC_VERSION, CREATE_AS_FOLDER, DEFAULT_GENESIS_SPACE_SIZE…
│   └── error.rs                   # AaglError enum + pub type Result<T> (thiserror 8 variants)
├── domain/                        # PURE DATA MODELS. Zero impl blocks, zero IO
│   ├── manifest.rs                # Manifest, Profile, CoreHints, Units, ChunkMetadata, ViewsRegistry
│   ├── graph.rs                   # Chunk, Node, Relation (4 Node fields: parent/coords/relations/metadata)
│   └── view.rs                    # Theme (3-tier fallback) + PatchContext (OCC base anchors)
├── application/                   # Use cases / interactors
│   ├── initializer.rs             # create_empty_project (build tempdir → rename folder / zip)
│   ├── node_manager.rs            # ⭐ working v1: add_object / delete_object / update_object
│   └── commands.rs                # 25+ anchor-style signatures todo!() for next milestones
└── infrastructure/                # IO details / driven adapters
    ├── hash.rs                    # sha256_hex("sha256:"+64hex) + sha256_canonical_json (§3.2)
    ├── storage.rs                 # StorageBackend trait + FolderBackend + ZipBackend + enum AnyStorage + zip_folder()
    └── open_storage.rs            # pub fn open_storage(path) → auto Folder vs ZIP

core/tests/
├── create_db_file_tests.rs        # 10 integration tests initializer
└── node_crud_tests.rs             # 13 integration tests node CRUD v1
```

---

## 🧪 Quick Start (Rust side)

```bash
# 0. Enter the core crate
cd core

# 1. Sanity build check
cargo check                       # ✅ 0 errors, 0 warnings

# 2. Run the FULL test suite
cargo test                        # ✅ 25/25 passed (2 unit + 23 integration)

# 3. Run the live CRUD demo
cargo run
# → creates core/demo_crud_objects/ :
#    create → add wall → add window → update wall (stretch 4.5→6m) → delete window
#    versions: manifest.global_version=4, chunk_genesis.version=4, state_hash recomputed cleanly

# 4. Build the FFI shared library (.dylib / .so / .dll) for Python ctypes
cargo build --release             # → target/release/libaagl_core.{dylib,so,dll}
```

---

## 🐍 Python Integration (Next Release — Almost Ready!)

The core is already callable from Python via standard `ctypes`:
- crate-type: `cdylib` (shared lib for dynamic loading)
- all public methods use the Anchor-style signature "DB path first, unified Result back"
- planned layer: module `src/ffi.rs` with `extern "C"` wrappers, args / results passed as JSON strings via `CString` (caller runs `aagl_free_c_string()` to prevent leaks — standard rocksdb/libgit2 pattern)

Example Python-side (after `--release` build):

```python
from aagl_core_ffi import AaglFFI
ffi = AaglFFI(library_path="./core/target/release/libaagl_core.dylib")

ffi.create_db_file("./my_project.aagl", "My CAD Project")
wall_id = ffi.add_object("./my_project.aagl",
    parent_id="node_0001",
    coords=[0, 0, 0, 4500, 380, 2700],
    metadata={"profile_key":"wall", "material_code":"brick", "thickness_mm":380},
    relations=[], prefix="wall_")
ffi.update_object("./my_project.aagl", wall_id,
    new_coords=[0,0,0, 6000, 380, 2700])
ffi.delete_object("./my_project.aagl", wall_id)
```

---

## 📁 Repository Structure

```text
aagl/
├── SPECIFICATION_en.md       # Technical specification (English)
├── SPECIFICATION_ru.md       # Technical specification (Russian)
├── example_cad_floorplan/    # Sample .aagl project (arch_floorplan_v0.1 — 4×5 m room with walls/window/door)
│   ├── manifest.json         # Routing table + profile declaration + OCC anchors
│   ├── schema.json           # *Bundle schema*: (A) AAGL Container Layer structural rules
│   │                         #                (B) View-theme mechanism structural rules
│   │                         #                (C) arch_floorplan_v0.1 profile semantic metadata contracts
│   ├── chunks/chunk_genesis.json
│   └── views/default_theme.json
│
├── core/                     # ⭐ Rust crate aagl-core — DB ENGINE (MVP working, 25/25 tests green)
│   ├── Cargo.toml            # crate-type = [cdylib, rlib, staticlib], features = ["ffi"]
│   ├── src/                  # Clean Architecture 4 layers (common/domain/application/infrastructure)
│   ├── tests/                # integration tests (create_db_file + node_crud = 23 test cases)
│   └── examples/             # (soon) FFI demo examples
│
├── crates/                   # (future) Rust workspace — aagl-container / aagl-cli
├── bindings/                 # (next release) Python — FFI ctypes loader + pip package
└── tests/                    # (future) Multi-agent OCC integration scenarios
```

---

📦 The .aagl Container Format

Instead of massive monolithic files or unmanageable folder trees, an .aagl project is a single portable container:

```text
project.aagl (ZIP)  OR  project_folder/  (plain folder dev-mode, CREATE_AS_FOLDER=true)
├── manifest.json             # Global routing table, spatial_index, version hashes, and base units
├── schema.json               # Active profile *bundle-schema*: container structural rules + profile metadata rules
├── chunks/                   # Sharded sub-graphs (50-100 KB chunks for efficient LLM context)
│   ├── chunk_genesis.json    # Mandatory identity-root chunk with global bounds. v1: ONLY chunk in the graph
│   └── sector_01.json        # (future) sharding expansion
├── views/                    # Presentation strategies (style themes — not part of OCC data hashes)
│   └── default_theme.json
└── assets/                   # Binary resources (textures, floor plan scans, point clouds)
```

---

🛠️ Tech Stack

Core Engine: Rust (Zero-cost abstractions, memory safety, high-speed parsing).
- `serde / serde_json` — serialization, canonical JSON for state hashing
- `thiserror` — unified AaglError error enum
- `zip` (deflate) — `.aagl` ZIP backend
- `sha2` — sha256 OCC anchors
- `tempfile` — atomic writes (tempdir → rename pattern)

AI Orchestration: Python (Seamless integration with LLM frameworks via stdlib ctypes — no PyO3 build step needed, works on stock CPython 3.8+).

Interchange: JSON Patch (RFC 6902) for atomic, stateful updates.

---

📄 License

AAGL follows a Dual-Licensing Model:

Non-Commercial Use: This project is licensed under the Non-Commercial License — free for academic, research, personal, and evaluation purposes.

Commercial Use: For any commercial application, enterprise deployment, or integration into revenue-generating products, a Commercial License is required.

Please see the LICENSE file for full details or contact us for commercial inquiries.

---

🗺️ Roadmap

- [x] **v0.1.0 Core MVP**: Clean Architecture 4 layers, create_db_file, dual Folder/ZIP backend, genesis cube, OCC anchors (state_hash + versions), 3 node CRUD operations single-chunk, 25/25 tests green, FFI-ready cdylib build target
- [ ] **v0.1.1**: Python ctypes FFI bundle + pip package (`pip install aagl-core`) + `demo_crud.py`
- [ ] **v0.2**: Multi-chunk graph (`register_chunk`, `load_chunk`, `spatial_index` in manifest)
- [ ] **v0.3**: Real OCC validation (`verify_occ(ctx)`, apply_data_patch returns Err OccConflict on mismatch)
- [ ] **v0.4**: Profile Schema Layer (arch_cad_v1): TypeDefinitions for wall/window/door/column/beam, compile schema.json, coords layout validation (bbox6/polyline2N/…)
- [ ] **v0.5**: Spatial indexing (R-Tree) + multi-agent concurrency integration tests
- [ ] **v0.6**: `aagl-cli` packer/unpacker (folder ↔ .aagl) + git hooks helper
- [ ] **v1.0-beta**: Extensible profile validation rules (JSON Schema) + stable FFI ABI