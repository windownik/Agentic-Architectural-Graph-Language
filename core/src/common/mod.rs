//! Common — no internal dependencies (root of Clean Architecture dependency graph).
//! Exports:
//!   • `constants` — every global const/toggle
//!   • `error`     — `AaglError` enum + `Result<T>` alias

pub mod constants;
pub mod error;

// ─── Flat re-exports so callers can `use crate::common::{*, SPEC_VERSION, AaglError}`
pub use constants::*;
pub use error::{AaglError, Result};
