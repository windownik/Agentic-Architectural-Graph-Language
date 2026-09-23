//! Anchor / FFI-style command signatures (application layer use cases, stateless).
//! Each function below takes a DB path as its FIRST argument and coordinates
//! the domain + infrastructure layers to perform a user-visible operation.
//!
//! Current status: PLACEHOLDER signatures with todo!() bodies so the public API
//! surface is visible and future-implementation-ready. When we get to these
//! they'll each be ~20-60 lines orchestrating the layers below.

use crate::common::Result;
use crate::domain::{Chunk, Manifest, Node, PatchContext};
use std::collections::HashMap;
use std::path::Path;

// ── Load / inspect ──────────────────────────────────────────────────────

pub fn load_manifest(db_path: impl AsRef<Path>) -> Result<Manifest> {
    todo!("load_manifest: select backend by path → backend.read_json(\"manifest.json\")")
}

pub fn load_chunk(db_path: impl AsRef<Path>, chunk_id: &str) -> Result<Chunk> {
    todo!("load_chunk: manifest.chunks[id].path → backend.read_json(path)")
}

pub fn get_node(db_path: impl AsRef<Path>, node_id: &str) -> Result<Option<Node>> {
    todo!("get_node: manifest.spatial_index[node_id] → load_chunk → .nodes[id].cloned()")
}

pub fn list_nodes(db_path: impl AsRef<Path>) -> Result<Vec<(String, String, String)>> {
    todo!("list_nodes: for (cid,meta) in manifest.chunks → load_chunk → (nid,cid,parent_id)")
}

pub fn db_info(db_path: impl AsRef<Path>) -> Result<serde_json::Value> {
    todo!("db_info: manifest + list_nodes → summary Value")
}

// ── Node mutations ──────────────────────────────────────────────────────

pub fn add_node(
    db_path: impl AsRef<Path>,
    node_id: impl Into<String>,
    parent_id: impl Into<String>,
    coords: Vec<i64>,
    metadata: serde_json::Value,
) -> Result<()> {
    todo!("add_node: → add_node_to_chunk with GENESIS_CHUNK_ID")
}

pub fn add_node_to_chunk(
    db_path: impl AsRef<Path>,
    node_id: impl Into<String>,
    parent_id: impl Into<String>,
    coords: Vec<i64>,
    metadata: serde_json::Value,
    chunk_id: &str,
) -> Result<()> {
    todo!("[orchestration] load manifest+chunk → insert node → chunk.version++ → hash → global_version++ → state_hash → persist")
}

pub fn set_node_parent(db_path: impl AsRef<Path>, node_id: &str, new_parent_id: impl Into<String>) -> Result<()> {
    todo!("[acyclic-check] walk ancestors of new_parent; mutate node; persist")
}

pub fn set_node_coords(db_path: impl AsRef<Path>, node_id: &str, new_coords: Vec<i64>) -> Result<()> {
    todo!()
}

pub fn set_node_metadata(db_path: impl AsRef<Path>, node_id: &str, new_metadata: serde_json::Value) -> Result<()> {
    todo!()
}

pub fn patch_node_metadata(db_path: impl AsRef<Path>, node_id: &str, json_patch_ops: serde_json::Value) -> Result<()> {
    todo!()
}

pub fn add_relation(
    db_path: impl AsRef<Path>,
    source_node_id: &str,
    target_node_id: &str,
    target_chunk_id: &str,
    rel_type: impl Into<String>,
) -> Result<()> {
    todo!("[validate rel_type ∈ core_hints defaults] → push Relation; symmetric = push mirror")
}

pub fn delete_node(db_path: impl AsRef<Path>, node_id: &str) -> Result<()> {
    todo!("§7.1 DFS children-first cascade delete across ALL chunks; purge dead relations; prune empty chunks")
}

// ── Chunk management ──────────────────────────────────────────────────

pub fn register_chunk(
    db_path: impl AsRef<Path>,
    chunk_id: impl Into<String>,
    internal_path: impl Into<String>,
    initial_bounds: Option<[i64; 6]>,
    initial_nodes: HashMap<String, (String, Vec<i64>, serde_json::Value)>,
) -> Result<()> {
    todo!()
}

pub fn recompute_chunk_hashes(db_path: impl AsRef<Path>) -> Result<HashMap<String, String>> {
    todo!()
}

// ── Validation & OCC ──────────────────────────────────────────────────

pub fn validate_hashes(db_path: impl AsRef<Path>) -> Result<Vec<String>> {
    todo!("[E1] sha256 each chunk/view file, compare against metadata; verify state_hash")
}

pub fn validate_structure(db_path: impl AsRef<Path>) -> Result<Vec<String>> {
    todo!("[E2] genesis chunk/bounds, spatial_index consistency, parent cycles, relation.target_chunk refs")
}

pub fn verify_occ(db_path: impl AsRef<Path>, ctx: &PatchContext) -> crate::common::Result<()> {
    todo!("[E3 §6] manifest.global_version == ctx.base; chunks[cid].version == ctx.base[cid] else OccConflict")
}

// ── JSON Patch RFC 6902 ────────────────────────────────────────────────

pub fn apply_data_patch(
    db_path: impl AsRef<Path>,
    patch_json: serde_json::Value,
    ctx: &PatchContext,
) -> Result<()> {
    todo!("[F1 §7] 1) verify_occ FIRST; 2) parse Patch; 3) scope-whitelist; 4) in-mem apply; 5) cascade delete; 6) versions++; 7) state_hash; 8) persist")
}

// ── State hash ─────────────────────────────────────────────────────────

pub fn recompute_state_hash(db_path: impl AsRef<Path>) -> Result<String> {
    todo!("[G1 §3.2] recompute chunk/view hashes FIRST; null state_hash → sha256 → write back → return hex")
}

// ── Format converters ──────────────────────────────────────────────────

pub fn folder_to_aagl(folder_src: impl AsRef<Path>, zip_dst: impl AsRef<Path>) -> Result<()> {
    todo!("[H1] zip_folder wrapper with precondition checks")
}

pub fn aagl_to_folder(zip_src: impl AsRef<Path>, folder_dst: impl AsRef<Path>) -> Result<()> {
    todo!("[H2] enumerate zip entries → write to folder; validate_hashes afterwards")
}

// ── View layer (§8 — mechanism, not rendering) ────────────────────────

pub fn resolve_style(
    theme: &crate::domain::Theme,
    node_id: &str,
    type_key: Option<&str>,
) -> HashMap<String, serde_json::Value> {
    todo!("[I1 §8.1] 3-tier merge per key: fallback → by_type[type_key] → by_id[node_id]")
}

pub fn load_theme(db_path: impl AsRef<Path>, theme_id: &str) -> Result<crate::domain::Theme> {
    todo!("[I2] manifest.views.registered[id].path → backend.read_json")
}
