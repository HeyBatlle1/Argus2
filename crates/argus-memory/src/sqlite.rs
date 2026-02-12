//! Native Rust memory backend using SQLite
//!
//! Replaces the Python subprocess bridge. No more shelling out to Python
//! for every memory operation. Direct rusqlite with proper error handling.

use argus_core::tools::{MemoryBackend, MemoryRecord};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

/// SQLite-backed memory store
pub struct SqliteMemory {
    conn: Mutex<Connection>,
}

impl SqliteMemory {
    /// Open or create the memory database
    pub fn open(path: PathBuf) -> Result<Self, String> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create memory dir: {}", e))?;
        }

        let conn = Connection::open(&path)
            .map_err(|e| format!("Failed to open memory database: {}", e))?;

        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("Failed to set pragmas: {}", e))?;

        // Create table if not exists
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                memory_type TEXT NOT NULL DEFAULT 'fact',
                content TEXT NOT NULL,
                reasoning TEXT,
                importance REAL NOT NULL DEFAULT 5.0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_memories_type ON memories(memory_type);
            CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance DESC);",
        )
        .map_err(|e| format!("Failed to create memories table: {}", e))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open using the default path (~/.argus/memory.db)
    pub fn open_default() -> Result<Self, String> {
        let path = dirs::home_dir()
            .ok_or_else(|| "No home directory".to_string())?
            .join(".argus")
            .join("memory.db");
        Self::open(path)
    }
}

impl MemoryBackend for SqliteMemory {
    fn remember(
        &self,
        memory_type: &str,
        content: &str,
        reasoning: Option<&str>,
        importance: f64,
    ) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // Check for duplicate content
        let existing: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM memories WHERE content = ?1",
                params![content],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if existing {
            // Update importance if higher
            conn.execute(
                "UPDATE memories SET importance = MAX(importance, ?1), updated_at = datetime('now') WHERE content = ?2",
                params![importance, content],
            )
            .map_err(|e| e.to_string())?;
            return Ok("✅ Memory updated (already existed)".to_string());
        }

        conn.execute(
            "INSERT INTO memories (memory_type, content, reasoning, importance) VALUES (?1, ?2, ?3, ?4)",
            params![memory_type, content, reasoning, importance],
        )
        .map_err(|e| format!("Failed to store memory: {}", e))?;

        Ok(format!("✅ Remembered [{}]: {}", memory_type, content))
    }

    fn recall(
        &self,
        query: Option<&str>,
        memory_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let (sql, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match (query, memory_type) {
            (Some(q), Some(t)) => (
                "SELECT memory_type, content, importance, created_at FROM memories WHERE content LIKE ?1 AND memory_type = ?2 ORDER BY importance DESC LIMIT ?3".to_string(),
                vec![
                    Box::new(format!("%{}%", q)) as Box<dyn rusqlite::types::ToSql>,
                    Box::new(t.to_string()),
                    Box::new(limit as i64),
                ],
            ),
            (Some(q), None) => (
                "SELECT memory_type, content, importance, created_at FROM memories WHERE content LIKE ?1 ORDER BY importance DESC LIMIT ?2".to_string(),
                vec![
                    Box::new(format!("%{}%", q)) as Box<dyn rusqlite::types::ToSql>,
                    Box::new(limit as i64),
                ],
            ),
            (None, Some(t)) => (
                "SELECT memory_type, content, importance, created_at FROM memories WHERE memory_type = ?1 ORDER BY importance DESC LIMIT ?2".to_string(),
                vec![
                    Box::new(t.to_string()) as Box<dyn rusqlite::types::ToSql>,
                    Box::new(limit as i64),
                ],
            ),
            (None, None) => (
                "SELECT memory_type, content, importance, created_at FROM memories ORDER BY importance DESC LIMIT ?1".to_string(),
                vec![Box::new(limit as i64) as Box<dyn rusqlite::types::ToSql>],
            ),
        };

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(MemoryRecord {
                    memory_type: row.get(0)?,
                    content: row.get(1)?,
                    importance: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for row in rows {
            if let Ok(record) = row {
                results.push(record);
            }
        }
        Ok(results)
    }

    fn forget(&self, content_match: &str) -> Result<String, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let deleted = conn
            .execute(
                "DELETE FROM memories WHERE content LIKE ?1",
                params![format!("%{}%", content_match)],
            )
            .map_err(|e| format!("Failed to forget: {}", e))?;

        Ok(format!("✅ Forgot {} memories", deleted))
    }
}

/// List all memories (for CLI `argus memory list`)
pub fn list_all_memories(memory: &SqliteMemory) -> Result<Vec<MemoryRecord>, String> {
    memory.recall(None, None, 100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_remember_and_recall() {
        let tmp = NamedTempFile::new().unwrap();
        let mem = SqliteMemory::open(tmp.path().to_path_buf()).unwrap();

        mem.remember("fact", "User likes Rust", None, 8.0).unwrap();
        mem.remember("preference", "Dark mode preferred", Some("They said so"), 6.0).unwrap();

        let all = mem.recall(None, None, 10).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].importance, 8.0); // Higher importance first

        let facts = mem.recall(None, Some("fact"), 10).unwrap();
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].content, "User likes Rust");

        let search = mem.recall(Some("Rust"), None, 10).unwrap();
        assert_eq!(search.len(), 1);
    }

    #[test]
    fn test_forget() {
        let tmp = NamedTempFile::new().unwrap();
        let mem = SqliteMemory::open(tmp.path().to_path_buf()).unwrap();

        mem.remember("fact", "temporary info", None, 3.0).unwrap();
        mem.remember("fact", "keep this", None, 5.0).unwrap();

        let result = mem.forget("temporary").unwrap();
        assert!(result.contains("1 memories"));

        let remaining = mem.recall(None, None, 10).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].content, "keep this");
    }

    #[test]
    fn test_duplicate_detection() {
        let tmp = NamedTempFile::new().unwrap();
        let mem = SqliteMemory::open(tmp.path().to_path_buf()).unwrap();

        mem.remember("fact", "same content", None, 3.0).unwrap();
        mem.remember("fact", "same content", None, 8.0).unwrap(); // higher importance

        let all = mem.recall(None, None, 10).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].importance, 8.0); // Should be updated to higher
    }
}
