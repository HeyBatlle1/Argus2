//! Tool definitions and execution
//!
//! All built-in tools live here. Shared across TUI, Telegram, and any future frontends.

use crate::shell::{self, ShellPolicy};
use serde_json::Value;

pub fn builtin_tool_schemas() -> Vec<Value> {
    serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Read the contents of a file from the filesystem",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "The path to the file to read" }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "list_directory",
                "description": "List files and directories in a given path",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "The directory path to list" }
                    },
                    "required": ["path"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "write_file",
                "description": "Write content to a file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "The path to write to" },
                        "content": { "type": "string", "description": "The content to write" }
                    },
                    "required": ["path", "content"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "shell",
                "description": "Execute a shell command and return output. Runs under an allowlist policy.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "The shell command to execute" }
                    },
                    "required": ["command"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "Search the web for current information, news, facts, or anything you don't know. Uses Brave Search API.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "The search query" }
                    },
                    "required": ["query"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "remember",
                "description": "Store information in persistent memory.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "The information to remember" },
                        "type": {
                            "type": "string",
                            "enum": ["fact", "preference", "task", "learning", "relationship"],
                            "description": "Category of memory"
                        },
                        "importance": { "type": "number", "description": "Importance score 1-10" },
                        "reasoning": { "type": "string", "description": "Why this is worth remembering" }
                    },
                    "required": ["content", "type"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "recall",
                "description": "Search and retrieve memories.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search term to find relevant memories" },
                        "type": {
                            "type": "string",
                            "enum": ["fact", "preference", "task", "learning", "relationship"],
                            "description": "Filter by memory type"
                        },
                        "limit": { "type": "number", "description": "Max memories to return (default 10)" }
                    }
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "forget",
                "description": "Delete memories matching a search term.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "content_match": { "type": "string", "description": "Text to match for deletion" }
                    },
                    "required": ["content_match"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "http_request",
                "description": "Make an HTTP request to a URL. Supports GET, POST, PUT, DELETE.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "The URL to request" },
                        "method": {
                            "type": "string",
                            "enum": ["GET", "POST", "PUT", "DELETE"],
                            "description": "HTTP method (default: GET)"
                        },
                        "body": { "type": "string", "description": "Request body (for POST/PUT)" },
                        "headers": {
                            "type": "object",
                            "description": "Additional headers as key-value pairs",
                            "additionalProperties": { "type": "string" }
                        }
                    },
                    "required": ["url"]
                }
            }
        }
    ]).as_array().unwrap().clone()
}

pub async fn execute_builtin(
    name: &str,
    args: &Value,
    shell_policy: &ShellPolicy,
    memory: &dyn MemoryBackend,
    http_client: &reqwest::Client,
    brave_search_key: Option<&str>,
) -> Option<String> {
    match name {
        "read_file"      => Some(tool_read_file(args)),
        "list_directory" => Some(tool_list_directory(args)),
        "write_file"     => Some(tool_write_file(args)),
        "shell"          => Some(tool_shell(args, shell_policy).await),
        "web_search"     => Some(tool_web_search(args, http_client, brave_search_key).await),
        "remember"       => Some(tool_remember(args, memory)),
        "recall"         => Some(tool_recall(args, memory)),
        "forget"         => Some(tool_forget(args, memory)),
        "http_request"   => Some(tool_http_request(args, http_client).await),
        _                => None,
    }
}

pub trait MemoryBackend: Send + Sync {
    fn remember(
        &self,
        memory_type: &str,
        content: &str,
        reasoning: Option<&str>,
        importance: f64,
    ) -> Result<String, String>;

    fn recall(
        &self,
        query: Option<&str>,
        memory_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>, String>;

    fn forget(&self, content_match: &str) -> Result<String, String>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryRecord {
    pub id: i64,
    pub memory_type: String,
    pub content: String,
    pub importance: f64,
    pub created_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

fn tool_read_file(args: &Value) -> String {
    let path = args["path"].as_str().unwrap_or("");
    match std::fs::read_to_string(path) {
        Ok(content) => {
            if content.len() > 8000 {
                format!("{}...\n[truncated, {} bytes total]", &content[..8000], content.len())
            } else {
                content
            }
        }
        Err(e) => format!("Error reading file: {}", e),
    }
}

fn tool_list_directory(args: &Value) -> String {
    let path = args["path"].as_str().unwrap_or(".");
    match std::fs::read_dir(path) {
        Ok(entries) => {
            let mut items: Vec<_> = entries.flatten().collect();
            items.sort_by_key(|e| e.file_name());
            let mut result = String::new();
            for entry in items {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                result.push_str(&format!("{} {}\n",
                    if is_dir { "[DIR]" } else { "[FILE]" },
                    name
                ));
            }
            if result.is_empty() { "(empty directory)".to_string() } else { result }
        }
        Err(e) => format!("Error listing directory: {}", e),
    }
}

fn tool_write_file(args: &Value) -> String {
    let path = args["path"].as_str().unwrap_or("");
    let content = args["content"].as_str().unwrap_or("");
    match std::fs::write(path, content) {
        Ok(_) => format!("Written {} bytes to {}", content.len(), path),
        Err(e) => format!("Error writing file: {}", e),
    }
}

async fn tool_shell(args: &Value, policy: &ShellPolicy) -> String {
    let command = args["command"].as_str().unwrap_or("");
    match shell::execute_shell(policy, command).await {
        Ok(output) => output,
        Err(e) => format!("Shell error: {}", e),
    }
}

async fn tool_web_search(args: &Value, client: &reqwest::Client, brave_key: Option<&str>) -> String {
    let query = args["query"].as_str().unwrap_or("");
    if query.is_empty() {
        return "No search query provided".to_string();
    }
    match brave_key {
        Some(key) => brave_search(query, client, key).await,
        None => "web_search not configured: run 'argus vault set brave_search_api_key YOUR_KEY'".to_string(),
    }
}

async fn brave_search(query: &str, client: &reqwest::Client, api_key: &str) -> String {
    let url = format!(
        "https://api.search.brave.com/res/v1/web/search?q={}&count=8&text_decorations=false&result_filter=web",
        urlencoding::encode(query)
    );

    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .header("Accept-Encoding", "gzip")
        .header("X-Subscription-Token", api_key)
        .send()
        .await;

    match resp {
        Err(e) => format!("Brave Search request failed: {}", e),
        Ok(r) => {
            let status = r.status();
            if !status.is_success() {
                let body = r.text().await.unwrap_or_default();
                return format!("Brave Search error {}: {}", status, body);
            }
            match r.json::<serde_json::Value>().await {
                Err(e) => format!("Failed to parse Brave Search response: {}", e),
                Ok(json) => {
                    let results = json["web"]["results"].as_array();
                    match results {
                        None => format!("No results found for '{}'", query),
                        Some(results) if results.is_empty() => format!("No results found for '{}'", query),
                        Some(results) => {
                            let mut output = format!("Search results for '{}':\n\n", query);
                            for r in results.iter().take(6) {
                                let title = r["title"].as_str().unwrap_or("Untitled");
                                let url   = r["url"].as_str().unwrap_or("");
                                let desc  = r["description"].as_str().unwrap_or("");
                                output.push_str(&format!("[{}]\n{}\n{}\n\n", title, desc, url));
                            }
                            output
                        }
                    }
                }
            }
        }
    }
}

fn tool_remember(args: &Value, memory: &dyn MemoryBackend) -> String {
    let content     = args["content"].as_str().unwrap_or("");
    let memory_type = args["type"].as_str().unwrap_or("fact");
    let importance  = args["importance"].as_f64().unwrap_or(5.0);
    let reasoning   = args["reasoning"].as_str();
    match memory.remember(memory_type, content, reasoning, importance) {
        Ok(msg) => msg,
        Err(e)  => format!("Memory error: {}", e),
    }
}

fn tool_recall(args: &Value, memory: &dyn MemoryBackend) -> String {
    let query       = args["query"].as_str();
    let memory_type = args["type"].as_str();
    let limit       = args["limit"].as_u64().unwrap_or(10) as usize;
    match memory.recall(query, memory_type, limit) {
        Err(e)   => format!("Recall error: {}", e),
        Ok(mems) => {
            if mems.is_empty() {
                "No memories found.".to_string()
            } else {
                let mut result = String::from("Recalled memories:\n\n");
                for m in mems {
                    result.push_str(&format!("- [{}] (importance: {:.1}): {}\n",
                        m.memory_type, m.importance, m.content));
                }
                result
            }
        }
    }
}

fn tool_forget(args: &Value, memory: &dyn MemoryBackend) -> String {
    let content_match = args["content_match"].as_str().unwrap_or("");
    match memory.forget(content_match) {
        Ok(msg) => msg,
        Err(e)  => format!("Forget error: {}", e),
    }
}

async fn tool_http_request(args: &Value, client: &reqwest::Client) -> String {
    let url = args["url"].as_str().unwrap_or("");
    if url.is_empty() { return "No URL provided".to_string(); }

    let method = args["method"].as_str().unwrap_or("GET");
    let mut builder = match method {
        "POST"   => client.post(url),
        "PUT"    => client.put(url),
        "DELETE" => client.delete(url),
        _        => client.get(url),
    };

    if let Some(headers) = args["headers"].as_object() {
        for (key, val) in headers {
            if let Some(v) = val.as_str() {
                builder = builder.header(key.as_str(), v);
            }
        }
    }

    if let Some(body) = args["body"].as_str() {
        builder = builder.body(body.to_string());
    }

    match builder.send().await {
        Err(e) => format!("HTTP request failed: {}", e),
        Ok(resp) => {
            let status = resp.status();
            match resp.text().await {
                Err(e) => format!("HTTP {} (body read error: {})", status, e),
                Ok(body) => {
                    let truncated = if body.len() > 8000 {
                        format!("{}...\n[truncated, {} bytes total]", &body[..8000], body.len())
                    } else {
                        body
                    };
                    format!("HTTP {}\n\n{}", status, truncated)
                }
            }
        }
    }
}
