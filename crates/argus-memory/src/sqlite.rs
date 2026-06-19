//! Native Rust memory backend using SQLite
//!
//! Replaces the Python subprocess bridge. No more shelling out to Python
//! for every memory operation. Direct rusqlite with proper error handling.

use argus_core::agent::ConversationMessage;
use argus_core::tools::{MemoryBackend, MemoryRecord};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

/// Metadata about a persisted conversation (web, telegram, discord).
#[derive(Debug, Clone)]
pub struct ConversationMeta {
    pub id: String,
    pub title: String,
    pub surface: String,
    pub model: Option<String>,
    pub message_count: i64,
    pub started_at: String,
    pub last_active_at: String,
}

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

        // Create tables if not exist
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
            CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance DESC);
            CREATE TABLE IF NOT EXISTS conversation_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                model TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_history_chat ON conversation_history(chat_id, id);",
        )
        .map_err(|e| format!("Failed to create tables: {}", e))?;

        // Migrate existing DBs that predate the model column (error ignored — column may already exist).
        let _ = conn.execute_batch(
            "ALTER TABLE conversation_history ADD COLUMN model TEXT;"
        );

        // Conversation metadata — one row per named conversation (web/discord).
        // Telegram uses integer chat_id in conversation_history; web uses these tables.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL DEFAULT 'New Conversation',
                surface TEXT NOT NULL DEFAULT 'web',
                model TEXT,
                message_count INTEGER NOT NULL DEFAULT 0,
                started_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_active_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS web_conversation_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                model TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_web_conv ON web_conversation_history(conversation_id, id);",
        )
        .map_err(|e| format!("Failed to create conversation tables: {}", e))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open using the default path.
    /// Respects ARGUS_DATA_DIR env var (persistent volume in Docker).
    /// Falls back to ~/.argus/memory.db for local dev.
    pub fn open_default() -> Result<Self, String> {
        let path = if let Ok(data_dir) = std::env::var("ARGUS_DATA_DIR") {
            std::path::PathBuf::from(data_dir).join("memory.db")
        } else {
            dirs::home_dir()
                .ok_or_else(|| "No home directory".to_string())?
                .join(".argus")
                .join("memory.db")
        };
        Self::open(path)
    }

    /// Persist conversation history for a chat. Replaces existing history for that chat_id.
    /// Keeps at most 40 most recent turns.
    pub fn save_history(&self, chat_id: i64, messages: &[ConversationMessage]) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM conversation_history WHERE chat_id = ?1", params![chat_id])
            .map_err(|e| format!("Failed to clear history: {}", e))?;

        let start = messages.len().saturating_sub(40);
        for msg in &messages[start..] {
            conn.execute(
                "INSERT INTO conversation_history (chat_id, role, content, model) VALUES (?1, ?2, ?3, ?4)",
                params![chat_id, msg.role, msg.content, msg.model],
            )
            .map_err(|e| format!("Failed to save history turn: {}", e))?;
        }
        Ok(())
    }

    /// Load persisted conversation history for a chat.
    pub fn load_history(&self, chat_id: i64) -> Result<Vec<ConversationMessage>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT role, content, model FROM conversation_history WHERE chat_id = ?1 ORDER BY id ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![chat_id], |row| {
                Ok(ConversationMessage {
                    role:    row.get(0)?,
                    content: row.get(1)?,
                    model:   row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut history = Vec::new();
        for row in rows {
            if let Ok(msg) = row {
                history.push(msg);
            }
        }
        Ok(history)
    }

    /// Persist web conversation history keyed by string conversation ID.
    /// Replaces all existing messages for that conversation. Keeps last 40.
    pub fn save_history_str(&self, conversation_id: &str, messages: &[ConversationMessage]) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM web_conversation_history WHERE conversation_id = ?1",
            params![conversation_id],
        ).map_err(|e| format!("Failed to clear web history: {}", e))?;

        let start = messages.len().saturating_sub(40);
        for msg in &messages[start..] {
            conn.execute(
                "INSERT INTO web_conversation_history (conversation_id, role, content, model) VALUES (?1, ?2, ?3, ?4)",
                params![conversation_id, msg.role, msg.content, msg.model],
            ).map_err(|e| format!("Failed to save web history turn: {}", e))?;
        }
        Ok(())
    }

    /// Load web conversation history by string conversation ID.
    pub fn load_history_str(&self, conversation_id: &str) -> Result<Vec<ConversationMessage>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT role, content, model FROM web_conversation_history WHERE conversation_id = ?1 ORDER BY id ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![conversation_id], |row| {
                Ok(ConversationMessage {
                    role:    row.get(0)?,
                    content: row.get(1)?,
                    model:   row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut history = Vec::new();
        for row in rows {
            if let Ok(msg) = row { history.push(msg); }
        }
        Ok(history)
    }

    /// Insert or update conversation metadata. Title is only set on creation;
    /// subsequent calls update model, message_count, and last_active_at.
    pub fn upsert_conversation(
        &self,
        id: &str,
        title: &str,
        surface: &str,
        model: Option<&str>,
        message_count: usize,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO conversations (id, title, surface, model, message_count)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               model         = excluded.model,
               message_count = excluded.message_count,
               last_active_at = datetime('now')",
            params![id, title, surface, model, message_count as i64],
        ).map_err(|e| format!("Failed to upsert conversation: {}", e))?;
        Ok(())
    }

    /// Return metadata for the most recently active conversation.
    pub fn latest_conversation(&self) -> Result<Option<ConversationMeta>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, title, surface, model, message_count, started_at, last_active_at
                 FROM conversations ORDER BY last_active_at DESC LIMIT 1",
            )
            .map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map([], |row| {
                Ok(ConversationMeta {
                    id:             row.get(0)?,
                    title:          row.get(1)?,
                    surface:        row.get(2)?,
                    model:          row.get(3)?,
                    message_count:  row.get(4)?,
                    started_at:     row.get(5)?,
                    last_active_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        Ok(rows.next().and_then(|r| r.ok()))
    }

    /// List recent conversations ordered by last active, newest first.
    pub fn list_conversations(&self, limit: usize) -> Result<Vec<ConversationMeta>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, title, surface, model, message_count, started_at, last_active_at
                 FROM conversations ORDER BY last_active_at DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(ConversationMeta {
                    id:             row.get(0)?,
                    title:          row.get(1)?,
                    surface:        row.get(2)?,
                    model:          row.get(3)?,
                    message_count:  row.get(4)?,
                    started_at:     row.get(5)?,
                    last_active_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        for row in rows {
            if let Ok(meta) = row { result.push(meta); }
        }
        Ok(result)
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
                "SELECT id, memory_type, content, importance, created_at FROM memories WHERE content LIKE ?1 AND memory_type = ?2 ORDER BY importance DESC LIMIT ?3".to_string(),
                vec![
                    Box::new(format!("%{}%", q)) as Box<dyn rusqlite::types::ToSql>,
                    Box::new(t.to_string()),
                    Box::new(limit as i64),
                ],
            ),
            (Some(q), None) => (
                "SELECT id, memory_type, content, importance, created_at FROM memories WHERE content LIKE ?1 ORDER BY importance DESC LIMIT ?2".to_string(),
                vec![
                    Box::new(format!("%{}%", q)) as Box<dyn rusqlite::types::ToSql>,
                    Box::new(limit as i64),
                ],
            ),
            (None, Some(t)) => (
                "SELECT id, memory_type, content, importance, created_at FROM memories WHERE memory_type = ?1 ORDER BY importance DESC LIMIT ?2".to_string(),
                vec![
                    Box::new(t.to_string()) as Box<dyn rusqlite::types::ToSql>,
                    Box::new(limit as i64),
                ],
            ),
            (None, None) => (
                "SELECT id, memory_type, content, importance, created_at FROM memories ORDER BY importance DESC LIMIT ?1".to_string(),
                vec![Box::new(limit as i64) as Box<dyn rusqlite::types::ToSql>],
            ),
        };

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(MemoryRecord {
                    id: row.get(0)?,
                    memory_type: row.get(1)?,
                    content: row.get(2)?,
                    importance: row.get(3)?,
                    created_at: row.get(4)?,
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
