#![allow(unused_variables, unused_mut, dead_code)]
// ==========================================================================
// AAGL-CORE — SINGLE-FILE FFI/COMMAND API SKETCH (all bodies = todo!())
// Target audience: Python (ctypes/cffi), C#, C++ cross-platform clients.
// Every public function:
//   • 1st arg = path to DB (either `.aagl` ZIP or flat project folder)
//   • returns aagl_core::Result<T> → auto-converts to {int code + message} via FFI
// ==========================================================================

// ─── Public Result type ────────────────────────────────────────────────────
pub type Result<T> = std::result::Result<T, AaglError>;

// ─── Constants (container format §1) ───────────────────────────────────────
pub const SPEC_VERSION: &str = "0.1.0";
pub const GENESIS_CHUNK_ID: &str = "chunk_genesis";
pub const GENESIS_ROOT_PARENT: &str = "genesis";
pub const DEFAULT_BASE_UNIT: &str = "mm";
pub const DEFAULT_RESOLUTION: f64 = 0.001;
pub const DEFAULT_PROFILE_ID: &str = "generic_spatial_v0";
pub const DEFAULT_VIEW_TYPE_KEY: &str = "metadata.profile_key";
pub const DEFAULT_RELATION_CODES: [&str; 4] = ["directed","symmetric","control","dependent"];

// ─── Data model types (§3 Manifest, §5 Chunk/Node/Relation) ───────────────
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(thiserror::Error, Debug)]
pub enum AaglError {
    #[error("I/O: {0}")] Io(#[from] std::io::Error),
    #[error("JSON: {0}")] Serialization(#[from] serde_json::Error),
    #[error("ZIP: {0}")] Zip(#[from] zip::result::ZipError),
    #[error("Validation: {0}")] Validation(String),
    #[error("OCC: expected v{expected}, found v{found}")] OccConflict{ expected:u64, found:u64 },
    #[error("Node not found: {0}")] NodeNotFound(String),
    #[error("Chunk not found: {0}")] ChunkNotFound(String),
    #[error("Patch: {0}")] Patch(String),
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Manifest {
    pub spec_version: String, pub project_name: String, pub state_hash: String,
    pub global_version: u64, pub profile: Profile, pub units: Units,
    pub chunks: HashMap<String, ChunkMetadata>,
    pub views: ViewsRegistry,
    pub spatial_index: HashMap<String, String>,
}
#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Profile { pub id:String, pub schema_path:String, pub view_type_key:String, pub core_hints:Option<CoreHints> }
#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct CoreHints { pub relation_codes: Vec<String> }
#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Units { pub base_unit: String, pub internal_resolution_to_meters: f64 }
#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct ChunkMetadata { pub path: String, pub hash: String, pub version: u64 }
#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct ViewsRegistry { pub active_theme: String, pub registered: HashMap<String, ChunkMetadata> }
#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Chunk { pub chunk_id: String, pub version: u64, pub bounds: Option<[i64;6]>, pub nodes: HashMap<String, Node> }
#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Node { pub parent_id: String, pub coords: Vec<i64>, pub relations: Vec<Relation>, pub metadata: serde_json::Value }
#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Relation { pub target: String, pub target_chunk: String, pub rel_type: String }

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct Theme {
    pub theme_id: String, pub theme_name: String, pub spec_version: String,
    #[serde(default)] pub fallback: HashMap<String, serde_json::Value>,
    #[serde(default)] pub by_type:  HashMap<String, HashMap<String, serde_json::Value>>,
    #[serde(default)] pub by_id:    HashMap<String, HashMap<String, serde_json::Value>>,
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct PatchContext {
    pub base_global_version: u64,
    pub base_chunk_versions: HashMap<String, u64>,
}

// ==========================================================================
// PUBLIC HIGH-LEVEL COMMAND API (Anchor-style / FFI-ready stateless calls)
// Every callable signature + TODO plan. Compiles with no logic yet.
// ==========================================================================

// ── Group A: Database creation ────────────────────────────────────────────

pub fn create_db_file(zip_path: impl AsRef<std::path::Path>, project_name: impl Into<String>) -> Result<()> {
    todo!("[A1] 1) create empty ZIP; 2) genesis Chunk (bounds + node_0001 parent=genesis); 3) write chunks/chunk_genesis.json; 4) build Manifest + chunk hashes; 5) state_hash §3.2; 6) flush ZIP")
}

pub fn create_db_folder(folder: impl AsRef<std::path::Path>, project_name: impl Into<String>) -> Result<()> {
    todo!("[A2] 1) mkdir chunks/views/assets; 2) chunk_genesis.json; 3) views/default_theme.json placeholder; 4) schema.json placeholder; 5) manifest.json with state_hash; 6) persist")
}

// ── Group B: Load / inspect ───────────────────────────────────────────────

pub fn load_manifest(db_path: impl AsRef<std::path::Path>) -> Result<Manifest> {
    todo!("[B1] 1) detect ZIP vs folder backend; 2) read manifest.json bytes; 3) deserialize → Manifest")
}

pub fn load_chunk(db_path: impl AsRef<std::path::Path>, chunk_id: &str) -> Result<Chunk> {
    todo!("[B2] 1) manifest.chunks[chunk_id] → path + hash; 2) read bytes; 3) verify sha256 vs stored; 4) deserialize → Chunk")
}

pub fn get_node(db_path: impl AsRef<std::path::Path>, node_id: &str) -> Result<Option<Node>> {
    todo!("[B3] 1) manifest.spatial_index[node_id] → chunk_id (None→Ok(None)); 2) load_chunk; 3) return nodes[id].cloned()")
}

pub fn list_nodes(db_path: impl AsRef<std::path::Path>) -> Result<Vec<(String, String, String)>> {
    todo!("[B4] 1) for (cid, meta) in manifest.chunks → load_chunk; 2) push (nid, cid, parent_id); 3) Vec returned")
}

pub fn db_info(db_path: impl AsRef<std::path::Path>) -> Result<serde_json::Value> {
    todo!("[B5] 1) manifest + list_nodes; 2) build summary JSON (name, version, chunks_count, nodes_count, chunks table); 3) return Value")
}

// ── Group C: Node mutations ───────────────────────────────────────────────

pub fn add_node(
    db_path: impl AsRef<std::path::Path>,
    node_id: impl Into<String>,
    parent_id: impl Into<String>,
    coords: Vec<i64>,
    metadata: serde_json::Value,
) -> Result<()> {
    todo!("[C1] → delegate to add_node_to_chunk(db_path, nid, pid, coords, meta, GENESIS_CHUNK_ID)")
}

pub fn add_node_to_chunk(
    db_path: impl AsRef<std::path::Path>,
    node_id: impl Into<String>,
    parent_id: impl Into<String>,
    coords: Vec<i64>,
    metadata: serde_json::Value,
    chunk_id: &str,
) -> Result<()> {
    todo!("[C2] 1) Err if node exists; 2) chunk.nodes.insert; 3) chunk.version++; 4) update .hash in manifest; 5) spatial_index[nid]=chunk_id; 6) global_version++; 7) state_hash §3.2; 8) persist all")
}

pub fn set_node_parent(
    db_path: impl AsRef<std::path::Path>,
    node_id: &str,
    new_parent_id: impl Into<String>,
) -> Result<()> {
    todo!("[C3] 1) resolve node → chunk; 2) ACYCLIC CHECK: walk ancestors(new_parent) — fail if hits node_id; 3) node.parent_id = new; 4) chunk.version++ → hash → global_version++ → state_hash → persist")
}

pub fn set_node_coords(
    db_path: impl AsRef<std::path::Path>,
    node_id: &str,
    new_coords: Vec<i64>,
) -> Result<()> {
    todo!("[C4] 1) resolve; 2) node.coords = new_coords (Vec<i64> only; floats banned by type system); 3) version bump → hash → gv++ → state_hash → persist")
}

pub fn set_node_metadata(
    db_path: impl AsRef<std::path::Path>,
    node_id: &str,
    new_metadata: serde_json::Value,
) -> Result<()> {
    todo!("[C5] 1) resolve; 2) opaque replace node.metadata; 3) version/hash/gv++/state_hash/persist")
}

pub fn patch_node_metadata(
    db_path: impl AsRef<std::path::Path>,
    node_id: &str,
    json_patch_ops: serde_json::Value,
) -> Result<()> {
    todo!("[C6] 1) parse json_patch_ops into json_patch::Patch; 2) json_patch::patch(&mut metadata, &patch); 3) version/hash/gv++/state_hash/persist")
}

pub fn add_relation(
    db_path: impl AsRef<std::path::Path>,
    source_node_id: &str,
    target_node_id: &str,
    target_chunk_id: &str,
    rel_type: impl Into<String>,
) -> Result<()> {
    todo!("[C7] 1) validate rel_type ∈ core_hints.relation_codes or DEFAULT_RELATION_CODES; 2) target must exist via spatial_index[target]==target_chunk_id; 3) push Relation on source; 4) if symmetric → add mirror to target; 5) version/hash/gv++/state_hash/persist")
}

pub fn delete_node(db_path: impl AsRef<std::path::Path>, node_id: &str) -> Result<()> {
    todo!("[C8 §7.1] 1) resolve chunk; 2) DFS children-first ALL transitive descendants across ALL chunks; 3) reverse order → delete each; 4) purge Relations that POINT TO deleted nodes anywhere; 5) if chunk becomes empty → remove file + manifest.chunks entry + spatial_index; 6) global_version++ → state_hash → persist")
}

// ── Group D: Chunk management ─────────────────────────────────────────────

pub fn register_chunk(
    db_path: impl AsRef<std::path::Path>,
    chunk_id: impl Into<String>,
    internal_path: impl Into<String>,
    initial_bounds: Option<[i64; 6]>,
    initial_nodes: HashMap<String, (String, Vec<i64>, serde_json::Value)>,
) -> Result<()> {
    todo!("[D1] 1) Err if chunk_id already in manifest.chunks; 2) build Chunk {version=1,bounds,nodes}; 3) serialize → sha256 → metadata insert; 4) spatial_index for each initial node → chunk_id; 5) write chunks/<new>.json; 6) gv++ → state_hash → persist")
}

pub fn recompute_chunk_hashes(db_path: impl AsRef<std::path::Path>) -> Result<HashMap<String, String>> {
    todo!("[D2] (utility, post manual edits in folder mode) 1) iterate manifest.chunks + views.registered; 2) sha256_hex(read bytes) → write back .hash; 3) recompute_state_hash; 4) return map of id → new hash")
}

// ── Group E: Validation & OCC ─────────────────────────────────────────────

pub fn validate_hashes(db_path: impl AsRef<std::path::Path>) -> Result<Vec<String>> {
    todo!("[E1 §3.2] 1) manifest.chunks → each read_bytes + sha256 vs stored → push err; 2) views.registered same; 3) clone manifest → state_hash=\"\" → sha256_canonical vs stored state_hash; 4) return Vec<String> errors (empty = valid)")
}

pub fn validate_structure(db_path: impl AsRef<std::path::Path>) -> Result<Vec<String>> {
    todo!("[E2 structural-only (NOT profile/semantic)] 1) chunk_genesis exists + has bounds[6] + ≥1 node parent=genesis; 2) every Relation.target_chunk is in manifest.chunks; 3) spatial_index[k] → chunk.nodes has k; 4) NO parent_id cycles (across all chunks); 5) return violations list")
}

pub fn verify_occ(
    db_path: impl AsRef<std::path::Path>,
    ctx: &PatchContext,
) -> std::result::Result<(), AaglError> {
    todo!("[E3 §6] 1) load_manifest; 2) manifest.global_version == ctx.base_global_version; 3) for each (cid, base_v) in ctx.base_chunk_versions → chunks[cid].version == base_v; 4) any mismatch → Err(OccConflict {expected, found}) else Ok(())")
}

// ── Group F: JSON Patch (§7) ──────────────────────────────────────────────

pub fn apply_data_patch(
    db_path: impl AsRef<std::path::Path>,
    patch_json: serde_json::Value,
    ctx: &PatchContext,
) -> Result<()> {
    todo!("[F1 §7 RFC6902] 1) verify_occ(db, ctx) FIRST → bail mismatch; 2) parse patch_json into json_patch::Patch; 3) SCOPE WHITELIST (only /chunks/*, /spatial_index/*, /units/*, /profile/core_hints/relation_codes) BLOCK /views/* /profile/view_style_schema; 4) apply in-memory; 5) delete_node semantics → cascade §7.1; 6) touched chunk.versions++; 7) global_version++; 8) state_hash §3.2; 9) write → flush")
}

// ── Group G: State Hash Explicit ──────────────────────────────────────────

pub fn recompute_state_hash(db_path: impl AsRef<std::path::Path>) -> Result<String> {
    todo!("[G1 §3.2] 1) recompute_chunk_hashes first (always prereq); 2) tmp = manifest.clone(); tmp.state_hash=\"\"; 3) sha256_canonical(tmp) → write back; 4) persist → return hex")
}

// ── Group H: Format Converters (folder ↔ .aagl ZIP) ──────────────────────

pub fn folder_to_aagl(
    folder_src: impl AsRef<std::path::Path>,
    zip_dst: impl AsRef<std::path::Path>,
) -> Result<()> {
    todo!("[H1] 1) walk folder_src recursively → collect (abs_path, relpath_in_container); 2) create empty ZIP; 3) deflate each file under relpath; 4) flush; 5) open with Container + validate_hashes as sanity check")
}

pub fn aagl_to_folder(
    zip_src: impl AsRef<std::path::Path>,
    folder_dst: impl AsRef<std::path::Path>,
) -> Result<()> {
    todo!("[H2] 1) enumerate ZIP entries (skip trailing-slash dirs); 2) mkdir parents; 3) write bytes into folder_dst/relpath; 4) validate_hashes on resulting folder optionally")
}

// ── Group I: View Layer (§8 — mechanism only, NOT rendering) ─────────────

pub fn resolve_style(
    theme: &Theme,
    node_id: &str,
    type_key: Option<&str>,
) -> HashMap<String, serde_json::Value> {
    todo!("[I1 §8.1] 1) out = clone theme.fallback; 2) if let Some(tk)=type_key AND theme.by_type[tk] exists → overwrite each k,v; 3) if theme.by_id[node_id] → overwrite each k,v; 4) return SHALLOW merged (no deep merge, profile decides)")
}

pub fn load_theme(db_path: impl AsRef<std::path::Path>, theme_id: &str) -> Result<Theme> {
    todo!("[I2] 1) manifest.views.registered[theme_id] → path + hash; 2) read bytes; 3) sha256 vs stored; 4) deserialize Theme → return")
}

// ==========================================================================
// FFI LAYER (C ABI) — gated behind feature = "ffi"
// Python ctypes / C# DllImport / C++ consumers:
//   Linux  → libaagl_core.so
//   macOS  → libaagl_core.dylib
//   Windows→ aagl_core.dll
//
// Build commands:
//   cargo build -p aagl-core                (Rust lib only)
//   cargo build -p aagl-core --features ffi (shared lib with C ABI exports)
// ==========================================================================

#[cfg(feature = "ffi")]
pub mod ffi {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_int};

    pub const FFI_OK: c_int = 0;
    pub const FFI_ERR_IO: c_int = 1;
    pub const FFI_ERR_JSON: c_int = 2;
    pub const FFI_ERR_ZIP: c_int = 3;
    pub const FFI_ERR_VALIDATION: c_int = 4;
    pub const FFI_ERR_OCC: c_int = 5;
    pub const FFI_ERR_NODE_NOT_FOUND: c_int = 6;
    pub const FFI_ERR_CHUNK_NOT_FOUND: c_int = 7;
    pub const FFI_ERR_PATCH: c_int = 8;
    pub const FFI_ERR_PANIC: c_int = -1;

    thread_local! {
        static LAST_ERROR: std::cell::RefCell<Option<CString>> = std::cell::RefCell::new(None);
    }

    #[no_mangle] pub extern "C" fn aagl_last_error() -> *const c_char {
        todo!("[FFI 1] LAST_ERROR.with(|c| c.borrow().as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()))")
    }

    #[no_mangle] pub unsafe extern "C" fn aagl_create_db_file(
        zip_path: *const c_char, project_name: *const c_char,
    ) -> c_int {
        todo!("[FFI 2] CStr::from_ptr → Path/Into<String> → call create_db_file → catch_unwind → code via error discriminant → set LAST_ERROR on fail")
    }

    #[no_mangle] pub unsafe extern "C" fn aagl_create_db_folder(
        folder: *const c_char, project_name: *const c_char,
    ) -> c_int {
        todo!("[FFI 3] same wrapper pattern, call create_db_folder")
    }

    #[no_mangle] pub unsafe extern "C" fn aagl_validate_hashes(
        db_path: *const c_char, out_json_errors: *mut *mut c_char,
    ) -> c_int {
        todo!("[FFI 4] call validate_hashes → serde_json::to_string(Vec<String>) → CString into *out_json_errors; CALLER FREES via aagl_free_cstr")
    }

    #[no_mangle] pub unsafe extern "C" fn aagl_add_node_json(
        db_path: *const c_char, node_id: *const c_char, parent_id: *const c_char,
        coords_json: *const c_char, metadata_json: *const c_char,
        out_new_manifest_json: *mut *mut c_char,
    ) -> c_int {
        todo!("[FFI 5] Python-friendly: parse coords_json→Vec<i64>, metadata_json→Value; call add_node; optionally return manifest JSON as C string (out_new_manifest_json)")
    }

    #[no_mangle] pub unsafe extern "C" fn aagl_db_info_json(
        db_path: *const c_char, out_json: *mut *mut c_char,
    ) -> c_int {
        todo!("[FFI 6] call db_info → to_string → CString out; CALLER FREES")
    }

    #[no_mangle] pub unsafe extern "C" fn aagl_apply_data_patch_json(
        db_path: *const c_char, patch_json: *const c_char,
        base_global_version: u64, base_chunk_versions_json: *const c_char,
        out_manifest_json: *mut *mut c_char,
    ) -> c_int {
        todo!("[FFI 7] build PatchContext {base_global_version, parse base_chunk_versions JSON → HashMap<String,u64>}; parse patch_json → Value; call apply_data_patch; output manifest JSON; CALLER FREES")
    }

    #[no_mangle] pub unsafe extern "C" fn aagl_folder_to_aagl(
        folder_src: *const c_char, zip_dst: *const c_char,
    ) -> c_int {
        todo!("[FFI 8] wrapper pattern, call folder_to_aagl")
    }

    #[no_mangle] pub unsafe extern "C" fn aagl_aagl_to_folder(
        zip_src: *const c_char, folder_dst: *const c_char,
    ) -> c_int {
        todo!("[FFI 9] wrapper pattern, call aagl_to_folder")
    }

    #[no_mangle] pub unsafe extern "C" fn aagl_free_cstr(ptr: *mut c_char) {
        todo!("[FFI 10] if !ptr.is_null() { drop(CString::from_raw(ptr)); } — only caller calls this for OUT-parameter strings")
    }
}
