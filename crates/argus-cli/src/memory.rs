//! Argus Memory System - Supabase-backed persistent memory

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

const SUPABASE_URL: &str = "https://YOUR_SUPABASE_PROJECT_ID_2.supabase.co";
const SUPABASE_KEY: &str = "sb_publishable_1IG8vV7Q6hamc_19TLkQ3g_qUeZV9As";

#[derive(Debug, Serialize, Deserialize)]
pub struct Memory {
    pub id: Option<String>,
    pub user_id: String,
    #[serde(rename = "type")]
    pub memory_type: String,
    pub content: String,
    pub reasoning: Option<String>,
    pub context: Option<String>,
    pub importance: f64,
    pub tags: Option<Vec<String>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub display_name: Option<String>,
    pub preferences: serde_json::Value,
    pub communication_style: serde_json::Value,
    pub technical_level: String,
}

pub struct ArgusMemory {
    client: reqwest::Client,
    user_id: String,
}

impl ArgusMemory {
    pub fn new(user_id: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            user_id: user_id.to_string(),
        }
    }

    /// Store a new memory
    pub async fn remember(&self, memory_type: &str, content: &str, reasoning: Option<&str>, importance: f64, tags: Option<Vec<String>>) -> Result<String, String> {
        let memory = serde_json::json!({
            "user_id": self.user_id,
            "type": memory_type,
            "content": content,
            "reasoning": reasoning,
            "importance": importance,
            "tags": tags
        });

        let resp = self.client
            .post(format!("{}/rest/v1/argus_memories", SUPABASE_URL))
            .header("apikey", SUPABASE_KEY)
            .header("Authorization", format!("Bearer {}", SUPABASE_KEY))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&memory)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            Ok(format!("✅ Remembered: {}", &content[..content.len().min(50)]))
        } else {
            let err = resp.text().await.unwrap_or_default();
            Err(format!("Failed to store memory: {}", err))
        }
    }

    /// Recall memories by type or search
    pub async fn recall(&self, query: Option<&str>, memory_type: Option<&str>, limit: usize) -> Result<Vec<Memory>, String> {
        let mut url = format!(
            "{}/rest/v1/argus_memories?user_id=eq.{}&order=importance.desc,created_at.desc&limit={}",
            SUPABASE_URL, self.user_id, limit
        );

        if let Some(t) = memory_type {
            url.push_str(&format!("&type=eq.{}", t));
        }

        if let Some(q) = query {
            url.push_str(&format!("&content=ilike.*{}*", urlencoding::encode(q)));
        }

        let resp = self.client
            .get(&url)
            .header("apikey", SUPABASE_KEY)
            .header("Authorization", format!("Bearer {}", SUPABASE_KEY))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            let memories: Vec<Memory> = resp.json().await.map_err(|e| e.to_string())?;
            Ok(memories)
        } else {
            Err("Failed to recall memories".to_string())
        }
    }

    /// Get all memories for context loading
    pub async fn load_context(&self) -> Result<String, String> {
        let memories = self.recall(None, None, 20).await?;
        
        if memories.is_empty() {
            return Ok("No memories stored yet.".to_string());
        }

        let mut context = String::from("## Argus Memory Context\n\n");
        for mem in memories {
            context.push_str(&format!(
                "- [{}] (importance: {:.1}): {}\n",
                mem.memory_type, mem.importance, mem.content
            ));
        }
        Ok(context)
    }

    /// Store or update user profile
    pub async fn update_user_profile(&self, display_name: Option<&str>, preferences: Option<serde_json::Value>) -> Result<String, String> {
        let profile = serde_json::json!({
            "user_id": self.user_id,
            "display_name": display_name,
            "preferences": preferences.unwrap_or(serde_json::json!({})),
            "last_interaction": Utc::now()
        });

        let resp = self.client
            .post(format!("{}/rest/v1/argus_user_profiles", SUPABASE_URL))
            .header("apikey", SUPABASE_KEY)
            .header("Authorization", format!("Bearer {}", SUPABASE_KEY))
            .header("Content-Type", "application/json")
            .header("Prefer", "resolution=merge-duplicates,return=representation")
            .json(&profile)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            Ok("✅ Profile updated".to_string())
        } else {
            Err("Failed to update profile".to_string())
        }
    }

    /// Get user profile
    pub async fn get_user_profile(&self) -> Result<Option<UserProfile>, String> {
        let url = format!(
            "{}/rest/v1/argus_user_profiles?user_id=eq.{}&limit=1",
            SUPABASE_URL, self.user_id
        );

        let resp = self.client
            .get(&url)
            .header("apikey", SUPABASE_KEY)
            .header("Authorization", format!("Bearer {}", SUPABASE_KEY))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            let profiles: Vec<UserProfile> = resp.json().await.map_err(|e| e.to_string())?;
            Ok(profiles.into_iter().next())
        } else {
            Err("Failed to get profile".to_string())
        }
    }

    /// Store a learning (from mistakes or discoveries)
    pub async fn learn(&self, lesson: &str, source: &str, importance: i32) -> Result<String, String> {
        let learning = serde_json::json!({
            "user_id": self.user_id,
            "lesson": lesson,
            "source": source,
            "importance": importance,
            "applies_to": "all_users"
        });

        let resp = self.client
            .post(format!("{}/rest/v1/argus_learnings", SUPABASE_URL))
            .header("apikey", SUPABASE_KEY)
            .header("Authorization", format!("Bearer {}", SUPABASE_KEY))
            .header("Content-Type", "application/json")
            .json(&learning)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            Ok(format!("✅ Learned: {}", lesson))
        } else {
            Err("Failed to store learning".to_string())
        }
    }

    /// Delete a memory by content match
    pub async fn forget(&self, content_match: &str) -> Result<String, String> {
        let url = format!(
            "{}/rest/v1/argus_memories?user_id=eq.{}&content=ilike.*{}*",
            SUPABASE_URL, self.user_id, urlencoding::encode(content_match)
        );

        let resp = self.client
            .delete(&url)
            .header("apikey", SUPABASE_KEY)
            .header("Authorization", format!("Bearer {}", SUPABASE_KEY))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            Ok(format!("✅ Forgot memories matching: {}", content_match))
        } else {
            Err("Failed to delete memory".to_string())
        }
    }
}
