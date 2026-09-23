//! Manifest-level types (§3 of the spec). These are pure data models with no
//! logic — they can be (de)serialized and cloned, but contain zero business
//! rules.
//!
//! Dependency: only `common` (constants for SPEC_VERSION + the Error/Result
//! parent crate are NOT directly used here — this file only uses serde derives).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Profile declaration (manifest §3.1). Core treats most fields as opaque
/// except for the `core_hints` override of allowed relation type codes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: String,
    pub schema_path: String,
    pub view_type_key: String,
    pub core_hints: Option<CoreHints>,
}

/// Hints that the profile gives to the core engine (§3.1 override of defaults).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreHints {
    pub relation_codes: Vec<String>,
}

/// Units configuration (§3). Core stores this but never interprets it.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Units {
    pub base_unit: String,
    pub internal_resolution_to_meters: f64,
}

/// `{path, hash, version}` triple used both for chunks AND for registered themes (§3).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChunkMetadata {
    pub path: String,
    pub hash: String,
    pub version: u64,
}

/// §3.3 Theme registry: active theme id + hash-versioned catalogue.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ViewsRegistry {
    pub active_theme: String,
    pub registered: HashMap<String, ChunkMetadata>,
}

/// `manifest.json` routing table (§3). This struct is serialized on disk —
/// adding fields MUST be backwards-compatible (use `#[serde(default)]` for new fields).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub spec_version: String,
    pub project_name: String,
    /// §3.2 state hash: MUST be computed with field temporarily set to "",
    /// then sha256 of canonical JSON written back.
    pub state_hash: String,
    pub global_version: u64,
    pub profile: Profile,
    pub units: Units,
    pub chunks: HashMap<String, ChunkMetadata>,
    pub views: ViewsRegistry,
    /// node_id → chunk_id lookup. MUST stay in sync with `chunks[*].nodes.keys()`.
    pub spatial_index: HashMap<String, String>,
}
