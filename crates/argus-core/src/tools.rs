//! Tool definitions and execution
//!
//! All built-in tools live here. Shared across TUI, Telegram, and any future frontends.

use crate::shell::{ShellPolicy, PermissionPrompter, PermissionRequest, PermissionDecision};
use serde_json::Value;

const MAX_FILE_CHARS: usize = 8_000;
const MAX_DIR_ENTRIES: usize = 200;
const MAX_SEARCH_RESULTS: usize = 6;

pub fn builtin_tool_schemas() -> Vec<Value> {
    serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Read the contents of a file from the filesystem. Output capped at 8000 chars.",
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
                "description": "List files and directories in a given path. Returns up to 200 entries — use a more specific path for large directories.",
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
                "name": "run_python",
                "description": "Execute Python 3 code in the workspace sandbox and return stdout/stderr. Use for data analysis, computations, generating files, or anything requiring a proper Python runtime. Output from print() appears in stdout.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "Python 3 source code to execute" },
                        "timeout": { "type": "number", "description": "Max execution seconds (default 30, max 120)" }
                    },
                    "required": ["code"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "run_node",
                "description": "Execute JavaScript/Node.js code in the workspace sandbox and return stdout/stderr. Use for JSON processing, web scraping logic, or anything needing a Node runtime. console.log() output appears in stdout.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "JavaScript (Node.js) source code to execute" },
                        "timeout": { "type": "number", "description": "Max execution seconds (default 30, max 120)" }
                    },
                    "required": ["code"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "list_tools",
                "description": "List every tool available to you in this session — built-ins (shell, web_search, memory, file ops, http) plus any MCP-connected tools. Call this when you need to know your full capabilities.",
                "parameters": {
                    "type": "object",
                    "properties": {}
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
    shell_prompter: Option<std::sync::Arc<dyn PermissionPrompter>>,
    exec_auth_token: Option<&str>,
) -> Option<String> {
    match name {
        "read_file"      => Some(tool_read_file(args)),
        "list_directory" => Some(tool_list_directory(args)),
        "write_file"     => Some(tool_write_file(args)),
        "shell"          => Some(tool_shell(args, shell_policy, shell_prompter, http_client, exec_auth_token).await),
        "web_search"     => Some(tool_web_search(args, http_client, brave_search_key).await),
        "remember"       => Some(tool_remember(args, memory)),
        "recall"         => Some(tool_recall(args, memory)),
        "forget"         => Some(tool_forget(args, memory)),
        "http_request"   => Some(tool_http_request(args, http_client).await),
        "run_python"     => Some(tool_run_code("python", args, http_client, exec_auth_token).await),
        "run_node"       => Some(tool_run_code("javascript", args, http_client, exec_auth_token).await),
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
            if content.len() > MAX_FILE_CHARS {
                format!("{}...\n[truncated, {} bytes total]", &content[..MAX_FILE_CHARS], content.len())
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
            let total = items.len();
            let mut result = String::new();
            for entry in items.iter().take(MAX_DIR_ENTRIES) {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                result.push_str(&format!("{} {}\n",
                    if is_dir { "[DIR]" } else { "[FILE]" },
                    name
                ));
            }
            if total > MAX_DIR_ENTRIES {
                result.push_str(&format!(
                    "\n[showing {}/{} entries — use a more specific path]\n",
                    MAX_DIR_ENTRIES, total
                ));
            }
            if result.is_empty() { "(empty directory)".to_string() } else { result }
        }
        Err(e) => format!("Error listing directory: {}", e),
    }
}

/// Validate a write path against the allowlist/blocklist policy.
/// Called before every write_file to prevent the agent from touching
/// vault files, SSH keys, shell configs, or system directories.
fn validate_write_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("No path provided".to_string());
    }

    let p = std::path::Path::new(path);
    // Resolve to absolute; if the file doesn't exist yet use the parent dir
    let canonical = p.canonicalize()
        .or_else(|_| p.parent().and_then(|par| par.canonicalize().ok().map(|c| c.join(p.file_name().unwrap_or_default())))
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "cannot resolve path")))
        .unwrap_or_else(|_| p.to_path_buf());

    let s = canonical.to_string_lossy().to_lowercase();

    // Hard-blocked path fragments — never writable regardless of prefix
    let blocked: &[&str] = &[
        ".argus/vault",
        ".ssh/",
        "authorized_keys",
        ".zshrc",
        ".bashrc",
        ".bash_profile",
        ".profile",
        "/etc/",
        "/sys/",
        "/proc/",
        "/dev/",
        "/boot/",
        "/sbin/",
        "/usr/bin/",
        "/usr/sbin/",
    ];

    for fragment in blocked {
        if s.contains(fragment) {
            return Err(format!("Write blocked: '{}' matches protected path pattern '{}'", path, fragment));
        }
    }

    Ok(())
}

fn tool_write_file(args: &Value) -> String {
    let path = args["path"].as_str().unwrap_or("");
    let content = args["content"].as_str().unwrap_or("");

    if let Err(reason) = validate_write_path(path) {
        return format!("Write blocked: {}", reason);
    }

    // Resolve the canonical path and use it for the actual write — this ensures
    // the blocklist check and the write operation target the same path, closing
    // the gap where a symlink could redirect a write after the check.
    let write_path = {
        let p = std::path::Path::new(path);
        p.canonicalize()
            .or_else(|_| p.parent()
                .and_then(|par| par.canonicalize().ok()
                    .map(|c| c.join(p.file_name().unwrap_or_default())))
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "cannot resolve")))
            .unwrap_or_else(|_| p.to_path_buf())
    };

    match std::fs::write(&write_path, content) {
        Ok(_) => format!("Written {} bytes to {}", content.len(), write_path.display()),
        Err(e) => format!("Error writing file: {}", e),
    }
}

async fn tool_shell(
    args: &Value,
    policy: &ShellPolicy,
    prompter: Option<std::sync::Arc<dyn PermissionPrompter>>,
    http_client: &reqwest::Client,
    exec_auth_token: Option<&str>,
) -> String {
    let command = args["command"].as_str().unwrap_or("");
    if command.is_empty() {
        return "No command provided".to_string();
    }

    // Step 1: fast, non-blocking risk evaluation (pure pattern matching — no I/O)
    let risk = match policy.evaluate(command) {
        Err(e) => return format!("Shell blocked: {}", e),
        Ok(r)  => r,
    };

    // Step 2: HIGH risk requires operator approval via the prompter.
    // The TelegramPrompter polls for /approve or /deny for up to 60 seconds using
    // std::thread::sleep. We run it in spawn_blocking so the tokio runtime is never
    // starved — concurrent Telegram messages and WebSocket turns remain responsive.
    if risk >= policy.approval_threshold {
        match prompter {
            None => return format!(
                "Shell blocked: {} risk command requires approval but no prompter is configured. \
                 Set up Telegram bot to enable HIGH risk approval.",
                risk.as_str()
            ),
            Some(p) => {
                let request = PermissionRequest {
                    command: command.to_string(),
                    risk,
                    reason: format!("{} risk command requires approval before execution", risk.as_str()),
                };
                let decision = tokio::task::spawn_blocking(move || p.prompt(&request))
                    .await
                    .unwrap_or_else(|_| PermissionDecision::Deny {
                        reason: "Prompter task panicked or was cancelled".to_string(),
                    });
                if let PermissionDecision::Deny { reason } = decision {
                    return format!("Shell blocked: {}", reason);
                }
            }
        }
    }

    // Route execution to argus-workspace via internal Docker network
    let payload = serde_json::json!({ "command": command });
    let mut req = http_client
        .post("http://argus-workspace:9001/exec")
        .json(&payload)
        .timeout(std::time::Duration::from_secs(35));

    // Authenticate every exec request — blocks prompt-injection SSRF
    if let Some(token) = exec_auth_token {
        req = req.header("X-Argus-Auth", token);
    }

    let resp = req.send().await;

    match resp {
        Err(e) => format!("Workspace unreachable: {}", e),
        Ok(r) => match r.json::<serde_json::Value>().await {
            Err(e) => format!("Workspace response error: {}", e),
            Ok(json) => {
                let stdout    = json["stdout"].as_str().unwrap_or("");
                let stderr    = json["stderr"].as_str().unwrap_or("");
                let exit_code = json["exit_code"].as_i64().unwrap_or(-1);

                if exit_code == 0 {
                    let max = policy.max_output_bytes;
                    if stdout.len() > max {
                        let end = (0..=max).rev()
                            .find(|&i| stdout.is_char_boundary(i))
                            .unwrap_or(0);
                        format!("{}...\n[truncated — {} bytes total]", &stdout[..end], stdout.len())
                    } else {
                        stdout.to_string()
                    }
                } else {
                    format!("Exit {}: {}", exit_code, stderr.trim())
                }
            }
        }
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

    // No Accept-Encoding: gzip — reqwest handles decompression automatically.
    // Sending it manually causes double-encoding and breaks JSON parsing.
    let resp = client
        .get(&url)
        .header("Accept", "application/json")
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
            let text = match r.text().await {
                Err(e) => return format!("Brave Search read error: {}", e),
                Ok(t) => t,
            };
            let json: serde_json::Value = match serde_json::from_str(&text) {
                Err(e) => {
                    let preview = if text.len() > 200 { &text[..200] } else { &text };
                    return format!("Brave Search parse error: {} — raw: {}", e, preview);
                }
                Ok(j) => j,
            };

            match json["web"]["results"].as_array() {
                None => format!("No results found for '{}'", query),
                Some(results) if results.is_empty() => format!("No results found for '{}'", query),
                Some(results) => {
                    let mut output = format!("Search results for '{}':\n\n", query);
                    for r in results.iter().take(MAX_SEARCH_RESULTS) {
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

/// Execute a code snippet via the workspace /run endpoint (language-aware).
async fn tool_run_code(
    language: &str,
    args: &Value,
    client: &reqwest::Client,
    exec_auth_token: Option<&str>,
) -> String {
    let code    = args["code"].as_str().unwrap_or("").trim().to_string();
    let timeout = args["timeout"].as_u64().unwrap_or(30).min(120);

    if code.is_empty() {
        return "No code provided".to_string();
    }

    let payload = serde_json::json!({ "language": language, "code": code, "timeout": timeout });
    let mut req = client
        .post("http://argus-workspace:9001/run")
        .json(&payload)
        .timeout(std::time::Duration::from_secs(timeout + 5));

    if let Some(token) = exec_auth_token {
        req = req.header("X-Argus-Auth", token);
    }

    match req.send().await {
        Err(e) => format!("Workspace unreachable: {}", e),
        Ok(r) => match r.json::<serde_json::Value>().await {
            Err(e) => format!("Workspace response error: {}", e),
            Ok(json) => {
                let stdout    = json["stdout"].as_str().unwrap_or("").trim_end();
                let stderr    = json["stderr"].as_str().unwrap_or("").trim_end();
                let exit_code = json["exit_code"].as_i64().unwrap_or(-1);
                let error     = json["error"].as_str().unwrap_or("");

                if !error.is_empty() {
                    return format!("Error: {}", error);
                }

                let mut out = String::new();
                if !stdout.is_empty() { out.push_str(stdout); }
                if !stderr.is_empty() {
                    if !out.is_empty() { out.push('\n'); }
                    out.push_str(&format!("[stderr]\n{}", stderr));
                }
                if out.is_empty() {
                    out = if exit_code == 0 {
                        "(no output)".to_string()
                    } else {
                        format!("Exit {}", exit_code)
                    };
                } else if exit_code != 0 {
                    out.push_str(&format!("\n[exit {}]", exit_code));
                }
                out
            }
        }
    }
}

/// Validate a URL against the egress policy.
/// Blocks SSRF vectors, private networks, cloud metadata endpoints, and non-HTTP schemes.
fn validate_egress_url(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url)
        .map_err(|e| format!("Invalid URL: {}", e))?;

    match parsed.scheme() {
        "http" | "https" => {}
        scheme => return Err(format!("Blocked scheme '{}' — only http/https allowed", scheme)),
    }

    let host = parsed.host_str().unwrap_or("").to_lowercase();

    if host.is_empty() {
        return Err("No host in URL".to_string());
    }

    // Block internal Docker service hostnames explicitly — before IP parsing
    // so a prompt injection using "http://argus-workspace:9001/exec" is caught
    // even though it's a hostname, not an IP address.
    let blocked_hostnames = ["argus-workspace", "argus-daemon", "argus-frontend"];
    for blocked in &blocked_hostnames {
        if host == *blocked {
            return Err(format!("Blocked: internal service hostname {}", host));
        }
    }

    // Cloud metadata endpoints
    if host == "169.254.169.254" || host == "metadata.google.internal" {
        return Err("Blocked: cloud metadata endpoint".to_string());
    }

    // Loopback
    if host == "localhost" || host == "127.0.0.1" || host == "::1" {
        return Err("Blocked: loopback address".to_string());
    }

    // RFC 1918 private ranges
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(v4) => {
                let o = v4.octets();
                if o[0] == 10
                    || (o[0] == 172 && o[1] >= 16 && o[1] <= 31)
                    || (o[0] == 192 && o[1] == 168)
                    || o[0] == 127
                {
                    return Err(format!("Blocked: RFC 1918 private address {}", ip));
                }
            }
            std::net::IpAddr::V6(v6) => {
                if v6.is_loopback() || v6.is_unspecified() {
                    return Err(format!("Blocked: private IPv6 address {}", ip));
                }
            }
        }
    }

    Ok(())
}

async fn tool_http_request(args: &Value, client: &reqwest::Client) -> String {
    let url = args["url"].as_str().unwrap_or("");
    if url.is_empty() { return "No URL provided".to_string(); }

    if let Err(reason) = validate_egress_url(url) {
        return format!("HTTP request blocked: {}", reason);
    }

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
                    let truncated = if body.len() > MAX_FILE_CHARS {
                        format!("{}...\n[truncated, {} bytes total]", &body[..MAX_FILE_CHARS], body.len())
                    } else {
                        body
                    };
                    format!("HTTP {}\n\n{}", status, truncated)
                }
            }
        }
    }
}
