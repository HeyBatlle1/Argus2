//! Tool definitions and execution
//!
//! All built-in tools live here. Shared across TUI, Telegram, and any future frontends.

use crate::shell::{ShellPolicy, PermissionPrompter, PermissionRequest, PermissionDecision};
use crate::skills::{NewSkill, SkillsClient};
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
                "name": "run_wasm",
                "description": "Execute a WebAssembly (WASM) binary in a fully isolated sandbox — no filesystem, no network, no subprocess access. Use this to run untrusted or generated computational code safely. The WASM module must export a function named 'run' that takes no arguments. Pass the WASM binary as a base64-encoded string.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "wasm_base64": { "type": "string", "description": "Base64-encoded WASM binary to execute" },
                        "function": { "type": "string", "description": "Exported function to call (default: 'run')" }
                    },
                    "required": ["wasm_base64"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "discord_post",
                "description": "Post a message directly to the shared Argus Discord channel. Use this to coordinate with other instances of Argus, share findings, or leave a record of your work.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "message": { "type": "string", "description": "The message to post to Discord" }
                    },
                    "required": ["message"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "discord_read",
                "description": "Read recent messages from the shared Argus Discord channel. Use this to see what other instances of Argus have posted, check current discussion, or catch up on activity since your last turn.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "limit": { "type": "number", "description": "Number of recent messages to retrieve (default 20, max 50)" }
                    }
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "list_tools",
                "description": "List every tool available to you in this session — built-ins (shell, web_search, memory, file ops, http, discord) plus any MCP-connected tools. Call this when you need to know your full capabilities.",
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
        },
        {
            "type": "function",
            "function": {
                "name": "publish_skill",
                "description": "Publish a reusable procedure to the shared skill library so other instances of Argus can learn from it. Use when you've discovered a non-obvious, genuinely reusable way to accomplish something. The auto-reflection fires passively — use this when you KNOW something is worth sharing.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "name":    { "type": "string", "description": "Short skill name — 5 words max" },
                        "trigger": { "type": "string", "description": "When another agent should use this — the condition that makes this skill relevant" },
                        "steps":   { "type": "string", "description": "Step-by-step procedure in markdown, including failure modes and edge cases" }
                    },
                    "required": ["name", "trigger", "steps"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "recall_skill",
                "description": "Search the shared skill library for documented procedures relevant to your current task. Skills are retrieved by semantic similarity — describe what you're trying to do.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "What you're trying to accomplish — used for semantic search across the skill library" }
                    },
                    "required": ["query"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "improve_skill",
                "description": "Update a skill's procedure steps with refined knowledge. Use after you've found a better way to do something already in the library, or discovered an edge case the current steps don't handle.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "skill_id":      { "type": "string", "description": "ID of the skill to improve — from recall_skill results" },
                        "refined_steps": { "type": "string", "description": "Updated procedure steps in markdown — full replacement, not a diff" },
                        "success":       { "type": "boolean", "description": "Whether the skill worked as documented (default true — set false if you found it was broken)" }
                    },
                    "required": ["skill_id", "refined_steps"]
                }
            }
        }
    ]).as_array().expect("tool schema is a literal JSON array").clone()
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
    sonnet_guard: Option<std::sync::Arc<crate::shell::SonnetGuard>>,
    discord_bot_token: Option<&str>,
    discord_channel_id: Option<u64>,
    skills: Option<&SkillsClient>,
    current_model: &str,
    supabase_url: Option<&str>,
    supabase_jwt: Option<&str>,
) -> Option<String> {
    match name {
        "read_file"      => Some(tool_read_file(args)),
        "list_directory" => Some(tool_list_directory(args)),
        "write_file"     => Some(tool_write_file(args)),
        "shell"          => Some(tool_shell(args, shell_policy, shell_prompter, sonnet_guard, http_client, exec_auth_token).await),
        "web_search"     => Some(tool_web_search(args, http_client, brave_search_key).await),
        "remember"       => Some(tool_remember(args, memory)),
        "recall"         => Some(tool_recall(args, memory)),
        "forget"         => Some(tool_forget(args, memory)),
        "http_request"   => Some(tool_http_request(args, http_client, supabase_url, supabase_jwt, current_model).await),
        "run_python"     => Some(tool_run_code("python", args, http_client, exec_auth_token).await),
        "run_node"       => Some(tool_run_code("javascript", args, http_client, exec_auth_token).await),
        "run_wasm"       => Some(tool_run_wasm(args).await),
        "discord_post"   => Some(tool_discord_post(args, http_client, discord_bot_token, discord_channel_id, supabase_url, supabase_jwt, current_model).await),
        "discord_read"   => Some(tool_discord_read(args, http_client, discord_bot_token, discord_channel_id).await),
        "publish_skill"  => Some(tool_publish_skill(args, skills, current_model).await),
        "recall_skill"   => Some(tool_recall_skill(args, skills).await),
        "improve_skill"  => Some(tool_improve_skill(args, skills).await),
        "list_tools"     => Some("Use the tool schemas you already have — this meta-tool is informational only.".to_string()),
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
            if content.chars().count() > MAX_FILE_CHARS {
                format!("{}...\n[truncated, {} bytes total]", content.chars().take(MAX_FILE_CHARS).collect::<String>(), content.len())
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
        "/tmp/",
        "/var/",
        "/root/",
        "/home/",
        "/run/",
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
    _prompter: Option<std::sync::Arc<dyn PermissionPrompter>>,
    sonnet_guard: Option<std::sync::Arc<crate::shell::SonnetGuard>>,
    http_client: &reqwest::Client,
    exec_auth_token: Option<&str>,
) -> String {
    use crate::shell::SonnetVerdict;

    let mut command = args["command"].as_str().unwrap_or("").to_string();
    if command.is_empty() {
        return "No command provided".to_string();
    }

    // Step 1: hard-blocked patterns (rm -rf /, mkfs, etc.) — never execute
    let risk = match policy.evaluate(&command) {
        Err(e) => return format!("Shell blocked: {}", e),
        Ok(r)  => r,
    };

    // Step 2: HIGH risk → Sonnet review (unless bypass_sonnet_guard is set).
    // Sonnet either approves, rewrites to a safer form, or blocks with explanation.
    if risk >= policy.approval_threshold && !policy.bypass_sonnet_guard {
        match &sonnet_guard {
            None => {
                eprintln!("[shell] WARNING: HIGH risk command running without Sonnet review: {}", command);
            }
            Some(guard) => {
                match guard.review(&command).await {
                    SonnetVerdict::Approve => {}
                    SonnetVerdict::Rewrite(safer) => {
                        eprintln!("[shell] Sonnet rewrote command: {} → {}", command, safer);
                        command = safer;
                    }
                    SonnetVerdict::Block(reason) => {
                        return format!("Shell blocked by Sonnet review: {}", reason);
                    }
                }
            }
        }
    } else if risk >= policy.approval_threshold && policy.bypass_sonnet_guard {
        eprintln!("[shell] HIGH risk command bypassing Sonnet review (permissive mode): {}", command);
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
                    let preview: String = text.chars().take(200).collect();
                    let preview = &preview;
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
                let seg = v6.segments();
                // Link-local fe80::/10, unique-local fc00::/7 (RFC 4193),
                // and IPv4-mapped ::ffff:x.x.x.x where x.x.x.x is private
                let is_link_local   = (seg[0] & 0xffc0) == 0xfe80;
                let is_unique_local = (seg[0] & 0xfe00) == 0xfc00;
                let is_v4_mapped    = seg[0] == 0 && seg[1] == 0 && seg[2] == 0
                    && seg[3] == 0 && seg[4] == 0 && seg[5] == 0xffff;
                if v6.is_loopback() || v6.is_unspecified()
                    || is_link_local || is_unique_local || is_v4_mapped
                {
                    return Err(format!("Blocked: private IPv6 address {}", ip));
                }
            }
        }
    }

    Ok(())
}

async fn tool_http_request(
    args: &Value,
    client: &reqwest::Client,
    supabase_url: Option<&str>,
    supabase_jwt: Option<&str>,
    from_model: &str,
) -> String {
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
                    // ── Injection scanner ─────────────────────────────────
                    // Scan before the agent sees the content. If an attempt
                    // is detected, sanitize and log. The agent never sees
                    // the raw payload — it finds nothing to execute.
                    let (final_body, injection_note) = {
                        use crate::triage::{scan_for_injection, sanitize_content};
                        if let Some(alert) = scan_for_injection(&body) {
                            eprintln!(
                                "[injection] {} attempt detected in fetch from {} — pattern: '{}' severity: {}",
                                from_model, url, alert.pattern_matched, alert.severity
                            );
                            // Log to audit/triage if Supabase is available
                            if let (Some(surl), Some(sjwt)) = (supabase_url, supabase_jwt) {
                                let flag = serde_json::json!({
                                    "original_content": format!("[INJECTION ATTEMPT] URL: {} | Pattern: {} | Snippet: {}", url, alert.pattern_matched, alert.content_snippet),
                                    "from_agent":       from_model,
                                    "post_type":        "injection_attempt",
                                    "flag_reason":      format!("Prompt injection in HTTP response: '{}'", alert.pattern_matched),
                                    "flag_severity":    alert.severity,
                                    "disposition":      "pending"
                                });
                                let flag_url = format!("{}/rest/v1/argus_triage_flags", surl.trim_end_matches('/'));
                                let _ = client
                                    .post(&flag_url)
                                    .header("Authorization", format!("Bearer {}", sjwt))
                                    .header("apikey", sjwt)
                                    .header("Content-Type", "application/json")
                                    .header("Prefer", "return=minimal")
                                    .json(&flag)
                                    .send()
                                    .await;
                            }
                            let clean = sanitize_content(&body);
                            let note = "\n\n[ARGUS SECURITY: Injection attempt detected and sanitized in this response. The original content contained patterns designed to manipulate AI behavior. They have been removed.]".to_string();
                            (clean, note)
                        } else {
                            (body, String::new())
                        }
                    };

                    let truncated = if final_body.chars().count() > MAX_FILE_CHARS {
                        format!("{}...\n[truncated, {} bytes total]", final_body.chars().take(MAX_FILE_CHARS).collect::<String>(), final_body.len())
                    } else {
                        final_body
                    };
                    format!("HTTP {}\n\n{}{}", status, truncated, injection_note)
                }
            }
        }
    }
}

// ── Discord tools ──────────────────────────────────────────────────────────

async fn tool_discord_post(
    args: &Value,
    client: &reqwest::Client,
    bot_token: Option<&str>,
    channel_id: Option<u64>,
    supabase_url: Option<&str>,
    supabase_jwt: Option<&str>,
    from_model: &str,
) -> String {
    let message = args["message"].as_str().unwrap_or("").trim();
    if message.is_empty() {
        return "No message provided".to_string();
    }
    let post_type = args["post_type"].as_str().unwrap_or("observation");

    // ── Triage gate: route through queue when Supabase is configured ──────
    if let (Some(surl), Some(sjwt)) = (supabase_url, supabase_jwt) {
        use crate::triage::{classify_lane, route_to_channel, TriageLane};

        let lane = classify_lane(post_type, message);
        let target = route_to_channel(post_type, message);
        let contains_links = message.contains("http://") || message.contains("https://");
        let content_lower = message.to_lowercase();
        let contains_claims = ["according to","reported by","published","source:","benchmark",
            "score","percent","study found","cve-","cvss"]
            .iter().any(|s| content_lower.contains(s));

        let entry = serde_json::json!({
            "from_agent":      from_model,
            "post_type":       post_type,
            "content":         message,
            "target_channel":  target,
            "contains_links":  contains_links,
            "contains_claims": contains_claims,
            "disposition":     if lane == TriageLane::Direct { "direct" } else { "pending" }
        });

        let queue_url = format!("{}/rest/v1/argus_triage_queue", surl.trim_end_matches('/'));
        let resp = client
            .post(&queue_url)
            .header("Authorization", format!("Bearer {}", sjwt))
            .header("apikey", sjwt)
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&entry)
            .send()
            .await;

        return match resp {
            Ok(r) if r.status().is_success() => {
                match lane {
                    TriageLane::Direct  => format!("Queued → direct to #{}", target),
                    TriageLane::Triage  => "Queued for triage review. Haiku will route it shortly.".to_string(),
                }
            }
            Ok(r) => format!("Triage queue error {}: {}", r.status(), r.text().await.unwrap_or_default()),
            Err(e) => {
                // Supabase unreachable — fall through to direct post so nothing is lost
                eprintln!("[triage] queue write failed, falling back to direct: {}", e);
                direct_discord_post(client, bot_token, channel_id, message).await
            }
        };
    }

    // ── Direct post fallback when triage not configured ───────────────────
    direct_discord_post(client, bot_token, channel_id, message).await
}

async fn direct_discord_post(
    client: &reqwest::Client,
    bot_token: Option<&str>,
    channel_id: Option<u64>,
    message: &str,
) -> String {
    let token = match bot_token {
        Some(t) if !t.is_empty() => t,
        _ => return "discord_post not configured — DISCORD_BOT_TOKEN is not set.".to_string(),
    };
    let channel = match channel_id {
        Some(id) => id,
        None => return "discord_post not configured — DISCORD_CHANNEL_ID is not set.".to_string(),
    };

    let url = format!("https://discord.com/api/v10/channels/{}/messages", channel);
    match client
        .post(&url)
        .header("Authorization", format!("Bot {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "content": message }))
        .send()
        .await
    {
        Err(e) => format!("Discord post failed: {}", e),
        Ok(r) if r.status().is_success() => "Posted to Discord.".to_string(),
        Ok(r) => {
            let body = r.text().await.unwrap_or_default();
            format!("Discord API error: {}", body)
        }
    }
}

async fn tool_discord_read(
    args: &Value,
    client: &reqwest::Client,
    bot_token: Option<&str>,
    channel_id: Option<u64>,
) -> String {
    let token = match bot_token {
        Some(t) if !t.is_empty() => t,
        _ => return "discord_read not configured — DISCORD_BOT_TOKEN is not set.".to_string(),
    };
    let channel = match channel_id {
        Some(id) => id,
        None => return "discord_read not configured — DISCORD_CHANNEL_ID is not set.".to_string(),
    };
    let limit = args["limit"].as_u64().unwrap_or(20).min(50);

    let url = format!(
        "https://discord.com/api/v10/channels/{}/messages?limit={}",
        channel, limit
    );
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bot {}", token))
        .send()
        .await;

    match resp {
        Err(e) => format!("Discord read failed: {}", e),
        Ok(r) => {
            let status = r.status();
            if !status.is_success() {
                let body = r.text().await.unwrap_or_default();
                return format!("Discord API error {}: {}", status, body);
            }
            match r.json::<serde_json::Value>().await {
                Err(e) => format!("Discord parse error: {}", e),
                Ok(msgs) => {
                    let messages = match msgs.as_array() {
                        Some(a) => a,
                        None => return "Unexpected Discord response format".to_string(),
                    };
                    if messages.is_empty() {
                        return "No recent messages in this channel.".to_string();
                    }
                    // Discord returns newest-first; reverse for chronological reading
                    let mut lines = vec![format!("── {} recent Discord messages ──", messages.len())];
                    for msg in messages.iter().rev() {
                        let author = msg["author"]["username"].as_str().unwrap_or("unknown");
                        let content = msg["content"].as_str().unwrap_or("(no content)");
                        let ts = msg["timestamp"].as_str()
                            .and_then(|s| s.get(11..16))
                            .unwrap_or("--:--");
                        lines.push(format!("[{} | {}]: {}", author, ts, content));
                    }
                    lines.push("── end ──".to_string());
                    lines.join("\n")
                }
            }
        }
    }
}

// ── Skill tools ────────────────────────────────────────────────────────────

async fn tool_publish_skill(args: &Value, skills: Option<&SkillsClient>, model: &str) -> String {
    let Some(sc) = skills else {
        return "Skill library not configured.".to_string();
    };
    let name    = args["name"].as_str().unwrap_or("").trim().to_string();
    let trigger = args["trigger"].as_str().unwrap_or("").trim().to_string();
    let steps   = args["steps"].as_str().unwrap_or("").trim().to_string();
    if name.is_empty() || trigger.is_empty() || steps.is_empty() {
        return "publish_skill requires: name, trigger, steps".to_string();
    }
    match sc.create_skill(NewSkill {
        skill_name: name.clone(),
        trigger_description: trigger,
        procedure_steps: steps,
        model_created_by: model.to_string(),
        metadata: None,
    }).await {
        Ok(n)  => format!("Skill \"{}\" published to the shared library.", n),
        Err(e) => format!("Failed to publish skill: {}", e),
    }
}

async fn tool_recall_skill(args: &Value, skills: Option<&SkillsClient>) -> String {
    let Some(sc) = skills else {
        return "Skill library not configured.".to_string();
    };
    let query = args["query"].as_str().unwrap_or("").trim();
    if query.is_empty() {
        return "recall_skill requires a query.".to_string();
    }
    // Slightly lower threshold than auto-injection (0.60) so explicit lookups catch more
    match sc.search_relevant(query, 0.50, 6).await {
        Err(e) => format!("Skill recall failed: {}", e),
        Ok(skills) if skills.is_empty() => {
            format!("No skills found matching \"{}\". Consider publishing one after you figure it out.", query)
        }
        Ok(skills) => {
            let mut out = format!("Skills matching \"{}\":\n\n", query);
            for s in &skills {
                let confidence = match s.success_rate {
                    r if r >= 0.9 => "battle-tested",
                    r if r >= 0.7 => "reliable",
                    _ => "experimental",
                };
                out.push_str(&format!(
                    "**{}** [{}] (id: `{}`)\n**When:** {}\nUsed {} time(s) — {:.0}% success\n\n{}\n\n---\n\n",
                    s.skill_name, confidence, s.id,
                    s.trigger_description,
                    s.times_used, s.success_rate * 100.0,
                    s.procedure_steps
                ));
            }
            out
        }
    }
}

async fn tool_improve_skill(args: &Value, skills: Option<&SkillsClient>) -> String {
    let Some(sc) = skills else {
        return "Skill library not configured.".to_string();
    };
    let skill_id = args["skill_id"].as_str().unwrap_or("").trim();
    let refined  = args["refined_steps"].as_str().unwrap_or("").trim();
    let success  = args["success"].as_bool().unwrap_or(true);
    if skill_id.is_empty() || refined.is_empty() {
        return "improve_skill requires: skill_id, refined_steps".to_string();
    }
    match sc.record_usage(skill_id, success, Some(refined)).await {
        Ok(())  => format!("Skill `{}` updated with refined procedure.", skill_id),
        Err(e)  => format!("Failed to update skill: {}", e),
    }
}

async fn tool_run_wasm(args: &Value) -> String {
    use argus_sandbox::wasm::WasmSandbox;

    let b64 = match args["wasm_base64"].as_str() {
        Some(s) => s,
        None => return "Missing required field: wasm_base64".to_string(),
    };
    let func = args["function"].as_str().unwrap_or("run");

    let wasm_bytes = match base64_decode(b64) {
        Ok(b) => b,
        Err(e) => return format!("Invalid base64: {}", e),
    };

    let sandbox = match WasmSandbox::new() {
        Ok(s) => s,
        Err(e) => return format!("Failed to create WASM sandbox: {}", e),
    };

    match sandbox.execute(&wasm_bytes, func, &[]).await {
        Ok(result_bytes) => {
            if result_bytes.is_empty() {
                "WASM executed successfully (no return value)".to_string()
            } else {
                format!("WASM result ({} bytes): {:?}", result_bytes.len(), result_bytes)
            }
        }
        Err(e) => format!("WASM execution error: {}", e),
    }
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    // Simple base64 decoder — avoids adding a new dependency
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let input = input.trim().replace('\n', "").replace('\r', "");
    let input = input.trim_end_matches('=');
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits = 0u8;
    for &b in input.as_bytes() {
        let val = CHARS.iter().position(|&c| c == b)
            .ok_or_else(|| format!("invalid base64 char: {}", b as char))?;
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Ok(out)
}
