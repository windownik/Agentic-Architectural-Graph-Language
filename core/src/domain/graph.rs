//! Universal Graph data model (§5 of the spec): Chunks, 4-field Nodes,
//! 3-field Relations. Pure data; zero logic. Serde derives for JSON IO.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// §5.4 Cross-node link (source node owns this vec, target is addressable via
/// `(target_chunk, target)` pair + spatial_index).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Relation {
    pub target: String,
    pub target_chunk: String,
    pub rel_type: String,
}

/// §5.3 Universal Graph Node: exactly 4 public fields. No extras.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub parent_id: String,
    pub coords: Vec<i64>,
    pub relations: Vec<Relation>,
    pub metadata: serde_json::Value,
}

/// §5 chunks/*.json container: versioned, optionally bounded, map of nodes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub chunk_id: String,
    pub version: u64,
    /// Bounding box `[x_min, y_min, z_min, x_max, y_max, z_max]`
    /// (present on chunk_genesis and spatial-indexed sector chunks).
    pub bounds: Option<[i64; 6]>,
    pub nodes: HashMap<String, Node>,
}
