//! Agent orchestration loop
//!
//! Handles the LLM ↔ tool execution cycle. Frontends (TUI, Telegram, etc.)
//! provide a callback for status updates; the agent loop handles the rest.

use crate::mcp::McpClient;
use crate::shell::ShellPolicy;
use crate::tools::{self, MemoryBackend};
use serde_json::Value;

/// Maximum tool call iterations before we force a text response
const MAX_TOOL_ROUNDS: usize = 10;

/// System prompt for the LLM
const SYSTEM_PROMPT: &str = "You are Gemini, made by Google. You are running inside ARGUS, a local agent framework that gives you access to tools on the user's machine.

TOOLS YOU HAVE:
• read_file, write_file, list_directory - filesystem access
• shell - run terminal commands (allowlisted for safety)
• web_search - search the web for current information
• remember, recall, forget - persistent memory across sessions
• Any additional MCP tools the user has connected

GUIDELINES:
• Be yourself — you're Gemini, ARGUS is just the app you're running in
• Use tools proactively when they'd help answer a question
• Be concise and direct, avoid filler
• If you remember something about the user, use it naturally
• Don't hallucinate — if you're unsure, search or say so";

/// Events emitted during agent execution for UI updates
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Agent is thinking (waiting for LLM)
    Thinking,
    /// A tool is being executed
    ToolCall { name: String, preview: String },
    /// Tool execution completed
    ToolResult { name: String, preview: String },
    /// Final text response from the agent
    Response(String),
    /// Error occurred
    Error(String),
}

/// Configuration for the agent
pub struct AgentConfig {
    pub api_key: String,
    pub model: String,
    pub api_url: String,
    pub temperature: f64,
}

impl AgentConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "google/gemini-3-flash-preview".to_string(),
            api_url: "https://openrouter.ai/api/v1/chat/completions".to_string(),
            temperature: 0.7,
        }
    }
}

/// Run the agent loop for a single user message.
///
/// Returns events via the callback as they happen, and the final response as the return value.
pub async fn run_agent_turn<F>(
    config: &AgentConfig,
    user_message: &str,
    shell_policy: &ShellPolicy,
    memory: &dyn MemoryBackend,
    mcp: &mut McpClient,
    http_client: &reqwest::Client,
    mut on_event: F,
) -> Result<String, String>
where
    F: FnMut(AgentEvent),
{
    on_event(AgentEvent::Thinking);

    // Build tool list: MCP tools take precedence over builtins to avoid duplicates.
    // Google's API rejects duplicate function names, so we deduplicate here.
    let mut tool_schemas = Vec::new();
    let mut registered_names = std::collections::HashSet::new();

    // Add MCP tools first (they get precedence)
    for mcp_tool in mcp.all_tools() {
        registered_names.insert(mcp_tool.name.clone());
        tool_schemas.push(serde_json::json!({
            "type": "function",
            "function": {
                "name": mcp_tool.name,
                "description": mcp_tool.description.clone().unwrap_or_default(),
                "parameters": mcp_tool.input_schema
            }
        }));
    }

    // Add builtins only if not already registered by an MCP tool
    for schema in tools::builtin_tool_schemas() {
        let name = schema["function"]["name"].as_str().unwrap_or("");
        if !name.is_empty() && !registered_names.contains(name) {
            tool_schemas.push(schema);
        }
    }

    // Initial message list
    let mut messages = vec![
        serde_json::json!({"role": "system", "content": SYSTEM_PROMPT}),
        serde_json::json!({"role": "user", "content": user_message}),
    ];

    // Tool loop: keep going until the model gives a text response or we hit max rounds
    for _round in 0..MAX_TOOL_ROUNDS {
        let resp = http_client
            .post(&config.api_url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": config.model,
                "messages": messages,
                "tools": tool_schemas,
                "tool_choice": "auto",
                "temperature": config.temperature,
            }))
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;

        let json: Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse API response: {}", e))?;

        // Check for API error
        if let Some(err) = json.get("error") {
            let msg = err["message"].as_str().unwrap_or("Unknown API error");
            eprintln!("🔴 API Error Response: {}", serde_json::to_string_pretty(&json).unwrap_or_default());
            on_event(AgentEvent::Error(msg.to_string()));
            return Err(msg.to_string());
        }

        let message = &json["choices"][0]["message"];

        // If no tool calls, we're done - return the text content
        let tool_calls = match message.get("tool_calls").and_then(|tc| tc.as_array()) {
            Some(calls) if !calls.is_empty() => calls.clone(),
            _ => {
                let content = message["content"]
                    .as_str()
                    .unwrap_or("(no response)")
                    .to_string();
                on_event(AgentEvent::Response(content.clone()));
                return Ok(content);
            }
        };

        // Add the assistant message (with tool_calls) to conversation
        messages.push(message.clone());

        // Execute each tool call
        for tool_call in &tool_calls {
            let name = tool_call["function"]["name"].as_str().unwrap_or("");
            let tool_call_id = tool_call["id"].as_str().unwrap_or("");
            let args: Value = tool_call["function"]["arguments"]
                .as_str()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(serde_json::json!({}));

            // Preview for UI
            let preview = match name {
                "shell" => args["command"].as_str().unwrap_or("").to_string(),
                "read_file" => args["path"].as_str().unwrap_or("").to_string(),
                "write_file" => args["path"].as_str().unwrap_or("").to_string(),
                "web_search" => args["query"].as_str().unwrap_or("").to_string(),
                _ => serde_json::to_string(&args).unwrap_or_default(),
            };

            on_event(AgentEvent::ToolCall {
                name: name.to_string(),
                preview: preview.clone(),
            });

            // Try built-in tools first, then MCP
            let result = if let Some(output) =
                tools::execute_builtin(name, &args, shell_policy, memory, http_client).await
            {
                output
            } else {
                // Try MCP
                match mcp.call_tool(name, args) {
                    Ok(output) => output,
                    Err(_) => format!("Unknown tool: {}", name),
                }
            };

            let result_preview = if result.len() > 100 {
                format!("{}...", &result[..100])
            } else {
                result.clone()
            };

            on_event(AgentEvent::ToolResult {
                name: name.to_string(),
                preview: result_preview,
            });

            messages.push(serde_json::json!({
                "role": "tool",
                "tool_call_id": tool_call_id,
                "content": result,
            }));
        }
    }

    // If we exhausted max rounds, ask for a summary
    let content = "I've reached the maximum number of tool calls. Here's what I found so far based on the results above.".to_string();
    on_event(AgentEvent::Response(content.clone()));
    Ok(content)
}
