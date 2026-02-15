//! Tool definitions and execution
//!
//! All built-in tools live here. The TUI, Telegram, and any future
//! frontends share the same tool implementations.

use crate::shell::{self, ShellPolicy};
use serde_json::Value;

/// All built-in tool definitions as OpenAI-compatible function schemas
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
                "description": "Execute a shell command and return output. Runs under an allowlist policy - only approved commands are permitted.",
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
                "description": "Search the web for current information, news, facts, or anything you don't know.",
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
                "description": "Store information in persistent memory. Use for facts, preferences, important details about the user.",
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
                "description": "Search and retrieve memories. Use at the start of conversations or when you need context.",
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
                "description": "Make an HTTP request to a URL. Supports GET, POST, PUT, DELETE methods. Useful for calling APIs, fetching web pages, or interacting with web services.",
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

/// Execute a built-in tool by name
///
/// Memory tools delegate to the provided MemoryBackend.
/// MCP tools are NOT handled here - the caller should check MCP after this returns None.
pub async fn execute_builtin(
    name: &str,
    args: &Value,
    shell_policy: &ShellPolicy,
    memory: &dyn MemoryBackend,
    http_client: &reqwest::Client,
) -> Option<String> {
    match name {
        "read_file" => Some(tool_read_file(args)),
        "list_directory" => Some(tool_list_directory(args)),
        "write_file" => Some(tool_write_file(args)),
        "shell" => Some(tool_shell(args, shell_policy).await),
        "web_search" => Some(tool_web_search(args, http_client).await),
        "remember" => Some(tool_remember(args, memory)),
        "recall" => Some(tool_recall(args, memory)),
        "forget" => Some(tool_forget(args, memory)),
        "http_request" => Some(tool_http_request(args, http_client).await),
        _ => None, // Not a built-in - caller should try MCP
    }
}

/// Trait for memory backends so tools don't care about implementation
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

/// A memory record returned from recall
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryRecord {
    pub memory_type: String,
    pub content: String,
    pub importance: f64,
    pub created_at: Option<String>,
}

// --- Tool implementations ---

fn tool_read_file(args: &Value) -> String {
    let path = args["path"].as_str().unwrap_or("");
    match std::fs::read_to_string(path) {
        Ok(content) => {
            if content.len() > 8000 {
                format!(
                    "{}...\n[truncated, {} bytes total]",
                    &content[..8000],
                    content.len()
                )
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
            let mut result = String::new();
            let mut items: Vec<_> = entries.flatten().collect();
            items.sort_by_key(|e| e.file_name());
            for entry in items {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                result.push_str(&format!(
                    "{}{}\n",
                    if is_dir { "📁 " } else { "📄 " },
                    name
                ));
            }
            if result.is_empty() {
                "(empty directory)".to_string()
            } else {
                result
            }
        }
        Err(e) => format!("Error listing directory: {}", e),
    }
}

fn tool_write_file(args: &Value) -> String {
    let path = args["path"].as_str().unwrap_or("");
    let content = args["content"].as_str().unwrap_or("");
    match std::fs::write(path, content) {
        Ok(_) => format!("✅ Written {} bytes to {}", content.len(), path),
        Err(e) => format!("Error writing file: {}", e),
    }
}

async fn tool_shell(args: &Value, policy: &ShellPolicy) -> String {
    let command = args["command"].as_str().unwrap_or("");
    match shell::execute_shell(policy, command).await {
        Ok(output) => output,
        Err(e) => format!("⛔ {}", e),
    }
}

async fn tool_web_search(args: &Value, client: &reqwest::Client) -> String {
    let query = args["query"].as_str().unwrap_or("");
    if query.is_empty() {
        return "No search query provided".to_string();
    }

    // Use DuckDuckGo HTML endpoint - no API key needed, no scraping Google
    let url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        urlencoding::encode(query)
    );

    let resp = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        )
        .send()
        .await;

    match resp {
        Ok(r) => match r.text().await {
            Ok(html) => {
                let mut results = Vec::new();
                // Parse DuckDuckGo result snippets
                for (i, chunk) in html.split("result__snippet").enumerate() {
                    if i == 0 || results.len() >= 6 {
                        continue;
                    }
                    if let Some(start) = chunk.find('>') {
                        if let Some(end) = chunk[start..].find('<') {
                            let snippet = &chunk[start + 1..start + end];
                            let clean: String = snippet
                                .replace("&quot;", "\"")
                                .replace("&amp;", "&")
                                .replace("&lt;", "<")
                                .replace("&gt;", ">")
                                .replace("&#39;", "'")
                                .replace("<b>", "")
                                .replace("</b>", "")
                                .chars()
                                .filter(|c| !c.is_control())
                                .collect();
                            let trimmed = clean.trim();
                            if trimmed.len() > 20 {
                                results.push(format!("• {}", trimmed));
                            }
                        }
                    }
                }

                // Also try to extract result titles/URLs
                let mut titles = Vec::new();
                for (i, chunk) in html.split("result__a").enumerate() {
                    if i == 0 || titles.len() >= 6 {
                        continue;
                    }
                    if let Some(href_start) = chunk.find("href=\"") {
                        let after = &chunk[href_start + 6..];
                        if let Some(href_end) = after.find('"') {
                            let href = &after[..href_end];
                            // DuckDuckGo wraps URLs in redirect
                            if let Some(url_start) = href.find("uddg=") {
                                let raw = &href[url_start + 5..];
                                if let Some(url_end) = raw.find('&') {
                                    let decoded = urlencoding::decode(&raw[..url_end])
                                        .unwrap_or_default()
                                        .to_string();
                                    titles.push(decoded);
                                }
                            }
                        }
                    }
                }

                if results.is_empty() {
                    "No results found - try rephrasing your search".to_string()
                } else {
                    let mut output = format!("🔍 Search results for '{}':\n\n", query);
                    for (i, result) in results.iter().enumerate() {
                        output.push_str(result);
                        if let Some(url) = titles.get(i) {
                            output.push_str(&format!("\n  ({})", url));
                        }
                        output.push_str("\n\n");
                    }
                    output
                }
            }
            Err(e) => format!("Error reading response: {}", e),
        },
        Err(e) => format!("Error searching: {}", e),
    }
}

fn tool_remember(args: &Value, memory: &dyn MemoryBackend) -> String {
    let content = args["content"].as_str().unwrap_or("");
    let memory_type = args["type"].as_str().unwrap_or("fact");
    let importance = args["importance"].as_f64().unwrap_or(5.0);
    let reasoning = args["reasoning"].as_str();

    match memory.remember(memory_type, content, reasoning, importance) {
        Ok(msg) => msg,
        Err(e) => format!("❌ Memory error: {}", e),
    }
}

fn tool_recall(args: &Value, memory: &dyn MemoryBackend) -> String {
    let query = args["query"].as_str();
    let memory_type = args["type"].as_str();
    let limit = args["limit"].as_u64().unwrap_or(10) as usize;

    match memory.recall(query, memory_type, limit) {
        Ok(memories) => {
            if memories.is_empty() {
                "No memories found.".to_string()
            } else {
                let mut result = String::from("🧠 Recalled memories:\n\n");
                for m in memories {
                    result.push_str(&format!(
                        "• [{}] (importance: {:.1}): {}\n",
                        m.memory_type, m.importance, m.content
                    ));
                }
                result
            }
        }
        Err(e) => format!("❌ Recall error: {}", e),
    }
}

fn tool_forget(args: &Value, memory: &dyn MemoryBackend) -> String {
    let content_match = args["content_match"].as_str().unwrap_or("");
    match memory.forget(content_match) {
        Ok(msg) => msg,
        Err(e) => format!("❌ Forget error: {}", e),
    }
}

async fn tool_http_request(args: &Value, client: &reqwest::Client) -> String {
    let url = args["url"].as_str().unwrap_or("");
    if url.is_empty() {
        return "No URL provided".to_string();
    }

    let method = args["method"].as_str().unwrap_or("GET");

    let mut builder = match method {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => client.get(url),
    };

    // Add custom headers
    if let Some(headers) = args["headers"].as_object() {
        for (key, val) in headers {
            if let Some(v) = val.as_str() {
                builder = builder.header(key.as_str(), v);
            }
        }
    }

    // Add body for POST/PUT
    if let Some(body) = args["body"].as_str() {
        builder = builder.body(body.to_string());
    }

    match builder.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let status_text = resp.status().to_string();
            match resp.text().await {
                Ok(body) => {
                    let truncated = if body.len() > 8000 {
                        format!("{}...\n[truncated, {} bytes total]", &body[..8000], body.len())
                    } else {
                        body
                    };
                    format!("HTTP {} {}\n\n{}", status, status_text, truncated)
                }
                Err(e) => format!("HTTP {} {} (body read error: {})", status, status_text, e),
            }
        }
        Err(e) => format!("HTTP request failed: {}", e),
    }
}
