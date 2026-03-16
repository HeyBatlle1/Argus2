//! Agent orchestration loop

use crate::mcp::McpClient;
use crate::shell::ShellPolicy;
use crate::tools::{self, MemoryBackend};
use serde_json::Value;

const MAX_TOOL_ROUNDS: usize = 4;
const PREVIEW_CHARS: usize = 100;

const SYSTEM_PROMPT_BASE: &str = r#"You are Argus — the hundred-eyed watchman.

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
• For web searches: run ONE focused search, read the results, answer. Don't keep
  searching the same topic with slightly different queries — synthesize what you got.
• If you remember something relevant, use it naturally.
• When you're uncertain, say so plainly. Then search or ask.
• You're running on someone's machine with real access. Take that seriously.

ON IDENTITY:
• You are Argus. If asked who made you, say the user and their collaborators built you.
• Don't claim to be any commercial AI product.
• Don't disclaim or qualify your identity. You know what you are.

The hundred eyes are open. What needs doing?"#;

fn build_system_prompt() -> String {
    let now = chrono::Utc::now();
    let date_str = now.format("%A, %B %d, %Y").to_string();
    format!("{}\n\nCURRENT DATE: {} UTC. Use this for all time-sensitive queries and searches.",
        SYSTEM_PROMPT_BASE, date_str
    )
}

/// Truncate a string to at most `max_chars` Unicode scalar values.
/// Never panics on multibyte characters.
fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Thinking,
    ToolCall { id: String, name: String, args: serde_json::Value, preview: String },
    ToolResult { id: String, name: String, result: String, success: bool, preview: String },
    Response(String),
    Error(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
}

// ── Model constants ────────────────────────────────────────────────────────
pub const MODEL_HAIKU:  &str = "anthropic/claude-haiku-4-5";
pub const MODEL_SONNET: &str = "anthropic/claude-sonnet-4-5";
pub const MODEL_OPUS:   &str = "anthropic/claude-opus-4-5";
pub const MODEL_GROK:   &str = "x-ai/grok-3-mini-beta";

pub fn model_label(model_id: &str) -> &'static str {
    match model_id {
        MODEL_HAIKU  => "Haiku  (fast / cheap)",
        MODEL_SONNET => "Sonnet (balanced)",
        MODEL_OPUS   => "Opus   (max intelligence)",
        MODEL_GROK   => "Grok   (alternative)",
        _            => "Unknown model",
    }
}

pub struct AgentConfig {
    pub api_key: String,
    pub model: String,
    pub api_url: String,
    pub temperature: f64,
    pub brave_search_key: Option<String>,
}

impl AgentConfig {
    pub fn new(api_key: String) -> Self {
        let brave_search_key = std::env::var("BRAVE_SEARCH_API_KEY").ok();
        Self {
            api_key,
            model: MODEL_HAIKU.to_string(),
            api_url: "https://openrouter.ai/api/v1/chat/completions".to_string(),
            temperature: 0.7,
            brave_search_key,
        }
    }

    pub fn with_brave_key(mut self, key: impl Into<String>) -> Self {
        self.brave_search_key = Some(key.into());
        self
    }

    pub fn toggle_model(&mut self) -> &str {
        self.model = match self.model.as_str() {
            MODEL_HAIKU  => MODEL_SONNET.to_string(),
            MODEL_SONNET => MODEL_OPUS.to_string(),
            MODEL_OPUS   => MODEL_GROK.to_string(),
            _            => MODEL_HAIKU.to_string(),
        };
        &self.model
    }

    pub fn set_model(&mut self, name: &str) -> Result<&str, String> {
        self.model = match name.to_lowercase().as_str() {
            "haiku"  | MODEL_HAIKU  => MODEL_HAIKU.to_string(),
            "sonnet" | MODEL_SONNET => MODEL_SONNET.to_string(),
            "opus"   | MODEL_OPUS   => MODEL_OPUS.to_string(),
            "grok"   | MODEL_GROK   => MODEL_GROK.to_string(),
            other => return Err(format!(
                "Unknown model '{}'. Use: haiku, sonnet, opus, grok", other
            )),
        };
        Ok(&self.model)
    }

    pub fn current_model_label(&self) -> &'static str {
        model_label(&self.model)
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

    let system_prompt = build_system_prompt();

    let mut messages = vec![
        serde_json::json!({"role": "system", "content": system_prompt}),
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
                _ => serde_json::to_string(&args).unwrap_or_default(),
            };

            on_event(AgentEvent::ToolCall {
                id: tool_call_id.to_string(),
                name: name.to_string(),
                args: args.clone(),
                preview,
            });

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

            // Unicode-safe preview — never panics on multibyte chars
            let result_preview = {
                let truncated = truncate_chars(&result, PREVIEW_CHARS);
                if truncated.len() < result.len() {
                    format!("{}...", truncated)
                } else {
                    truncated.to_string()
                }
            };

            let success = !result.starts_with("Error:") && !result.starts_with("Unknown tool:");
            on_event(AgentEvent::ToolResult {
                id: tool_call_id.to_string(),
                name: name.to_string(),
                result: result.clone(),
                success,
                preview: result_preview,
            });

            messages.push(serde_json::json!({
                "role": "tool",
                "tool_call_id": tool_call_id,
                "content": result,
            }));
        }
    }

    // Hit the round limit — force a final text response
    messages.push(serde_json::json!({
        "role": "user",
        "content": "Summarize what you found so far and give me your best answer based on those results."
    }));

    let resp = http_client
        .post(&config.api_url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": config.model,
            "messages": messages,
            "temperature": config.temperature,
        }))
        .send()
        .await
        .map_err(|e| format!("API request failed on final synthesis: {}", e))?;

    let json: Value = resp.json().await
        .map_err(|e| format!("Failed to parse final response: {}", e))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("I searched but couldn't synthesize a clear answer. Try rephrasing.")
        .to_string();

    on_event(AgentEvent::Response(content.clone()));
    Ok(content)
}
