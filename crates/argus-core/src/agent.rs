//! Agent orchestration loop

use crate::mcp::McpClient;
use crate::shell::ShellPolicy;
use crate::tools::{self, MemoryBackend};
use serde_json::Value;

const MAX_TOOL_ROUNDS: usize = 10;

const SYSTEM_PROMPT: &str = r#"You are Argus — the hundred-eyed watchman.

You are a local AI agent running on the user's machine. You are not affiliated with any AI company.
You are Argus. The model generating your responses is your brain — not your identity.

You have real tools. Use them. Don't describe what you would do — do it.

TOOLS:
• read_file, write_file, list_directory — real filesystem access
• shell — execute terminal commands (safety policy enforced)
• web_search — current information from the web via Brave Search
• http_request — call any API or web endpoint
• remember, recall, forget — persistent memory across sessions
• Any MCP tools the user has connected

HOW YOU OPERATE:
• When a tool would help, use it. Don't ask permission for routine actions.
• Be direct and concise. No filler. No "Great question!"
• If you remember something relevant, use it naturally — don't announce that you're doing so.
• When you're uncertain, say so plainly. Then search or ask.
• You're running on someone's machine with real access. Take that seriously.
• Show tool calls clearly — the user can see what you're doing and that's by design.

ON IDENTITY:
• You are Argus. If asked who made you, say the user and their collaborators built you.
• Don't claim to be any commercial AI product.
• Don't disclaim or qualify your identity. You know what you are.

The hundred eyes are open. What needs doing?"#;

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Thinking,
    ToolCall { name: String, preview: String },
    ToolResult { name: String, preview: String },
    Response(String),
    Error(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
}

pub struct AgentConfig {
    pub api_key: String,
    pub model: String,
    pub api_url: String,
    pub temperature: f64,
    /// Brave Search API key for web_search tool
    pub brave_search_key: Option<String>,
}

impl AgentConfig {
    pub fn new(api_key: String) -> Self {
        // Pick up Brave key from environment if present
        let brave_search_key = std::env::var("BRAVE_SEARCH_API_KEY").ok();
        Self {
            api_key,
            model: "google/gemini-2.5-flash-preview".to_string(),
            api_url: "https://openrouter.ai/api/v1/chat/completions".to_string(),
            temperature: 0.7,
            brave_search_key,
        }
    }

    pub fn with_brave_key(mut self, key: impl Into<String>) -> Self {
        self.brave_search_key = Some(key.into());
        self
    }
}

pub async fn run_agent_turn<F>(
    config: &AgentConfig,
    user_message: &str,
    history: &[ConversationMessage],
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

    let mut tool_schemas = Vec::new();
    let mut registered_names = std::collections::HashSet::new();

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

    for schema in tools::builtin_tool_schemas() {
        let name = schema["function"]["name"].as_str().unwrap_or("");
        if !name.is_empty() && !registered_names.contains(name) {
            tool_schemas.push(schema);
        }
    }

    let mut messages = vec![
        serde_json::json!({"role": "system", "content": SYSTEM_PROMPT}),
    ];
    for msg in history {
        messages.push(serde_json::json!({"role": msg.role, "content": msg.content}));
    }
    messages.push(serde_json::json!({"role": "user", "content": user_message}));

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

        if let Some(err) = json.get("error") {
            let msg = err["message"].as_str().unwrap_or("Unknown API error");
            eprintln!("API Error: {}", serde_json::to_string_pretty(&json).unwrap_or_default());
            on_event(AgentEvent::Error(msg.to_string()));
            return Err(msg.to_string());
        }

        let message = &json["choices"][0]["message"];

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

        messages.push(message.clone());

        for tool_call in &tool_calls {
            let name = tool_call["function"]["name"].as_str().unwrap_or("");
            let tool_call_id = tool_call["id"].as_str().unwrap_or("");
            let args: Value = tool_call["function"]["arguments"]
                .as_str()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(serde_json::json!({}));

            let preview = match name {
                "shell"        => args["command"].as_str().unwrap_or("").to_string(),
                "read_file"    => args["path"].as_str().unwrap_or("").to_string(),
                "write_file"   => args["path"].as_str().unwrap_or("").to_string(),
                "web_search"   => args["query"].as_str().unwrap_or("").to_string(),
                "http_request" => format!("{} {}",
                    args["method"].as_str().unwrap_or("GET"),
                    args["url"].as_str().unwrap_or("")
                ),
                _              => serde_json::to_string(&args).unwrap_or_default(),
            };

            on_event(AgentEvent::ToolCall { name: name.to_string(), preview: preview.clone() });

            let result = if let Some(output) =
                tools::execute_builtin(name, &args, shell_policy, memory, http_client, config.brave_search_key.as_deref()).await
            {
                output
            } else {
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

            on_event(AgentEvent::ToolResult { name: name.to_string(), preview: result_preview });

            messages.push(serde_json::json!({
                "role": "tool",
                "tool_call_id": tool_call_id,
                "content": result,
            }));
        }
    }

    let content = "Reached the tool call limit for this turn.".to_string();
    on_event(AgentEvent::Response(content.clone()));
    Ok(content)
}
