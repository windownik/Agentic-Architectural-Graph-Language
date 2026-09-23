//! Unified error enum for every fallible call in `aagl-core`,
//! plus the public `Result<T>` type alias used crate-wide (and FFI-wide).

use thiserror::Error;

/// Return type alias (drop-in replacement for `std::result::Result<T>`).
pub type Result<T> = std::result::Result<T, AaglError>;

#[derive(Error, Debug)]
pub enum AaglError {
    /// Anything that returns `std::io::Error` (file not found, permissions, …)
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),

    /// `serde_json` serialize / deserialize failures.
    #[error("JSON: {0}")]
    Serialization(#[from] serde_json::Error),

    /// `zip` crate errors (corrupt archive, missing entry, …)
    #[error("ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Any semantic validation (invalid params, structural checks, bad data).
    #[error("Validation: {0}")]
    Validation(String),

    /// §6 Optimistic Concurrency Control anchor mismatch
    /// (either manifest.global_version or a chunk.version mismatch).
    #[error(
        "OCC conflict: expected v{expected} (caller's base anchor) \
         but database already at v{found}"
    )]
    OccConflict { expected: u64, found: u64 },

    /// Lookup by node_id failed (not in spatial_index or not present in owning chunk).
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Lookup by chunk_id failed (not registered in manifest.chunks).
    #[error("Chunk not found: {0}")]
    ChunkNotFound(String),

    /// §7 JSON Patch / RFC 6902 errors (op path not found, test op failed, …).
    #[error("Patch: {0}")]
    Patch(String),
}
