//! 3 node mutation use cases for v1:
//!   • add_object    — insert a new node into chunk_genesis (always 1 chunk in this version)
//!   • delete_object — remove a node by id (no recursive cascade delete in v1)
//!   • update_object — overwrite coords / metadata / parent_id / relations of an existing node
//!
//! Precondition:  db_path MUST point to a database already created with create_db_file.
//!
//! Invariant: single-chunk, everything lives in chunk_id = GENESIS_CHUNK_ID (§5.2).
//! Manifest spatial_index is NOT maintained in this minimal v1 (will come later with §6 OCC).

use crate::common::{GENESIS_CHUNK_ID, Result};
use crate::common::AaglError;
use crate::domain::{Chunk, Manifest, Node};
use crate::infrastructure::{AnyStorage, StorageBackend, sha256_canonical_json, sha256_hex};
use crate::infrastructure::open_storage::open_storage;
use std::path::Path;

// ── Internal helpers (private to this file) ──────────────────────────────

fn load_manifest_and_chunk(backend: &AnyStorage) -> Result<(Manifest, Chunk)> {
    let manifest: Manifest = backend.read_json("manifest.json")?;
    let genesis_internal_path = manifest
        .chunks
        .get(GENESIS_CHUNK_ID)
        .ok_or_else(|| AaglError::Validation("No chunk_genesis entry in manifest.chunks".into()))?
        .path
        .clone();
    let internal_path = genesis_internal_path.trim_start_matches("./").to_string();
    let chunk: Chunk = backend.read_json(&internal_path)?;
    Ok((manifest, chunk))
}

fn finalize_manifest_with_chunk(
    backend: &mut AnyStorage,
    mut manifest: Manifest,
    mut chunk: Chunk,
) -> Result<()> {
    // 1) chunk version bump (per-chunk monotonic anchor for OCC §6 later)
    chunk.version = chunk.version.checked_add(1)
        .ok_or_else(|| AaglError::Validation("chunk version overflow u64".into()))?;

    let meta = manifest.chunks.get_mut(GENESIS_CHUNK_ID)
        .ok_or_else(|| AaglError::Validation("manifest.chunks missing chunk_genesis".into()))?;
    let internal_path = meta.path.trim_start_matches("./").to_string();

    // 2) Persist chunk as pretty JSON → read back bytes for canonical hash
    backend.write_json(&internal_path, &chunk)?;
    let chunk_bytes = backend.read_file(&internal_path)?;
    meta.hash = sha256_hex(&chunk_bytes);
    meta.version = chunk.version;

    // 3) global_version bump on manifest
    manifest.global_version = manifest.global_version.checked_add(1)
        .ok_or_else(|| AaglError::Validation("global_version overflow u64".into()))?;

    // 4) state_hash recompute per §3.2
    manifest.state_hash = String::new();
    manifest.state_hash = sha256_canonical_json(&manifest)?;

    // 5) persist manifest + flush to make durable (ZIP rebuilds here)
    backend.write_json("manifest.json", &manifest)?;
    backend.flush()?;
    Ok(())
}

// ── Public 3 API (Anchor-style: path is first arg) ───────────────────────

/// Insert a NEW object into chunk_genesis.
/// Returns the assigned `node_id` (e.g. "wall_0002" when prefix is "wall_",
/// "node_0003" when prefix is "").
pub fn add_object(
    db_path: impl AsRef<Path>,
    parent_id: impl Into<String>,
    coords: Vec<i64>,
    metadata: serde_json::Value,
    relations: Vec<crate::domain::Relation>,
    suggested_id_prefix: impl AsRef<str>,
) -> Result<String> {
    let mut backend = open_storage(db_path.as_ref())?;
    let (manifest, mut chunk) = load_manifest_and_chunk(&backend)?;

    let prefix = suggested_id_prefix.as_ref();
    let prefix = if prefix.is_empty() { "node_" } else { prefix };
    let existing_numbers: Vec<u32> = chunk.nodes.keys()
        .filter_map(|k| k.strip_prefix(prefix).and_then(|digits| digits.parse::<u32>().ok()))
        .collect();
    let next_n = existing_numbers.iter().max().copied().unwrap_or(0) + 1;
    let node_id = format!("{prefix}{next_n:04}");

    if chunk.nodes.contains_key(&node_id) {
        return Err(AaglError::Validation(format!("generated id collides — existing: {node_id}")));
    }
    chunk.nodes.insert(node_id.clone(), Node {
        parent_id: parent_id.into(),
        coords,
        relations,
        metadata,
    });

    finalize_manifest_with_chunk(&mut backend, manifest, chunk)?;
    Ok(node_id)
}

/// Delete an object by node_id. In this minimal v1 we do a SINGLE-level delete only
/// (no recursive children cascade §7.1 — caller must delete children first).
/// Returns error if node has children (parent_id pointing at it) to avoid orphan dangles,
/// or if you try to delete the genesis root "node_0001".
pub fn delete_object(
    db_path: impl AsRef<Path>,
    node_id: &str,
) -> Result<()> {
    if node_id == "node_0001" {
        return Err(AaglError::Validation("cannot delete genesis root node_0001".into()));
    }
    let mut backend = open_storage(db_path.as_ref())?;
    let (manifest, mut chunk) = load_manifest_and_chunk(&backend)?;

    if !chunk.nodes.contains_key(node_id) {
        return Err(AaglError::NodeNotFound(node_id.to_string()));
    }
    let has_children = chunk.nodes.values().any(|n| n.parent_id == node_id);
    if has_children {
        return Err(AaglError::Validation(format!(
            "node {node_id} still has parent-referencing children — delete children first (v1 no cascade)"
        )));
    }
    chunk.nodes.remove(node_id);

    // Also purge any dangling RELATIONS pointing to the deleted node (cross references cleanup)
    for n in chunk.nodes.values_mut() {
        n.relations.retain(|r| r.target != node_id);
    }

    finalize_manifest_with_chunk(&mut backend, manifest, chunk)?;
    Ok(())
}

/// Update an existing node in-place (overwrite coords / parent_id / relations / metadata).
///
/// Pass `None` for a field to leave it unchanged; pass `Some(new_value)` to replace.
pub fn update_object(
    db_path: impl AsRef<Path>,
    node_id: &str,
    new_parent_id: Option<String>,
    new_coords: Option<Vec<i64>>,
    new_relations: Option<Vec<crate::domain::Relation>>,
    new_metadata: Option<serde_json::Value>,
) -> Result<()> {
    let mut backend = open_storage(db_path.as_ref())?;
    let (manifest, mut chunk) = load_manifest_and_chunk(&backend)?;

    let node = chunk.nodes.get_mut(node_id)
        .ok_or_else(|| AaglError::NodeNotFound(node_id.to_string()))?;

    if let Some(v) = new_parent_id   { node.parent_id = v; }
    if let Some(v) = new_coords      { node.coords    = v; }
    if let Some(v) = new_relations   { node.relations = v; }
    if let Some(v) = new_metadata    { node.metadata  = v; }

    finalize_manifest_with_chunk(&mut backend, manifest, chunk)?;
    Ok(())
}
