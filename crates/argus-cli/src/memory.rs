//! Argus Memory System - Python Bridge
//! Shells out to memory.py for persistence (Supabase or SQLite)

use std::process::Command;
use serde::{Deserialize, Serialize};

fn memory_script_path() -> String {
    // Check if running from repo (dev) or installed
    let dev_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .map(|p| p.join("../../scripts/memory.py"));
    
    if let Some(p) = dev_path {
        if p.exists() {
            return p.to_string_lossy().to_string();
        }
    }
    
    // Fallback to ~/.argus/memory.py
    dirs::home_dir()
        .unwrap_or_default()
        .join(".argus")
        .join("memory.py")
        .to_string_lossy()
        .to_string()
}

#[derive(Debug, Deserialize)]
struct MemoryResponse {
    success: bool,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    memories: Option<Vec<MemoryRecord>>,
    #[serde(default)]
    deleted: Option<i32>,
    #[serde(default)]
    backend: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemoryRecord {
    #[serde(rename = "type")]
    pub memory_type: String,
    pub content: String,
    pub importance: f64,
    pub created_at: Option<String>,
}

pub struct ArgusMemory;

impl ArgusMemory {
    pub fn new() -> Self {
        Self
    }

    fn call_python(&self, command: &str, data: &serde_json::Value) -> Result<MemoryResponse, String> {
        let script = memory_script_path();
        let json_arg = serde_json::to_string(data).map_err(|e| e.to_string())?;
        
        let output = Command::new("python3")
            .arg(&script)
            .arg(command)
            .arg(&json_arg)
            .output()
            .map_err(|e| format!("Failed to run memory.py: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("memory.py failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout).map_err(|e| format!("Invalid response: {}", e))
    }

    pub fn remember(
        &self,
        memory_type: &str,
        content: &str,
        reasoning: Option<&str>,
        importance: f64,
        _tags: Option<Vec<String>>,
    ) -> Result<String, String> {
        let data = serde_json::json!({
            "type": memory_type,
            "content": content,
            "reasoning": reasoning,
            "importance": importance
        });
        
        let resp = self.call_python("remember", &data)?;
        
        if resp.success {
            Ok(format!("✅ {}", resp.message.unwrap_or_else(|| "Remembered".to_string())))
        } else {
            Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    pub fn recall(
        &self,
        query: Option<&str>,
        memory_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>, String> {
        let data = serde_json::json!({
            "query": query,
            "type": memory_type,
            "limit": limit
        });
        
        let resp = self.call_python("recall", &data)?;
        
        if resp.success {
            Ok(resp.memories.unwrap_or_default())
        } else {
            Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    pub fn forget(&self, content_match: &str) -> Result<String, String> {
        let data = serde_json::json!({
            "match": content_match
        });
        
        let resp = self.call_python("forget", &data)?;
        
        if resp.success {
            Ok(format!("✅ Forgot {} memories", resp.deleted.unwrap_or(0)))
        } else {
            Err(resp.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}
