#![allow(unused_variables, unused_mut, dead_code)]
// ==========================================================================
// aagl-core — PUBLIC FACADE
//
// This file is INTENTIONALLY THIN. It declares 4 Clean Architecture layer
// modules and re-exports their PUBLIC symbols flat on the crate root.
// NO business logic, NO IO, NO hash code lives here.
//
// Dependency flow (Clean Architecture):
//
//     lib.rs (YOU ARE HERE)  ←  user / tests / FFI consumers
//              │
//              ▼
//     application (use cases:  create_empty_project, add_node, delete_node …)
//              │
//              ├───► domain  (pure data models: Manifest, Node, Chunk, Theme)
//              └───► infrastructure  (IO details: FolderBackend, ZipBackend, sha256)
//              │
//              ▼
//     common (no internal deps:  AaglError, Result, constants)
//
// To understand WHAT EACH LAYER OWNS, open its mod.rs header comment:
//   [common/mod.rs](common/index.html)
//   [domain/mod.rs](domain/index.html)
//   [infrastructure/mod.rs](infrastructure/index.html)
//   [application/mod.rs](application/index.html)
// ==========================================================================

// ─── 1. Layer module declarations ─────────────────────────────────────────
pub mod common;
pub mod domain;
pub mod infrastructure;
pub mod application;

// ==========================================================================
// 2. FLAT PUBLIC API RE-EXPORTS  (the only place a user ever needs to import from)
//    Everything below is one clickable declaration → users open lib.rs, SEE
//    EVERY CALLABLE in one place.
// ==========================================================================

// ─── Common (no-dep root types) ───────────────────────────────────────────
pub use common::{AaglError, Result};
pub use common::{
    CREATE_AS_FOLDER, DEFAULT_BASE_UNIT, DEFAULT_GENESIS_SPACE_SIZE, DEFAULT_PROFILE_ID,
    DEFAULT_RELATION_CODES, DEFAULT_RESOLUTION, DEFAULT_THEME_ID, DEFAULT_VIEW_TYPE_KEY,
    GENESIS_CHUNK_ID, GENESIS_ROOT_PARENT, SPEC_VERSION,
};

// ─── Domain (pure data models) ────────────────────────────────────────────
pub use domain::{
    Chunk, ChunkMetadata, CoreHints, Manifest, Node, PatchContext, Profile, Relation, Theme,
    Units, ViewsRegistry,
};

// ─── Infrastructure (hash + storage backends) ─────────────────────────────
pub use infrastructure::{
    AnyStorage, FolderBackend, StorageBackend, ZipBackend, open_storage,
    sha256_canonical_json, sha256_hex, zip_folder,
};

// ─── Application (use cases) ──────────────────────────────────────────────
//
// (Full todo!() command signatures live in application::commands::*
//  re-exported below; one implementation is done: create_empty_project used
//  by create_db_file thin wrappers.)
pub use application::create_empty_project;
pub use application::commands::{
    add_node, add_node_to_chunk, add_relation, apply_data_patch, aagl_to_folder, db_info,
    delete_node, folder_to_aagl, get_node, list_nodes, load_chunk, load_manifest, load_theme,
    patch_node_metadata, recompute_chunk_hashes, recompute_state_hash, register_chunk,
    resolve_style, set_node_coords, set_node_metadata, set_node_parent, validate_hashes,
    validate_structure, verify_occ,
};

// ─── Working v1 node mutations (single-chunk implementation, done):
pub use application::{add_object, delete_object, update_object};

// ==========================================================================
// 3. TOP-LEVEL ANCHOR-STYLE FACADE FUNCTIONS  (thin wrappers with defaults)
//
// These are the only routines that read the global `CREATE_AS_FOLDER` toggle
// and `DEFAULT_GENESIS_SPACE_SIZE` constant. Everything below delegates to
// application::create_empty_project (real implementation lives there).
// ==========================================================================

/// Create a new empty valid AAGL database.
///
/// # Behaviour
/// * **Folder mode** if `CREATE_AS_FOLDER = true` (dev default): `db_path` is created
///   as a plain directory with 4 JSON files + 3 subdirs (git diffable).
/// * **ZIP mode** if `CREATE_AS_FOLDER = false` (document distribution): `db_path` must
///   be a path ending in `.aagl` — a single compressed ZIP file is produced.
///
/// Genesis cube size defaults to `DEFAULT_GENESIS_SPACE_SIZE = 1_000_000`
/// (bounds `[-500_000, ±, +500_000]`).
pub fn create_db_file(
    db_path: impl AsRef<std::path::Path>,
    project_name: impl Into<String>,
) -> Result<()> {
    create_db_file_with_space_size(db_path, project_name, DEFAULT_GENESIS_SPACE_SIZE)
}

/// Same as `create_db_file` but with an explicit genesis cube edge length.
pub fn create_db_file_with_space_size(
    db_path: impl AsRef<std::path::Path>,
    project_name: impl Into<String>,
    genesis_space_size: i64,
) -> Result<()> {
    use std::path::Path;
    let db_path_ref: &Path = db_path.as_ref();
    let name: String = project_name.into();
    create_empty_project(db_path_ref, &name, CREATE_AS_FOLDER, genesis_space_size)
}
