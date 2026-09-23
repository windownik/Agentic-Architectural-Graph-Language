//! Demo: create DB → add 2 objects (wall + window) → update wall coords → delete window.
//! Run with:  `cargo run` (from `core/` directory). Everything uses folder mode default.

use aagl_core::{create_db_file, add_object, update_object, delete_object};
use serde_json::json;
use std::path::Path;

fn main() -> aagl_core::Result<()> {

    // 1) Prepare demo output dir
    let out_dir = Path::new("./demo_crud_objects");
    if out_dir.exists() {
        if out_dir.is_dir() { std::fs::remove_dir_all(out_dir)?; } else { std::fs::remove_file(out_dir)?; }
    }
    create_db_file(out_dir, "CAD Demo — Walls + Window")?;
    println!("✅  DB created at: {}", out_dir.canonicalize()?.display());

    // 2) ADD wall (parent = root node_0001, bbox6 mm coords, metadata)
    let wall_id = add_object(
        out_dir,
        "node_0001",                              // parent: root genesis
        vec![0, 0, 0, 4500, 380, 2700],          // xmin,ymin,zmin,xmax,ymax,zmax (mm)
        json!({"profile_key":"wall","material_code":"brick","thickness_mm":380,"height_mm":2700}),
        vec![],                                    // no relations yet
        "wall_",                                   // id prefix → wall_0001
    )?;
    println!("➕  Created wall:    id = {wall_id}");

    // 3) ADD window INSIDE that wall (parent = wall_id)
    let window_id = add_object(
        out_dir,
        wall_id.clone(),
        vec![1200, 0, 900,  1200+1500, 380, 900+1500],    // 1.5m x 1.5m window, sill 900mm high
        json!({"profile_key":"window","parent_wall_id":wall_id,"width_mm":1500,"height_mm":1500,"sill_height_mm":900}),
        vec![],
        "window_",
    )?;
    println!("➕  Created window:  id = {window_id}   (parent = {wall_id})");

    // 4) UPDATE wall — stretch it horizontally 4.5m → 6m (right edge moves)
    update_object(
        out_dir,
        &wall_id,
        None,                                    // parent: no change
        Some(vec![0, 0, 0, 6000, 380, 2700]),   // new bbox: length 6m instead of 4.5m
        None,                                    // relations: no change
        None,                                    // metadata: keep as-is
    )?;
    println!("🔄  Updated {wall_id}: xmax 4500mm → 6000mm (stretched 1.5m right)");

    // 5) DELETE window
    delete_object(out_dir, &window_id)?;
    println!("➖  Deleted window:  {window_id}");

    println!("\n✅  Demo complete. Check files in:  {}", out_dir.canonicalize()?.display());
    println!("   → manifest.json has global_version = 4  (create=1 + 3 mutations)");
    println!("   → chunks/chunk_genesis.json  has chunk.version = 4  + 1 node: {wall_id}");

    Ok(())
}