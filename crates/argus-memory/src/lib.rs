//! Argus Memory - Native Rust SQLite storage
//!
//! No more Python subprocess bridge. Direct SQLite with rusqlite.

pub mod sqlite;

pub use sqlite::SqliteMemory;
