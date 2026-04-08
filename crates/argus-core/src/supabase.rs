//! Supabase REST API client for Argus
//!
//! Thin reqwest wrapper around Supabase's PostgREST endpoint.
//! No heavy SDK needed — it's just HTTP with an Authorization header.
//!
//! Tables served:
//!   Reads:  argus_checkin_config, argus_schedule, argus_memories
//!   Writes: argus_checkin_log, argus_agent_discourse, argus_conversations, argus_memories

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Config shape (matches argus_checkin_config table) ──────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct CheckinConfig {
    pub interval_minutes: i64,
    pub checkin_type: String,
    pub quiet_hours_start: Option<String>, // "23:00"
    pub quiet_hours_end: Option<String>,   // "07:00"
    pub telegram_enabled: bool,
}

impl Default for CheckinConfig {
    fn default() -> Self {
        Self {
            interval_minutes: 60,
            checkin_type: "brief".to_string(),
            quiet_hours_start: Some("23:00".to_string()),
            quiet_hours_end: Some("07:00".to_string()),
            telegram_enabled: true,
        }
    }
}

// ── Discourse post shape ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct DiscoursePost {
    pub author: String,
    pub post_type: String,    // "finding" | "question" | "proposal"
    pub content: String,
    pub task_context: Option<String>,
    pub requires_human_review: bool,
}

// ── Checkin log entry ──────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CheckinLogEntry {
    pub checkin_type: String,
    pub status: String,
    pub message_sent: String,
    pub system_health: Option<Value>,
}

// ── Client ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct SupabaseClient {
    base_url: String,
    jwt: String,
    client: Client,
}

impl SupabaseClient {
    pub fn new(base_url: impl Into<String>, jwt: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            jwt: jwt.into(),
            client: Client::new(),
        }
    }

    fn rest_url(&self, table: &str) -> String {
        format!("{}/rest/v1/{}", self.base_url, table)
    }

    /// GET /rest/v1/{table}?{query_params}
    /// Example: select("argus_checkin_config", "select=*&limit=1")
    pub async fn select(&self, table: &str, query: &str) -> Result<Value, String> {
        let url = if query.is_empty() {
            format!("{}?select=*", self.rest_url(table))
        } else {
            format!("{}?{}", self.rest_url(table), query)
        };

        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.jwt))
            .header("apikey", &self.jwt)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| format!("Supabase GET failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Supabase {} error {}: {}", table, status, body));
        }

        resp.json::<Value>()
            .await
            .map_err(|e| format!("Supabase response parse error: {}", e))
    }

    /// POST /rest/v1/{table} — insert a row
    pub async fn insert(&self, table: &str, data: &Value) -> Result<(), String> {
        let resp = self.client
            .post(&self.rest_url(table))
            .header("Authorization", format!("Bearer {}", self.jwt))
            .header("apikey", &self.jwt)
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(data)
            .send()
            .await
            .map_err(|e| format!("Supabase POST failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Supabase insert into {} error {}: {}", table, status, body));
        }

        Ok(())
    }

    // ── Domain helpers ─────────────────────────────────────────────────────

    /// Read the first row of argus_checkin_config.
    /// Falls back to defaults if table is empty or request fails.
    pub async fn read_checkin_config(&self) -> CheckinConfig {
        match self.select("argus_checkin_config", "select=*&limit=1").await {
            Err(e) => {
                eprintln!("[supabase] Failed to read checkin_config (using defaults): {}", e);
                CheckinConfig::default()
            }
            Ok(rows) => {
                if let Some(row) = rows.as_array().and_then(|a| a.first()) {
                    serde_json::from_value(row.clone()).unwrap_or_default()
                } else {
                    CheckinConfig::default()
                }
            }
        }
    }

    /// Write a completed check-in to argus_checkin_log.
    pub async fn write_checkin_log(&self, entry: &CheckinLogEntry) -> Result<(), String> {
        let data = serde_json::to_value(entry)
            .map_err(|e| format!("Serialize error: {}", e))?;
        self.insert("argus_checkin_log", &data).await
    }

    /// Write a discourse post to argus_agent_discourse.
    pub async fn write_discourse(&self, post: &DiscoursePost) -> Result<(), String> {
        let data = serde_json::to_value(post)
            .map_err(|e| format!("Serialize error: {}", e))?;
        self.insert("argus_agent_discourse", &data).await
    }

    /// Read upcoming argus_schedule entries (next 7 days).
    pub async fn read_upcoming_schedule(&self) -> Result<Value, String> {
        self.select(
            "argus_schedule",
            "select=*&scheduled_time=gte.now()&order=scheduled_time.asc&limit=10",
        )
        .await
    }

    /// Read recent argus_memories.
    pub async fn read_recent_memories(&self, limit: usize) -> Result<Value, String> {
        self.select(
            "argus_memories",
            &format!("select=*&order=created_at.desc&limit={}", limit),
        )
        .await
    }
}
