//! Helper: auto-open a StorageBackend (Folder or ZIP) based on what the path actually is.
//!
//! Rule matches initializer behavior:
//!   * If `db_path` is a directory             → FolderBackend
//!   * If `db_path` is a file ending in .aagl  → ZipBackend
//!   * Otherwise the path does not exist yet   → pick based on CREATE_AS_FOLDER toggle.
//!
//! Returns an `AnyStorage` enum (Folder/Zip variants) because trait `StorageBackend`
//! has generic methods `read_json<T>` / `write_json<T>` → NOT dyn-object-safe.
//! Static dispatch via enum match is zero-cost and fully generic-compatible.

use crate::common::{CREATE_AS_FOLDER, Result};
use crate::infrastructure::{AnyStorage, FolderBackend, ZipBackend};
use std::fs;
use std::path::Path;

pub fn open_storage(db_path: &Path) -> Result<AnyStorage> {
    if db_path.exists() {
        let meta = fs::metadata(db_path)?;
        if meta.is_dir() {
            Ok(AnyStorage::Folder(FolderBackend::open(db_path)?))
        } else {
            // File on disk → assume .aagl ZIP per convention.
            Ok(AnyStorage::Zip(ZipBackend::open(db_path)?))
        }
    } else {
        // Path doesn't exist yet → create a fresh backend using the default mode.
        if CREATE_AS_FOLDER {
            Ok(AnyStorage::Folder(FolderBackend::create(db_path)?))
        } else {
            Ok(AnyStorage::Zip(ZipBackend::create(db_path)?))
        }
    }
}
