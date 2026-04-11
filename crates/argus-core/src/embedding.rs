//! Semantic embedding layer for Argus
//!
//! Converts text → 768-dim vectors via google/gemini-embedding-001 on OpenRouter.
//! Stores vectors in Supabase pgvector tables.
//! Searches all three surfaces (memories, discourse, conversations) simultaneously.
//!
//! The result: every agent turn starts with semantically relevant context
//! already loaded — no explicit recall tool calls needed.
//! This is associative memory. The way human cognition actually works.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::supabase::SupabaseClient;

// ── Embedding model ───────────────────────────────────────────────────────

/// google/gemini-embedding-001 via OpenRouter
/// 768-dimensional output, optimized for semantic similarity
pub const EMBEDDING_MODEL: &str = "google/gemini-embedding-001";
pub const EMBEDDING_DIMS: usize = 768;

// ── Result types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticResult {
    pub source: String,     // "memory" | "discourse" | "conversation"
    pub content: String,
    pub from_agent: String,
    pub similarity: f64,
}

// ── Embedding client ──────────────────────────────────────────────────────

#[derive(Clone)]
pub struct EmbeddingClient {
    openrouter_key: String,
    supabase: SupabaseClient,
    http: Client,
}

impl EmbeddingClient {
    pub fn new(openrouter_key: impl Into<String>, supabase: SupabaseClient) -> Self {
        Self {
            openrouter_key: openrouter_key.into(),
            supabase,
            http: Client::new(),
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let truncated = if text.len() > 8000 { &text[..8000] } else { text };

        let body = serde_json::json!({
            "model": EMBEDDING_MODEL,
            "input": truncated,
        });

        let resp = self.http
            .post("https://openrouter.ai/api/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.openrouter_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Embedding request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Embedding API error {}: {}", status, body));
        }

        let json: Value = resp.json().await
            .map_err(|e| format!("Embedding parse error: {}", e))?;

        let vector = json["data"][0]["embedding"]
            .as_array()
            .ok_or("No embedding in response")?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect::<Vec<f32>>();

        if vector.len() != EMBEDDING_DIMS {
            return Err(format!("Expected {} dims, got {}", EMBEDDING_DIMS, vector.len()));
        }

        Ok(vector)
    }

    fn to_pg_vector(v: &[f32]) -> String {
        let inner = v.iter()
            .map(|f| format!("{:.8}", f))
            .collect::<Vec<_>>()
            .join(",");
        format!("[{}]", inner)
    }

    pub async fn store_memory_embedding(
        &self, memory_id: &str, content: &str, from_agent: &str,
    ) -> Result<(), String> {
        let vector = self.embed(content).await?;
        let data = serde_json::json!({
            "memory_id": memory_id,
            "from_agent": from_agent,
            "content": content,
            "embedding": Self::to_pg_vector(&vector),
            "model_used": EMBEDDING_MODEL,
        });
        self.supabase.insert("argus_memory_vectors", &data).await
    }

    pub async fn store_discourse_embedding(
        &self, discourse_id: &str, content: &str, from_agent: &str, post_type: &str,
    ) -> Result<(), String> {
        let vector = self.embed(content).await?;
        let data = serde_json::json!({
            "discourse_id": discourse_id,
            "from_agent": from_agent,
            "content": content,
            "post_type": post_type,
            "embedding": Self::to_pg_vector(&vector),
            "model_used": EMBEDDING_MODEL,
        });
        self.supabase.insert("argus_discourse_vectors", &data).await
    }

    pub async fn store_conversation_embedding(
        &self, conversation_id: &str, summary: &str, surface: &str,
    ) -> Result<(), String> {
        let vector = self.embed(summary).await?;
        let data = serde_json::json!({
            "conversation_id": conversation_id,
            "surface": surface,
            "summary": summary,
            "embedding": Self::to_pg_vector(&vector),
            "model_used": EMBEDDING_MODEL,
        });
        self.supabase.insert("argus_conversation_vectors", &data).await
    }

    pub async fn search_all(
        &self, query: &str,
        memories_count: i64, discourse_count: i64, conversation_count: i64,
    ) -> Result<Vec<SemanticResult>, String> {
        let vector = self.embed(query).await?;
        let body = serde_json::json!({
            "query_embedding": Self::to_pg_vector(&vector),
            "memories_count": memories_count,
            "discourse_count": discourse_count,
            "conversation_count": conversation_count,
            "min_similarity": 0.45
        });

        let result = self.supabase.rpc("search_all_semantic", &body).await?;
        let rows = result.as_array().ok_or("search_all_semantic returned non-array")?;

        Ok(rows.iter().filter_map(|row| {
            Some(SemanticResult {
                source: row["source"].as_str()?.to_string(),
                content: row["content"].as_str()?.to_string(),
                from_agent: row["from_agent"].as_str()?.to_string(),
                similarity: row["similarity"].as_f64()?,
            })
        }).collect())
    }

    pub fn format_context_block(results: &[SemanticResult]) -> Option<String> {
        if results.is_empty() { return None; }

        let mut lines = vec!["── SEMANTIC CONTEXT (auto-retrieved, most relevant) ──".to_string()];
        for r in results {
            let label = match r.source.as_str() {
                "memory" => "📍 memory",
                "discourse" => "💬 intranet",
                "conversation" => "🗒 past conv",
                other => other,
            };
            lines.push(format!(
                "[{label} | {agent} | {sim:.0}% match]\n{content}",
                label = label, agent = r.from_agent,
                sim = r.similarity * 100.0, content = r.content,
            ));
        }
        lines.push("── END SEMANTIC CONTEXT ──".to_string());
        Some(lines.join("\n\n"))
    }
}
