//! AuditChain — append-only SQLite storage for the cryptographic audit log.
//!
//! Rules:
//!   - Append only. No UPDATE, no DELETE, ever.
//!   - WAL mode for concurrent reads.
//!   - Each append atomically advances last_hash and last_id.
//!   - verify_recent() checks both internal hash integrity and chain links.

use rusqlite::{Connection, params};
use std::sync::Mutex;
use std::path::Path;
use chrono::Utc;
use uuid::Uuid;
use crate::entry::{AuditEntry, sha256_hex, genesis_prev_hash};

pub struct AuditChain {
    conn: Mutex<Connection>,
    pub session_id: String,
    last_hash: Mutex<String>,
    last_id: Mutex<u64>,
}

impl AuditChain {
    /// Open or create the audit database at the given path.
    /// Typically /argus/data/audit.db inside the daemon container.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open(path)
            .map_err(|e| format!("Failed to open audit DB: {}", e))?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             CREATE TABLE IF NOT EXISTS audit_entries (
                 id              INTEGER PRIMARY KEY,
                 timestamp_us    INTEGER NOT NULL,
                 agent_model     TEXT NOT NULL,
                 action_type     TEXT NOT NULL,
                 tool_name       TEXT,
                 args_hash       TEXT NOT NULL,
                 result_hash     TEXT NOT NULL,
                 session_id      TEXT NOT NULL,
                 prev_entry_hash TEXT NOT NULL,
                 entry_hash      TEXT NOT NULL UNIQUE
             );
             CREATE INDEX IF NOT EXISTS idx_session   ON audit_entries(session_id);
             CREATE INDEX IF NOT EXISTS idx_timestamp ON audit_entries(timestamp_us);",
        ).map_err(|e| format!("Failed to initialise audit schema: {}", e))?;

        // Resume the chain from the last persisted entry
        let (last_id, last_hash) = {
            let mut stmt = conn.prepare(
                "SELECT id, entry_hash FROM audit_entries ORDER BY id DESC LIMIT 1"
            ).map_err(|e| e.to_string())?;

            let result: Result<(u64, String), _> = stmt.query_row([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            });

            match result {
                Ok((id, hash)) => (id, hash),
                Err(_) => (0, genesis_prev_hash()),
            }
        };

        Ok(Self {
            conn: Mutex::new(conn),
            session_id: Uuid::new_v4().to_string(),
            last_hash: Mutex::new(last_hash),
            last_id: Mutex::new(last_id),
        })
    }

    /// Append a new entry to the chain. The only write path — no updates, no deletes.
    ///
    /// `args`   — serialized args string (caller hashes it here; pass None for model calls)
    /// `result` — result string (caller hashes it here; pass None for system events)
    ///
    /// Returns the new entry id on success.
    pub fn append(
        &self,
        agent_model: &str,
        action_type: &str,
        tool_name: Option<&str>,
        args: Option<&str>,
        result: Option<&str>,
    ) -> Result<u64, String> {
        let mut last_hash = self.last_hash.lock().map_err(|e| e.to_string())?;
        let mut last_id   = self.last_id.lock().map_err(|e| e.to_string())?;

        let new_id   = *last_id + 1;
        let now_us   = Utc::now().timestamp_micros();
        let args_hash   = sha256_hex(args.unwrap_or(""));
        let result_hash = sha256_hex(result.unwrap_or(""));

        let mut entry = AuditEntry {
            id: new_id,
            timestamp_us: now_us,
            agent_model: agent_model.to_string(),
            action_type: action_type.to_string(),
            tool_name: tool_name.map(String::from),
            args_hash,
            result_hash,
            session_id: self.session_id.clone(),
            prev_entry_hash: last_hash.clone(),
            entry_hash: String::new(),
        };
        entry.compute_entry_hash();

        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO audit_entries
             (id, timestamp_us, agent_model, action_type, tool_name,
              args_hash, result_hash, session_id, prev_entry_hash, entry_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                entry.id,
                entry.timestamp_us,
                entry.agent_model,
                entry.action_type,
                entry.tool_name,
                entry.args_hash,
                entry.result_hash,
                entry.session_id,
                entry.prev_entry_hash,
                entry.entry_hash,
            ],
        ).map_err(|e| format!("Audit insert failed: {}", e))?;

        *last_hash = entry.entry_hash;
        *last_id   = new_id;

        Ok(new_id)
    }

    /// Verify the last `n` entries form a valid chain.
    /// Checks both internal hash integrity and chain links.
    /// Returns Ok(count_verified) or Err(description of first failure).
    pub fn verify_recent(&self, n: usize) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp_us, agent_model, action_type, tool_name,
                    args_hash, result_hash, session_id, prev_entry_hash, entry_hash
             FROM audit_entries ORDER BY id DESC LIMIT ?1"
        ).map_err(|e| e.to_string())?;

        let mut entries: Vec<AuditEntry> = stmt.query_map(params![n as i64], |row| {
            Ok(AuditEntry {
                id:              row.get(0)?,
                timestamp_us:    row.get(1)?,
                agent_model:     row.get(2)?,
                action_type:     row.get(3)?,
                tool_name:       row.get(4)?,
                args_hash:       row.get(5)?,
                result_hash:     row.get(6)?,
                session_id:      row.get(7)?,
                prev_entry_hash: row.get(8)?,
                entry_hash:      row.get(9)?,
            })
        }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        // Query returns newest-first; reverse to oldest-first for chain walk
        entries.reverse();

        let mut verified = 0;
        for i in 0..entries.len() {
            if !entries[i].verify() {
                return Err(format!(
                    "Entry {} has corrupted entry_hash — audit chain tampered",
                    entries[i].id
                ));
            }
            if i > 0 && entries[i].prev_entry_hash != entries[i - 1].entry_hash {
                return Err(format!(
                    "Chain break between entries {} and {} — audit chain tampered",
                    entries[i - 1].id, entries[i].id
                ));
            }
            verified += 1;
        }

        Ok(verified)
    }

    /// Compute a day root: SHA-256 of all entry_hashes for `date` concatenated in id order.
    /// Used for daily HMAC signing and Supabase anchoring.
    pub fn compute_day_root(&self, date: &str) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let day_start_us = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|e| e.to_string())?
            .and_hms_opt(0, 0, 0)
            .ok_or("Invalid midnight")?
            .and_utc()
            .timestamp_micros();
        let day_end_us = day_start_us + 86_400_000_000i64;

        let mut stmt = conn.prepare(
            "SELECT entry_hash FROM audit_entries
             WHERE timestamp_us >= ?1 AND timestamp_us < ?2
             ORDER BY id ASC"
        ).map_err(|e| e.to_string())?;

        let hashes: Vec<String> = stmt.query_map(params![day_start_us, day_end_us], |row| {
            row.get(0)
        }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        if hashes.is_empty() {
            return Ok(sha256_hex(&format!("EMPTY_DAY_{}", date)));
        }

        Ok(sha256_hex(&hashes.join("")))
    }

    /// Count audit entries logged today (UTC).
    pub fn entry_count_today(&self) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let today_start_us = Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .ok_or("Invalid midnight")?
            .and_utc()
            .timestamp_micros();

        conn.query_row(
            "SELECT COUNT(*) FROM audit_entries WHERE timestamp_us >= ?1",
            params![today_start_us],
            |row| row.get(0),
        ).map_err(|e| e.to_string())
    }
}
