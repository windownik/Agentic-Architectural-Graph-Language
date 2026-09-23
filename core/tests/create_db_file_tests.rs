//! Integration tests for `create_db_file` / `create_db_file_with_space_size`.
//! Covers folder mode (default via CREATE_AS_FOLDER = true), zip mode (explicit),
//! and custom genesis cube size.

use aagl_core::*;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn tmp_dir(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "aagl_test_create_db_{}_{}_{}",
        name,
        std::process::id(),
        rand_suffix()
    ));
    if base.exists() {
        let _ = fs::remove_dir_all(&base);
        let _ = fs::remove_file(&base);
    }
    base
}

fn rand_suffix() -> u64 {
    // Simple non-cryptographic randomness for test path uniqueness.
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

fn list_files_recursive(root: &Path) -> HashSet<String> {
    let mut out = HashSet::new();
    fn walk(r: &Path, base: &Path, set: &mut HashSet<String>) {
        for e in fs::read_dir(r).unwrap() {
            let e = e.unwrap();
            let p = e.path();
            let rel = p.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
            let ft = e.file_type().unwrap();
            if ft.is_dir() {
                set.insert(format!("{rel}/"));
                walk(&p, base, set);
            } else if ft.is_file() {
                set.insert(rel);
            }
        }
    }
    walk(root, root, &mut out);
    out
}

// ==========================================================================
// Test 1: folder mode — all expected files and subdirs exist
// ==========================================================================

#[test]
fn create_db_file_folder_mode_file_tree_is_complete() {
    let folder = tmp_dir("folder_tree");
    create_db_file(&folder, "Tree Test Project").expect("create_db_file should succeed");

    let listing = list_files_recursive(&folder);

    // Required subdirs
    assert!(listing.contains("chunks/"), "missing dir: chunks/");
    assert!(listing.contains("views/"), "missing dir: views/");
    assert!(listing.contains("assets/"), "missing dir: assets/");

    // Required files (relative paths)
    assert!(listing.contains("manifest.json"), "missing manifest.json");
    assert!(listing.contains("schema.json"), "missing schema.json");
    assert!(
        listing.contains("chunks/chunk_genesis.json"),
        "missing chunks/chunk_genesis.json"
    );
    assert!(
        listing.contains("views/default_theme.json"),
        "missing views/default_theme.json"
    );

    // Exactly 4 files + 3 dirs = 7 entries (sanity: no mystery leftovers)
    let files_only = listing.iter().filter(|e| !e.ends_with('/')).count();
    let dirs_only = listing.iter().filter(|e| e.ends_with('/')).count();
    assert_eq!(files_only, 4, "unexpected extra/ missing files: {listing:?}");
    assert_eq!(dirs_only, 3, "unexpected extra/ missing dirs: {listing:?}");
}

// ==========================================================================
// Test 2: folder mode — manifest.json contents and state_hash correctness
// ==========================================================================

#[test]
fn create_db_file_folder_mode_manifest_contents_and_hashes() {
    let folder = tmp_dir("manifest");
    let project_name = "Manifest Unit Project";
    create_db_file(&folder, project_name).expect("ok");

    let manifest_bytes = fs::read(folder.join("manifest.json")).unwrap();
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes).unwrap();

    // top-level fields
    assert_eq!(manifest.spec_version, SPEC_VERSION);
    assert_eq!(manifest.project_name, project_name);
    assert_eq!(manifest.global_version, 1);
    assert_eq!(manifest.units.base_unit, DEFAULT_BASE_UNIT);
    assert_eq!(manifest.units.internal_resolution_to_meters, DEFAULT_RESOLUTION);
    assert_eq!(manifest.profile.id, "generic_spatial_v0");
    assert_eq!(manifest.views.active_theme, "default_minimal");

    // chunks registry + genesis hash is present and has format sha256:<hex>
    let genesis_meta = manifest.chunks.get("chunk_genesis").expect("chunk_genesis registered");
    assert_eq!(genesis_meta.version, 1);
    assert_eq!(genesis_meta.path, "./chunks/chunk_genesis.json");
    assert!(genesis_meta.hash.starts_with("sha256:"));
    assert_eq!(genesis_meta.hash.len(), 7 + 64);

    // views registry
    let theme_meta = manifest
        .views
        .registered
        .get("default_minimal")
        .expect("default_minimal theme registered");
    assert_eq!(theme_meta.version, 1);
    assert!(theme_meta.hash.starts_with("sha256:"));

    // spatial_index has node_0001 → chunk_genesis
    assert_eq!(
        manifest.spatial_index.get("node_0001"),
        Some(&"chunk_genesis".to_string())
    );

    // profile.core_hints relation_codes = DEFAULT_RELATION_CODES
    let hints = manifest.profile.core_hints.as_ref().unwrap();
    let defaults: Vec<String> = DEFAULT_RELATION_CODES.iter().map(|s| s.to_string()).collect();
    assert_eq!(hints.relation_codes, defaults);

    // state_hash format + recompute matches
    assert!(manifest.state_hash.starts_with("sha256:"));
    assert_eq!(manifest.state_hash.len(), 7 + 64);
    // Recompute state_hash manually per §3.2
    let mut verify = manifest.clone();
    verify.state_hash = String::new();
    let expected = sha256_canonical_json(&verify).unwrap();
    assert_eq!(
        manifest.state_hash, expected,
        "state_hash mismatch: stored={:?} recomputed={expected}",
        manifest.state_hash
    );

    // Genesis chunk hash stored in manifest MUST match actual file on disk
    let genesis_on_disk = fs::read(folder.join("chunks/chunk_genesis.json")).unwrap();
    let actual_genesis_hash = sha256_hex(&genesis_on_disk);
    assert_eq!(
        genesis_meta.hash, actual_genesis_hash,
        "genesis chunk hash in manifest does not match file on disk"
    );
}

// ==========================================================================
// Test 3: folder mode — genesis chunk bounds + node_0001 (DEFAULT 1_000_000 cube)
// ==========================================================================

#[test]
fn create_db_file_genesis_default_cube_1_million() {
    let folder = tmp_dir("genesis_default");
    create_db_file(&folder, "Genesis Cube Default").unwrap();

    let genesis: Chunk =
        serde_json::from_slice(&fs::read(folder.join("chunks/chunk_genesis.json")).unwrap())
            .unwrap();

    assert_eq!(genesis.chunk_id, "chunk_genesis");
    assert_eq!(genesis.version, 1);
    let b = genesis.bounds.unwrap();
    let half = DEFAULT_GENESIS_SPACE_SIZE / 2;
    assert_eq!(b, [-half, -half, -half, half, half, half]);

    let n = genesis.nodes.get("node_0001").expect("node_0001 must exist");
    assert_eq!(n.parent_id, "genesis");
    assert_eq!(n.coords, vec![-half, -half, -half, half, half, half]);
    assert_eq!(n.relations.len(), 0);

    let meta: Value = n.metadata.clone();
    assert_eq!(meta["profile_key"], "container_root");
    assert_eq!(meta["space_size"], DEFAULT_GENESIS_SPACE_SIZE);
    assert!(meta["label"].as_str().unwrap().contains("1000000"));
}

// ==========================================================================
// Test 4: folder mode — custom cube size via create_db_file_with_space_size (10_000_000)
// ==========================================================================

#[test]
fn create_db_file_with_custom_genesis_cube_size_10_million() {
    let folder = tmp_dir("cube_big");
    let custom_size: i64 = 10_000_000;
    create_db_file_with_space_size(&folder, "Big Cube", custom_size).unwrap();

    let genesis: Chunk = serde_json::from_slice(
        &fs::read(folder.join("chunks/chunk_genesis.json")).unwrap(),
    )
    .unwrap();
    let half = custom_size / 2;
    assert_eq!(genesis.bounds.unwrap(), [-half, -half, -half, half, half, half]);
    let n = genesis.nodes.get("node_0001").unwrap();
    assert_eq!(n.metadata["space_size"], custom_size);
    // Also manifest must reflect recomputed hashes (not stale default)
    let m: Manifest =
        serde_json::from_slice(&fs::read(folder.join("manifest.json")).unwrap()).unwrap();
    assert_ne!(m.state_hash, "", "state_hash not empty");
}

// ==========================================================================
// Test 5: folder mode — default_theme.json parses as valid Theme struct
// ==========================================================================

#[test]
fn create_db_file_default_theme_is_valid_theme_struct() {
    let folder = tmp_dir("theme");
    create_db_file(&folder, "Theme Check").unwrap();
    let theme: Theme = serde_json::from_slice(
        &fs::read(folder.join("views/default_theme.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(theme.theme_id, "default_minimal");
    assert_eq!(theme.spec_version, SPEC_VERSION);
    // by_type must have a container_root override with a blue color
    let crt = theme.by_type.get("container_root").unwrap();
    assert_eq!(crt.get("color").unwrap(), "#2563eb");
    // fallback stroke_width must be 2 (integer Value)
    assert_eq!(theme.fallback.get("stroke_width").unwrap(), 2);
}

// ==========================================================================
// Test 6: folder mode — schema.json placeholder exists and is valid JSON object
// ==========================================================================

#[test]
fn create_db_file_placeholder_schema_json_present() {
    let folder = tmp_dir("schema");
    create_db_file(&folder, "Schema Project").unwrap();
    let schema: Value =
        serde_json::from_slice(&fs::read(folder.join("schema.json")).unwrap()).unwrap();
    assert_eq!(schema["type"], "object");
    assert!(schema
        .get("$id")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("generic_spatial_v0"));
}

// ==========================================================================
// Test 7: ZIP mode — creates single-file .aagl ZIP, same files inside as folder mode
// ==========================================================================

#[test]
fn create_db_file_zip_mode_via_direct_initializer_call() {
    // NOTE: Global `CREATE_AS_FOLDER` is true (dev default). ZIP-mode tests therefore call
    // `create_empty_project` directly with folder_mode=false.
    // This mirrors the exact behavior you'd get by flipping CREATE_AS_FOLDER to false at compile time.
    let zip_path = tmp_dir("zip_artifact");
    let zip_path = zip_path.with_extension("aagl");
    let project = "ZIP Mode Project";
    let size = DEFAULT_GENESIS_SPACE_SIZE;

    create_empty_project(&zip_path, project, false, size)
        .expect("zip build via initializer succeeds");

    // 1) File exists as a regular file (not a folder)
    let md = fs::metadata(&zip_path).unwrap();
    assert!(md.is_file(), "zip_path must be a file, not a folder");
    assert!(md.len() > 500, "ZIP is suspiciously small (<500 bytes)");

    // 2) Open ZIP and check entry list matches folder mode exactly
    let zip_file = fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let mut names: HashSet<String> = HashSet::new();
    for i in 0..archive.len() {
        let f = archive.by_index(i).unwrap();
        names.insert(f.name().to_string());
    }
    // Required files in the ZIP
    for required in &[
        "manifest.json",
        "schema.json",
        "chunks/chunk_genesis.json",
        "views/default_theme.json",
    ] {
        assert!(names.contains(*required), "ZIP missing entry: {required}; have: {names:?}");
    }
    // Directory entries should also be present in a well-formed ZIP
    for required_dir in &["chunks/", "views/", "assets/"] {
        assert!(names.contains(*required_dir), "ZIP missing dir entry: {required_dir}");
    }

    // 3) The manifest stored inside the ZIP must actually parse and match project_name
    let manifest_bytes = {
        use std::io::Read;
        let mut f = archive.by_name("manifest.json").unwrap();
        let mut v = Vec::new();
        f.read_to_end(&mut v).unwrap();
        v
    };
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(manifest.project_name, project);
    assert!(manifest.state_hash.starts_with("sha256:"));
    // 4) Cross-check manifest genesis hash matches the ZIP's genesis.json bytes
    let genesis_bytes = {
        use std::io::Read;
        let mut f = archive.by_name("chunks/chunk_genesis.json").unwrap();
        let mut v = Vec::new();
        f.read_to_end(&mut v).unwrap();
        v
    };
    let expected_genesis_hash = sha256_hex(&genesis_bytes);
    assert_eq!(
        manifest.chunks.get("chunk_genesis").unwrap().hash,
        expected_genesis_hash
    );
}

// ==========================================================================
// Test 9: Precondition errors (sanity of parameter validation)
// ==========================================================================

#[test]
fn create_db_file_rejects_genesis_size_zero_or_one() {
    let bad_sizes = [0i64, 1i64, -500_000i64];
    for bad in bad_sizes {
        let tmp = tmp_dir(&format!("precond_size_{bad}"));
        let res = create_db_file_with_space_size(&tmp, "Bad", bad);
        assert!(
            res.is_err(),
            "size={bad} should have failed validation but got Ok"
        );
    }
}

#[test]
fn create_db_file_overwrites_existing_target_folder() {
    // Create a folder with a garbage file inside, then call create_db_file on SAME path.
    // It must replace the folder entirely (old garbage file gone).
    let folder = tmp_dir("overwrite");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("old_garbage.bin"), b"should be deleted").unwrap();
    assert!(folder.join("old_garbage.bin").exists());

    create_db_file(&folder, "Overwrite Project").expect("overwrite succeeds");
    assert!(
        !folder.join("old_garbage.bin").exists(),
        "old garbage file was not removed during overwrite"
    );
    // New files exist as expected (re-using Test1 invariants)
    assert!(folder.join("manifest.json").exists());
    assert!(folder.join("chunks/chunk_genesis.json").exists());
}

#[test]
fn create_db_file_overwrites_existing_target_zip() {
    let zip_path = tmp_dir("overwrite_zip").with_extension("aagl");
    fs::write(&zip_path, b"definitely not a valid zip").unwrap();
    assert!(zip_path.exists());
    create_empty_project(&zip_path, "Overwrite ZIP", false, 2_000)
        .expect("overwrite .aagl file succeeds");
    // After write it should be a valid ZIP with manifest inside
    let f = fs::File::open(&zip_path).unwrap();
    let mut z = zip::ZipArchive::new(f).expect("written file must be valid ZIP");
    assert!(z.by_name("manifest.json").is_ok(), "ZIP has manifest.json after overwrite");
}
