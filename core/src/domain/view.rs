//! View layer types (§8 theme structure) + OCC patch context.
//! Both are pure data models (no logic — logic lives in infrastructure and application).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// §8.1 Three-tier theme: fallback → per-type → per-id.
/// Merging happens in application code, not here.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Theme {
    pub theme_id: String,
    pub theme_name: String,
    pub spec_version: String,
    #[serde(default)]
    pub fallback: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub by_type: HashMap<String, HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub by_id: HashMap<String, HashMap<String, serde_json::Value>>,
}

/// §6 Optimistic Concurrency Control anchors.
/// Callers submit their "known good" base versions; `verify_occ` compares each
/// field against the on-disk manifest and fails with `AaglError::OccConflict`
/// if ANY anchor drifted.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatchContext {
    pub base_global_version: u64,
    pub base_chunk_versions: HashMap<String, u64>,
}
