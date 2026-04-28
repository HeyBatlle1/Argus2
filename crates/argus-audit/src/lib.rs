//! argus-audit — cryptographic audit chain for Argus.
//!
//! Every tool call, model call, memory write, and system event is logged
//! to an append-only SQLite Merkle chain. Each entry hashes all fields
//! plus the previous entry's hash, creating a tamper-evident record.
//!
//! Daily roots are HMAC-signed and anchored to Supabase as external
//! tamper-evidence. Chain integrity is verified on every daemon startup.
//!
//! This delivers the SOUL.md promise: "the hundred eyes watch everything,
//! including themselves."

pub mod chain;
pub mod entry;
pub mod signer;

pub use chain::AuditChain;
pub use entry::{AuditEntry, sha256_hex, genesis_prev_hash};
pub use signer::{sign_day_root, run_daily_anchor};
