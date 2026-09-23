//! Domain layer — pure enterprise data models (NO logic, NO IO, NO serde external config beyond derives).
//!
//! Dependency rule (Clean Architecture):
//!   domain → common (only constants for defaults if necessary; types from common::error are NOT used here).
//!   Application layer uses domain types to run business logic.
//!   Infrastructure layer uses domain types to (de)serialize from disk/ZIP.

pub mod manifest;
pub mod graph;
pub mod view;

// ─── Flat re-exports (consumers write `use aagl_core::domain::{Manifest, Node, Theme}`).
pub use manifest::{ChunkMetadata, CoreHints, Manifest, Profile, Units, ViewsRegistry};
pub use graph::{Chunk, Node, Relation};
pub use view::{PatchContext, Theme};
