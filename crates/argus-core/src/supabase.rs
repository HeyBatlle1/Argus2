//! Supabase REST API client for Argus
//!
//! Thin reqwest wrapper around Supabase's PostgREST endpoint.
//! No heavy SDK needed — it's just HTTP with an Authorization header.
//!
//! Tables served:
//!   Reads:  argus_checkin_config, argus_schedule, argus_memories
//!   Writes: argus_checkin_log, argus_agent_discourse, argus_conversations, argus_memories
//!   RPC:    search_all_semantic (pgvector similarity search)

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Config shape (matches argus_checkin_config table) ──────────────────────
//
// quiet_hours_start / quiet_hours_end are stored as plain integers in
// Postgres (e.g. 23, 7) representing the hour boundary.  The old
// Option<String> type caused serde to reject the entire row and fall back
// to CheckinConfig::default(), silently ignoring the real interval_minutes
// value.  Fixed to Option<i64> to match the actual column type.

#[derive(Debug, Clone, Deserialize)]
pub struct CheckinConfig {
    pub interval_minutes: i64,
    pub checkin_type: String,
    /// Hour at which quiet hours begin (0–23). Stored as integer in DB.
    pub quiet_hours_start: Option<i64>,
    /// Hour at which quiet hours end (0–23). Stored as integer in DB.
    pub quiet_hours_end: Option<i64>,
    pub telegram_enabled: bool,
}

impl Default for CheckinConfig {
    fn default() -> Self {
        Self {
            interval_minutes: 60,
            checkin_type: "brief".to_string(),
            quiet_hours_start: Some(23),
            quiet_hours_end: Some(7),
            telegram_enabled: true,
        }
    }
}

// ── Discourse post shape ───────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct DiscoursePost {
    pub from_agent: String,
    pub post_type: String,    // "finding" | "question" | "proposal"
    pub content: String,
    pub task_context: Option<String>,
    pub requires_human_review: bool,
}

/// A post read back from argus_agent_discourse
#[derive(Debug, Clone, Deserialize)]
pub struct DiscourseRecord {
    pub from_agent: String,
    pub post_type: String,
    pub content: String,
    pub created_at: Option<String>,
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

    fn rpc_url(&self, function: &str) -> String {
        format!("{}/rest/v1/rpc/{}", self.base_url, function)
    }

    /// GET /rest/v1/{table}?{query_params}
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

    pub async fn patch(&self, url: &str, data: &Value) -> Result<(), String> {
        let resp = self.client
            .patch(url)
            .header("Authorization", format!("Bearer {}", self.jwt))
            .header("apikey", &self.jwt)
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(data)
            .send()
            .await
            .map_err(|e| format!("Supabase PATCH failed: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Supabase PATCH error: {}", body));
        }
        Ok(())
    }

    /// POST /rest/v1/rpc/{function} — call a Postgres function
    /// Used for pgvector similarity search (search_all_semantic, etc.)
    pub async fn rpc(&self, function: &str, params: &Value) -> Result<Value, String> {
        let resp = self.client
            .post(&self.rpc_url(function))
            .header("Authorization", format!("Bearer {}", self.jwt))
            .header("apikey", &self.jwt)
            .header("Content-Type", "application/json")
            .json(params)
            .send()
            .await
            .map_err(|e| format!("Supabase RPC {} failed: {}", function, e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Supabase RPC {} error {}: {}", function, status, body));
        }

        resp.json::<Value>()
            .await
            .map_err(|e| format!("Supabase RPC response parse error: {}", e))
    }

    // ── Domain helpers ─────────────────────────────────────────────────────

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

    pub async fn write_checkin_log(&self, entry: &CheckinLogEntry) -> Result<(), String> {
        let data = serde_json::to_value(entry)
            .map_err(|e| format!("Serialize error: {}", e))?;
        self.insert("argus_checkin_log", &data).await
    }

    pub async fn write_discourse(&self, post: &DiscoursePost) -> Result<(), String> {
        let data = serde_json::to_value(post)
            .map_err(|e| format!("Serialize error: {}", e))?;
        self.insert("argus_agent_discourse", &data).await
    }

    // ── Triage queue ──────────────────────────────────────────────────────

    pub async fn enqueue_triage(&self, entry: &crate::triage::TriageEntry) -> Result<(), String> {
        let data = serde_json::to_value(entry)
            .map_err(|e| format!("Serialize error: {}", e))?;
        self.insert("argus_triage_queue", &data).await
    }

    pub async fn read_pending_triage(&self) -> Result<Vec<serde_json::Value>, String> {
        match self.select("argus_triage_queue", "select=*&disposition=eq.pending&order=created_at.asc&limit=20").await {
            Ok(val) => Ok(val.as_array().cloned().unwrap_or_default()),
            Err(e) => Err(e),
        }
    }

    pub async fn mark_triage_processed(&self, id: &str, disposition: &str) -> Result<(), String> {
        let url = format!(
            "{}/rest/v1/argus_triage_queue?id=eq.{}",
            self.base_url, id
        );
        let body = serde_json::json!({ "disposition": disposition });
        self.patch(&url, &body).await
    }

    // ── Triage flags ──────────────────────────────────────────────────────

    pub async fn write_triage_flag(&self, flag: &crate::triage::TriageFlag) -> Result<(), String> {
        let data = serde_json::to_value(flag)
            .map_err(|e| format!("Serialize error: {}", e))?;
        self.insert("argus_triage_flags", &data).await
    }

    pub async fn read_pending_flags(&self) -> Result<Vec<serde_json::Value>, String> {
        match self.select("argus_triage_flags", "select=*&disposition=eq.pending&order=created_at.desc&limit=20").await {
            Ok(val) => Ok(val.as_array().cloned().unwrap_or_default()),
            Err(e) => Err(e),
        }
    }

    pub async fn read_upcoming_schedule(&self) -> Result<Value, String> {
        self.select(
            "argus_schedule",
            "select=*&scheduled_time=gte.now()&order=scheduled_time.asc&limit=10",
        )
        .await
    }

    pub async fn read_recent_memories(&self, limit: usize) -> Result<Value, String> {
        self.select(
            "argus_memories",
            &format!("select=*&order=created_at.desc&limit={}", limit),
        )
        .await
    }

    /// Read recent posts from the intranet, optionally excluding the calling agent.
    /// Used to inject other agents' findings as context before a turn starts.
    pub async fn read_recent_discourse(
        &self,
        limit: usize,
        exclude_author: Option<&str>,
    ) -> Result<Vec<DiscourseRecord>, String> {
        let query = match exclude_author {
            Some(author) => format!(
                "select=from_agent,post_type,content,created_at&from_agent=neq.{}&order=created_at.desc&limit={}",
                urlencoding::encode(author), limit
            ),
            None => format!(
                "select=from_agent,post_type,content,created_at&order=created_at.desc&limit={}",
                limit
            ),
        };

        let rows = self.select("argus_agent_discourse", &query).await?;
        let records = rows.as_array()
            .ok_or("argus_agent_discourse returned non-array")?
            .iter()
            .filter_map(|row| serde_json::from_value::<DiscourseRecord>(row.clone()).ok())
            .collect();
        Ok(records)
    }
}
