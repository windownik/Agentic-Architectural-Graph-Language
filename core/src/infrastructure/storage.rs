//! Persistence backends (§2 container format).
//!
//! Two backends implement the exact same IO operations, so application-level
//! code never has to care if the DB is a plain folder or a `.aagl` ZIP file:
//!   * `FolderBackend` — reads/writes JSON directly to disk (git/dev mode).
//!   * `ZipBackend`    — reads/writes entries inside an in-memory ZipWriter that
//!                       is flushed atomically at `flush()` (user document mode).
//!
//! Both backends use the unified trait `StorageBackend` so application code can
//! be generic over either one.

use crate::common::{AaglError, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;

// ==========================================================================
// Helper — internal path normalizer (strips "./" and leading "/" prefixes
// so manifest paths like "./chunks/chunk_genesis.json" work in both backends).
// ==========================================================================

fn normalize_internal(internal: &str) -> String {
    internal
        .trim_start_matches("./")
        .trim_start_matches('/')
        .replace('\\', "/")
}

// ==========================================================================
// Trait: every storage backend implements these 6 primitives.
// ==========================================================================

pub trait StorageBackend {
    /// Absolute or relative on-disk path of the target (folder or .aagl file).
    fn db_path(&self) -> &Path;

    /// Read a file from inside the container as raw bytes.
    fn read_file(&self, internal_path: &str) -> Result<Vec<u8>>;

    /// Stage a file write (in-memory) — `flush()` makes it durable on disk.
    fn write_file(&mut self, internal_path: &str, bytes: &[u8]) -> Result<()>;

    /// Does the path currently exist inside the container?
    fn file_exists(&self, internal_path: &str) -> Result<bool>;

    /// Atomically persist all staged writes to disk. Until this is called
    /// writes are NOT guaranteed to be durable (ZIP rebuilds happen here).
    fn flush(&mut self) -> Result<()>;

    // ─── JSON helpers (default impls on top of the 4 primitives above) ──

    fn read_json<T: DeserializeOwned>(&self, internal_path: &str) -> Result<T> {
        let bytes = self.read_file(internal_path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn write_json<T: Serialize>(&mut self, internal_path: &str, val: &T) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(val)?;
        self.write_file(internal_path, &bytes)
    }
}

// ==========================================================================
// FolderBackend — direct JSON files on disk (dev / git mode)
// ==========================================================================

pub struct FolderBackend {
    root: PathBuf,
}

impl FolderBackend {
    pub fn open(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(AaglError::Validation(format!(
                "Folder DB path does not exist: {}",
                path.display()
            )));
        }
        if !path.is_dir() {
            return Err(AaglError::Validation(format!(
                "FolderBackend path is not a directory: {}",
                path.display()
            )));
        }
        Ok(Self { root: path.to_path_buf() })
    }

    pub fn create(path: &Path) -> Result<Self> {
        if path.exists() {
            if path.is_dir() {
                // Keep existing — callers may have pre-created with partial content.
            } else {
                return Err(AaglError::Validation(format!(
                    "Target for FolderBackend exists but is a file: {}",
                    path.display()
                )));
            }
        } else {
            fs::create_dir_all(path)?;
        }
        Ok(Self { root: path.to_path_buf() })
    }

    fn resolve(&self, internal: &str) -> PathBuf {
        self.root.join(normalize_internal(internal))
    }
}

impl StorageBackend for FolderBackend {
    fn db_path(&self) -> &Path {
        &self.root
    }

    fn read_file(&self, internal_path: &str) -> Result<Vec<u8>> {
        Ok(fs::read(self.resolve(internal_path))?)
    }

    fn write_file(&mut self, internal_path: &str, bytes: &[u8]) -> Result<()> {
        let full = self.resolve(internal_path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(fs::write(&full, bytes)?)
    }

    fn file_exists(&self, internal_path: &str) -> Result<bool> {
        Ok(self.resolve(internal_path).exists())
    }

    fn flush(&mut self) -> Result<()> {
        // Folder writes are synchronous. Flush is a no-op but callers
        // should still invoke it for backend-agnosticism (ZipBackend requires it).
        Ok(())
    }
}

// ==========================================================================
// ZipBackend — single-file `.aagl` container (zip::ZipWriter, atomic flush)
// ==========================================================================

pub struct ZipBackend {
    target_path: PathBuf,
    pending: std::collections::HashMap<String, Vec<u8>>,
}

impl ZipBackend {
    pub fn open(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(AaglError::Validation(format!(
                "ZIP DB path does not exist: {}",
                path.display()
            )));
        }
        // Validates ZIP integrity by opening the archive and reading the manifest.
        let file = fs::File::open(path)?;
        let _archive = zip::ZipArchive::new(file)?;
        Ok(Self { target_path: path.to_path_buf(), pending: Default::default() })
    }

    pub fn create(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        // Start with an empty pending map; `flush()` writes the first ZIP.
        Ok(Self { target_path: path.to_path_buf(), pending: Default::default() })
    }
}

impl StorageBackend for ZipBackend {
    fn db_path(&self) -> &Path {
        &self.target_path
    }

    fn read_file(&self, internal_path: &str) -> Result<Vec<u8>> {
        let key = normalize_internal(internal_path);
        if let Some(cached) = self.pending.get(&key) {
            return Ok(cached.clone());
        }
        // Not in pending writes: read through the on-disk ZIP.
        let file = fs::File::open(&self.target_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut zf = archive.by_name(&key)?;
        let mut buf = Vec::with_capacity(zf.size() as usize);
        zf.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn write_file(&mut self, internal_path: &str, bytes: &[u8]) -> Result<()> {
        self.pending.insert(normalize_internal(internal_path), bytes.to_vec());
        Ok(())
    }

    fn file_exists(&self, internal_path: &str) -> Result<bool> {
        let key = normalize_internal(internal_path);
        if self.pending.contains_key(&key) {
            return Ok(true);
        }
        if !self.target_path.exists() {
            return Ok(false);
        }
        let file = fs::File::open(&self.target_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let x = archive.by_name(&key).is_ok();
        Ok(x)
    }

    fn flush(&mut self) -> Result<()> {
        // 1. Seed the output with the full on-disk ZIP (if any) as a HashMap →
        //    unmodified files flow through unchanged.
        let mut final_files: std::collections::BTreeMap<String, Vec<u8>> =
            std::collections::BTreeMap::new();
        if self.target_path.exists() {
            let file = fs::File::open(&self.target_path)?;
            let mut archive = zip::ZipArchive::new(file)?;
            for i in 0..archive.len() {
                let mut zf = archive.by_index(i)?;
                let name = zf.name().to_string();
                if name.ends_with('/') {
                    continue; // directory entries are re-emitted based on file paths
                }
                let mut buf = Vec::with_capacity(zf.size() as usize);
                zf.read_to_end(&mut buf)?;
                final_files.insert(name, buf);
            }
        }
        // 2. Overlay pending writes (new / modified)
        for (k, v) in self.pending.drain() {
            final_files.insert(k, v);
        }
        // 3. Rebuild ZIP entirely from scratch with deterministic ordering.
        let tmp = self.target_path.with_extension("aagl.tmp");
        if tmp.exists() {
            fs::remove_file(&tmp).ok();
        }
        {
            let file = fs::File::create(&tmp)?;
            let mut zw = zip::ZipWriter::new(file);
            let options = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o644);
            for (rel, data) in &final_files {
                // Write explicit directory entries for any path segments
                let parts: Vec<&str> = rel.split('/').collect();
                for i in 1..parts.len() {
                    let dir_chain = parts[0..i].join("/") + "/";
                    zw.add_directory(dir_chain, options)?;
                }
                zw.start_file(rel, options)?;
                zw.write_all(data)?;
            }
            zw.finish()?;
        }
        // 4. Atomic rename — readers never see a half-written .aagl
        fs::rename(&tmp, &self.target_path)?;
        Ok(())
    }
}

// ==========================================================================
// Helpers used by the initializer: turn a populated folder → a zip file.
// ==========================================================================

/// Walk `src_folder` recursively and DEFLATE-everything into a brand-new
/// `.aagl` ZIP file at `dst_aagl`. Used by initializer (folder → zip mode).
pub fn zip_folder(src_folder: &Path, dst_aagl: &Path) -> Result<()> {
    let file = fs::File::create(dst_aagl)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    let mut stack: Vec<PathBuf> = Vec::new();
    stack.push(src_folder.to_path_buf());
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            let p = entry.path();
            let rel = p.strip_prefix(src_folder).unwrap().to_path_buf();
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if ft.is_dir() {
                zip.add_directory(rel_str.clone(), options)?;
                stack.push(p);
            } else if ft.is_file() {
                zip.start_file(rel_str, options)?;
                let bytes = fs::read(&p)?;
                zip.write_all(&bytes)?;
            }
        }
    }
    zip.finish()?;
    Ok(())
}
