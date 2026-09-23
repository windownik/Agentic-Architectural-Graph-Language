//! Application layer (Clean Architecture: "Use Cases" / "Interactors").
//!
//! Dependency rule (MANDATORY):
//!   application → common (Result/Error + constants)
//!   application → domain         (all data models)
//!   application → infrastructure (hash + storage backends for IO)
//!
//! Application MUST NOT depend on lib.rs or any FFI code.
//! FFI + command thin wrappers in lib.rs CALL INTO this layer.

pub mod initializer;
pub mod commands;
pub mod node_manager;

// ─── Flat re-exports of the most-used entries
pub use initializer::create_empty_project;
pub use commands::*;
pub use node_manager::{add_object, delete_object, update_object};
