//! Cryptographic hash utilities used by the storage layer and state-hash
//! computation (§3.2 of the spec). All hashes are produced in the canonical
//! format `sha256:<64 lowercase hex chars>` so they're identifiable at a
//! glance from file paths and other opaque strings.
//!
//! Two helpers here cover every use case:
//!   * `sha256_hex(bytes)`           — raw byte hash (use for already-serialized JSON or assets).
//!   * `sha256_canonical_json(val)`  — serialize any `T: Serialize` to COMPACT JSON (no whitespace,
//!                                     serde_json default stable key order), then hash that.

use crate::common::Result;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Raw-SHA-256 with the `sha256:` prefix (§3.2 container format).
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("sha256:{}", hex::encode(h.finalize()))
}

/// Hash the canonical JSON representation of a value. `serde_json::to_vec`
/// produces compact JSON (no whitespace) which gives byte-stable output for
/// the same value tree — exactly what we need for `state_hash` §3.2.
pub fn sha256_canonical_json<T: Serialize>(value: &T) -> Result<String> {
    let bytes = serde_json::to_vec(value)?;
    Ok(sha256_hex(&bytes))
}
