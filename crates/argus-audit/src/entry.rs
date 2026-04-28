//! Audit entry — the atomic unit of the cryptographic chain.
//!
//! Each entry hashes all its own fields plus the previous entry's hash,
//! creating a tamper-evident Merkle chain. Any modification to any
//! historical entry invalidates all subsequent entry_hashes.

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// A single link in the audit chain.
/// entry_hash must be computed via compute_entry_hash() after all other fields are set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp_us: i64,
    pub agent_model: String,
    pub action_type: String,        // "tool_call" | "model_call" | "memory_write" | "discourse_post" | "system"
    pub tool_name: Option<String>,
    pub args_hash: String,          // SHA-256 hex of serialized args — never the args themselves
    pub result_hash: String,        // SHA-256 hex of result — never the result itself
    pub session_id: String,         // UUID for current daemon session
    pub prev_entry_hash: String,    // SHA-256 of previous entry_hash, or SHA-256("GENESIS")
    pub entry_hash: String,         // SHA-256 of all other fields concatenated — computed last
}

/// Hash a string with SHA-256 and return lowercase hex.
pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// The prev_entry_hash for the very first entry in a fresh chain.
pub fn genesis_prev_hash() -> String {
    sha256_hex("GENESIS")
}

impl AuditEntry {
    /// Compute and set entry_hash from all other fields.
    /// Must be called once all other fields are populated.
    pub fn compute_entry_hash(&mut self) {
        let canonical = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.id,
            self.timestamp_us,
            self.agent_model,
            self.action_type,
            self.tool_name.as_deref().unwrap_or(""),
            self.args_hash,
            self.result_hash,
            self.session_id,
            self.prev_entry_hash,
        );
        self.entry_hash = sha256_hex(&canonical);
    }

    /// Verify that this entry's entry_hash is consistent with its fields.
    /// Returns false if any field has been tampered with.
    pub fn verify(&self) -> bool {
        let mut clone = self.clone();
        let claimed = clone.entry_hash.clone();
        clone.compute_entry_hash();
        clone.entry_hash == claimed
    }
}
