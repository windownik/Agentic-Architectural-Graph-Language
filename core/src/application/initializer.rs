//! Application Use Case: `create_empty_project(db_path, project_name, folder_mode, genesis_space_size)`.
//!
//! This is the ONLY public routine exposed from this file — the entire rest are
//! private defaults for content. It orchestrates the two lower layers:
//!
//!   1. **Always build project content inside a TEMP FOLDER first** (single
//!      code path; avoids duplicating logic).
//!   2. **Finalize according to `folder_mode`**:
//!        - true  → atomic rename/copy tempdir → target (plain folder mode)
//!        - false → `zip_folder` tempdir → `.aagl` file then drop tempdir

use crate::common::{AaglError, Result};
use crate::common::constants::*;
use crate::domain::{Chunk, ChunkMetadata, Manifest, Node, Profile, CoreHints, Theme, Units, ViewsRegistry};
use crate::infrastructure::{sha256_canonical_json, sha256_hex, zip_folder, FolderBackend, StorageBackend};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ==========================================================================
// Default content builders (private — purely determinstic based on inputs)
// ==========================================================================

fn default_genesis_chunk(space_size: i64) -> Chunk {
    let half = space_size / 2;
    let coords: Vec<i64> = vec![-half, -half, -half, half, half, half];
    let bounds: [i64; 6] = [-half, -half, -half, half, half, half];
    let nodes = HashMap::from([(
        "node_0001".to_string(),
        Node {
            parent_id: GENESIS_ROOT_PARENT.to_string(),
            coords,
            relations: vec![],
            metadata: json!({
                "profile_key": "container_root",
                "label": format!("Global Envelope (space {} units)", space_size),
                "space_size": space_size,
            }),
        },
    )]);
    Chunk { chunk_id: GENESIS_CHUNK_ID.into(), version: 1, bounds: Some(bounds), nodes }
}

fn default_theme() -> Theme {
    Theme {
        theme_id: DEFAULT_THEME_ID.into(),
        theme_name: "Default Minimal".into(),
        spec_version: SPEC_VERSION.into(),
        fallback: HashMap::from([
            ("stroke_width".into(), json!(2)),
            ("opacity".into(), json!(1.0)),
            ("color".into(), json!("#444444")),
        ]),
        by_type: HashMap::from([(
            "container_root".into(),
            HashMap::from([
                ("color".into(), json!("#2563eb")),
                ("stroke_dash".into(), json!([6, 4])),
            ]),
        )]),
        by_id: HashMap::new(),
    }
}

fn placeholder_schema_json() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "aagl://profiles/generic_spatial_v0/schema.json",
        "title": "Generic Spatial Profile (Placeholder Schema)",
        "description": "Override this with your profile schema. Core never validates against it.",
        "type": "object"
    })
}

fn default_manifest(project_name: String, space_size: i64) -> Manifest {
    let profile = Profile {
        id: DEFAULT_PROFILE_ID.into(),
        schema_path: "./schema.json".into(),
        view_type_key: DEFAULT_VIEW_TYPE_KEY.into(),
        core_hints: Some(CoreHints {
            relation_codes: DEFAULT_RELATION_CODES.iter().map(|s| s.to_string()).collect(),
        }),
    };
    let units = Units {
        base_unit: DEFAULT_BASE_UNIT.into(),
        internal_resolution_to_meters: DEFAULT_RESOLUTION,
    };
    let empty_meta = ChunkMetadata {
        path: "./chunks/chunk_genesis.json".into(),
        hash: String::new(),
        version: 1,
    };
    let empty_theme_meta = ChunkMetadata {
        path: "./views/default_theme.json".into(),
        hash: String::new(),
        version: 1,
    };
    let _ = space_size / 2;
    Manifest {
        spec_version: SPEC_VERSION.into(),
        project_name,
        state_hash: String::new(),
        global_version: 1,
        profile,
        units,
        chunks: HashMap::from([(GENESIS_CHUNK_ID.into(), empty_meta)]),
        views: ViewsRegistry {
            active_theme: DEFAULT_THEME_ID.into(),
            registered: HashMap::from([(DEFAULT_THEME_ID.into(), empty_theme_meta)]),
        },
        spatial_index: HashMap::from([("node_0001".into(), GENESIS_CHUNK_ID.into())]),
    }
}

// ==========================================================================
// Private: populate a folder root with all required files
// ==========================================================================

fn build_folder_tree(root: &Path, project_name: &str, genesis_space_size: i64) -> Result<()> {
    let mut backend = FolderBackend::create(root)?;

    // 1. Subdirs (FolderBackend::write_json auto-creates on write; mkdirs here for clarity)
    fs::create_dir_all(root.join("chunks"))?;
    fs::create_dir_all(root.join("views"))?;
    fs::create_dir_all(root.join("assets"))?;

    // 2. Genesis chunk
    let genesis = default_genesis_chunk(genesis_space_size);
    backend.write_json("chunks/chunk_genesis.json", &genesis)?;

    // 3. Default theme
    let theme = default_theme();
    backend.write_json("views/default_theme.json", &theme)?;

    // 4. Placeholder schema.json
    backend.write_json("schema.json", &placeholder_schema_json())?;

    // 5. First pass manifest (hash fields will be overwritten in step 6)
    let mut manifest = default_manifest(project_name.to_string(), genesis_space_size);

    // 6. Compute chunk + theme hashes from what was ACTUALLY written to disk.
    let genesis_bytes = backend.read_file("chunks/chunk_genesis.json")?;
    manifest.chunks.get_mut(GENESIS_CHUNK_ID).unwrap().hash = sha256_hex(&genesis_bytes);
    let theme_bytes = backend.read_file("views/default_theme.json")?;
    manifest
        .views
        .registered
        .get_mut(DEFAULT_THEME_ID)
        .unwrap()
        .hash = sha256_hex(&theme_bytes);

    // 7. State hash per §3.2 (null field → hash compact → write back)
    manifest.state_hash = String::new();
    manifest.state_hash = sha256_canonical_json(&manifest)?;

    // 8. Persist final manifest
    backend.write_json("manifest.json", &manifest)?;
    backend.flush()?;
    Ok(())
}

// ==========================================================================
// Public use-case entry (called from thin wrapper in lib.rs).
// ==========================================================================

pub fn create_empty_project(
    db_path: &Path,
    project_name: &str,
    folder_mode: bool,
    genesis_space_size: i64,
) -> Result<()> {
    // ─── Preconditions
    if genesis_space_size < 2 {
        return Err(AaglError::Validation(
            "genesis_space_size must be >=2 (so half size is non-zero)".into(),
        ));
    }
    if db_path.exists() {
        if folder_mode {
            fs::remove_dir_all(db_path).map_err(|e| {
                AaglError::Validation(format!(
                    "Target folder exists and cannot be removed: {e}"
                ))
            })?;
        } else {
            fs::remove_file(db_path).map_err(|e| {
                AaglError::Validation(format!(
                    "Target .aagl file exists and cannot be removed: {e}"
                ))
            })?;
        }
    }
    if let Some(parent) = db_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    // ─── Stage 1: always build the full project in a throwaway tempdir
    let tmp_root = tempfile::tempdir().map_err(AaglError::Io)?;
    build_folder_tree(tmp_root.path(), project_name, genesis_space_size)?;

    // ─── Stage 2: finalize per mode
    if folder_mode {
        let result = fs::rename(tmp_root.path(), db_path);
        match result {
            Ok(()) => {
                // Disarm tempfile RAII so drop() won't try to delete a moved path
                #[allow(deprecated)]
                let _ = tmp_root.into_path();
                Ok(())
            }
            Err(_) => {
                // Cross-filesystem rename fallback → recursive copy
                fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
                    fs::create_dir_all(dst)?;
                    for entry in fs::read_dir(src)? {
                        let e = entry?;
                        let ft = e.file_type()?;
                        let s = e.path();
                        let d = dst.join(e.file_name());
                        if ft.is_dir() {
                            copy_dir(&s, &d)?;
                        } else {
                            fs::copy(&s, &d)?;
                        }
                    }
                    Ok(())
                }
                copy_dir(tmp_root.path(), db_path)?;
                Ok(())
            }
        }
    } else {
        // Zip mode — turn tempdir → aagl ZIP, tempdir is cleaned up by RAII drop
        zip_folder(tmp_root.path(), db_path)
    }
}

// ==========================================================================
// Unit tests (2/2 — unit-style; integration 10/10 live in tests/create_db_file_tests.rs).
// ==========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_GENESIS_SPACE_SIZE;
    use crate::infrastructure::sha256_hex;

    #[test]
    fn hashes_are_expected_format() {
        let h = sha256_hex(b"hello");
        assert!(h.starts_with("sha256:"), "must start with prefix sha256:, got {h}");
        assert_eq!(h.len(), 7 + 64, "sha256 prefix + 64 hex = 71 chars total");
    }

    #[test]
    fn default_genesis_has_correct_bounds() {
        let c = default_genesis_chunk(DEFAULT_GENESIS_SPACE_SIZE);
        let half = DEFAULT_GENESIS_SPACE_SIZE / 2;
        assert_eq!(c.bounds, Some([-half, -half, -half, half, half, half]));
        let n = c.nodes.get("node_0001").expect("node_0001 present");
        assert_eq!(n.parent_id, GENESIS_ROOT_PARENT, "root node.parent_id == literal genesis");
        let meta_space = n.metadata.get("space_size").and_then(|m| m.as_i64());
        assert_eq!(meta_space, Some(DEFAULT_GENESIS_SPACE_SIZE));
    }
}
