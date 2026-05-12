//! Supabase integration for persistent memory
//!
//! Syncs `argus_memories` rows to/from the Supabase project that also holds
//! safety_companion data.  The Argus-specific tables are:
//!   - argus_memories   (id, type, content, reasoning, importance, tags, created_at)
//!
//! This client intentionally mirrors the thin reqwest wrapper in
//! `argus_core::supabase` — no heavy SDK, just HTTP + Authorization header.
//!
//! Usage (from daemon startup):
//!   let mem = SupabaseMemory::connect(&url, &service_key).await?;
//!   let ctx = mem.load_context("coffee preferences").await?;
//!   mem.store_memory("User prefers dark roast", "preference").await?;

use reqwest::Client;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SupabaseError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Supabase API error ({status}): {body}")]
    Api { status: u16, body: String },

    #[error("Parse error: {0}")]
    Parse(String),
}

/// Supabase client scoped to argus memory tables.
#[derive(Clone)]
pub struct SupabaseMemory {
    base_url: String,
    jwt: String,
    client: Client,
}

impl SupabaseMemory {
    /// Connect — just validates credentials are non-empty and builds the client.
    /// Does not make a network call on construction so startup is fast.
    pub async fn connect(url: &str, key: &str) -> Result<Self, SupabaseError> {
        Ok(Self {
            base_url: url.trim_end_matches('/').to_string(),
            jwt: key.to_string(),
            client: Client::new(),
        })
    }

    fn rest_url(&self, table: &str) -> String {
        format!("{}/rest/v1/{}", self.base_url, table)
    }

    fn headers(&self) -> Result<reqwest::header::HeaderMap, SupabaseError> {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
        let mut h = HeaderMap::new();
        let bearer = format!("Bearer {}", self.jwt);
        let auth_val = HeaderValue::from_str(&bearer)
            .map_err(|_| SupabaseError::Parse("Invalid characters in auth token".into()))?;
        let key_val = HeaderValue::from_str(&self.jwt)
            .map_err(|_| SupabaseError::Parse("Invalid characters in API key".into()))?;
        h.insert(AUTHORIZATION, auth_val);
        h.insert("apikey", key_val);
        h.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(h)
    }

    async fn check(&self, resp: reqwest::Response) -> Result<reqwest::Response, SupabaseError> {
        if resp.status().is_success() {
            return Ok(resp);
        }
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(SupabaseError::Api { status, body })
    }

    // ── Public API ──────────────────────────────────────────────────────────

    /// Load up to `limit` memories relevant to `query` (case-insensitive substring match).
    /// Results are ordered by importance descending.
    pub async fn load_context(&self, query: &str) -> Result<Vec<String>, SupabaseError> {
        let encoded = urlencoding::encode(&format!("%{}%", query));
        let url = format!(
            "{}?select=content&content=ilike.{}&order=importance.desc&limit=20",
            self.rest_url("argus_memories"),
            encoded
        );

        let resp = self
            .client
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        let resp = self.check(resp).await?;

        let rows: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SupabaseError::Parse(e.to_string()))?;

        let memories = rows
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|r| r["content"].as_str().map(|s| s.to_string()))
            .collect();

        Ok(memories)
    }

    /// Store a single memory entry.
    pub async fn store_memory(
        &self,
        content: &str,
        memory_type: &str,
    ) -> Result<(), SupabaseError> {
        let payload = serde_json::json!({
            "type":      memory_type,
            "content":   content,
            "importance": 5.0,
        });

        let resp = self
            .client
            .post(&self.rest_url("argus_memories"))
            .headers(self.headers()?)
            .header("Prefer", "return=minimal")
            .json(&payload)
            .send()
            .await?;
        self.check(resp).await?;

        Ok(())
    }

    /// Fetch the N most recent memories (any type).
    pub async fn recent(&self, limit: usize) -> Result<Vec<(String, String, f64)>, SupabaseError> {
        let url = format!(
            "{}?select=type,content,importance&order=created_at.desc&limit={}",
            self.rest_url("argus_memories"),
            limit
        );

        let resp = self.client.get(&url).headers(self.headers()?).send().await?;
        let resp = self.check(resp).await?;

        let rows: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SupabaseError::Parse(e.to_string()))?;

        let out = rows
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|r| {
                let t = r["type"].as_str()?.to_string();
                let c = r["content"].as_str()?.to_string();
                let i = r["importance"].as_f64().unwrap_or(5.0);
                Some((t, c, i))
            })
            .collect();

        Ok(out)
    }

    /// Delete memories whose content matches the given substring.
    pub async fn forget(&self, content_match: &str) -> Result<u64, SupabaseError> {
        let encoded = urlencoding::encode(&format!("%{}%", content_match));
        let url = format!(
            "{}?content=ilike.{}",
            self.rest_url("argus_memories"),
            encoded
        );

        let resp = self
            .client
            .delete(&url)
            .headers(self.headers()?)
            .header("Prefer", "return=minimal,count=exact")
            .send()
            .await?;

        let resp = self.check(resp).await?;

        // Supabase returns Content-Range: */N on deletes with count=exact
        let deleted = resp
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split('/').last())
            .and_then(|n| n.parse().ok())
            .unwrap_or(0u64);

        Ok(deleted)
    }
}
