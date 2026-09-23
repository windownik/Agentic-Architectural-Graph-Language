//! Global compile-time constants for the AAGL project (spec version,
//! default values, global mode toggles). Nothing in this file depends on any
//! anything other module (core::common::constants is the root import target
//! of the dependency graph.

// ─── Container format
pub const SPEC_VERSION: &str = "0.1.0";
pub const GENESIS_CHUNK_ID: &str = "chunk_genesis";
pub const GENESIS_ROOT_PARENT: &str = "genesis";

// ─── Default genesis cube edge length (integer internal units).
// Resulting bounds: [ -DEFAULT_GENESIS_SPACE_SIZE / 2 , … , + … / 2 ]
pub const DEFAULT_GENESIS_SPACE_SIZE: i64 = 1_000_000;

// ─── Profile + units defaults
pub const DEFAULT_BASE_UNIT: &str = "mm";
pub const DEFAULT_RESOLUTION: f64 = 0.001;
pub const DEFAULT_PROFILE_ID: &str = "generic_spatial_v0";
pub const DEFAULT_VIEW_TYPE_KEY: &str = "metadata.profile_key";
pub const DEFAULT_RELATION_CODES: [&str; 4] =
    ["directed", "symmetric", "control", "dependent"];

// ─── Default theme
pub const DEFAULT_THEME_ID: &str = "default_minimal";

// ─── Global behaviour toggles
//
// If true  → `create_db_file` builds a plain folder (development/giten
// If false → `create_db_file` builds a single-file `.aagl` ZIP document
pub const CREATE_AS_FOLDER: bool = true;
