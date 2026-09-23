//! Demo: create a sample AAGL database (folder mode) and print its file tree +
//! the contents of manifest.json, chunk_genesis.json, default_theme.json.

use aagl_core::*;
use serde_json::Value;
use std::fs;
use std::path::Path;


fn main() -> Result<()> {
    // 1) Decide where output will live (remove stale demo if any).
    let out_dir = Path::new("./demo_output_myproject");
    if out_dir.exists() {
        if out_dir.is_dir() { fs::remove_dir_all(out_dir)?; } else { fs::remove_file(out_dir)?; }
    }

    // 2) CREATE THE DATABASE (folder mode because CREATE_AS_FOLDER=true).
    //    This single line is the exact Anchor-style API you asked for.
    create_db_file(out_dir, "My First AAGL Project")?;


    Ok(())
}