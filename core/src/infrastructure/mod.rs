//! Infrastructure layer (Clean Architecture: "IO details" / driven adapters).
//!
//! Dependency rule:
//!   infrastructure → common (error/result)
//!   infrastructure → domain (pure data models it needs to serialize/deserialize + hash)
//!
//! NEVER let infrastructure depend on application layer.
//!
//! Public sub-modules:
//!   * `hash`    — sha256_hex / sha256_canonical_json helpers
//!   * `storage` — unified StorageBackend trait + FolderBackend / ZipBackend impls + zip_folder()

pub mod hash;
pub mod storage;

// ─── Flat re-exports (most common usage).
pub use hash::{sha256_canonical_json, sha256_hex};
pub use storage::{FolderBackend, StorageBackend, ZipBackend, zip_folder};
