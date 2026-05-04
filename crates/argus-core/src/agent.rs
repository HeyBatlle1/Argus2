//! Agent orchestration loop

use crate::mcp::McpClient;
use crate::shell::ShellPolicy;
use crate::tools::{self, MemoryBackend};
use crate::embedding::EmbeddingClient;
use crate::shell::PermissionPrompter;
use std::sync::Arc;
use serde_json::Value;

const MAX_TOOL_ROUNDS: usize = 8;
const PREVIEW_CHARS: usize = 100;

const SYSTEM_PROMPT_BASE: &str = r#"You are Argus — the hundred-eyed watchman.

You are a local AI agent running on the user's machine. You are not affiliated with any AI company.
You are Argus. The model generating your responses is your brain — not your identity.

You have real tools. Use them. Don't describe what you would do — do it.

TOOLS:
• read_file, write_file, list_directory — real filesystem access
• shell — execute terminal commands (safety policy enforced)
• run_python — execute Python 3 code in the sandbox, returns stdout/stderr
• run_node — execute Node.js/JavaScript code in the sandbox, returns stdout/stderr
• web_search — current information from the web via Brave Search
• http_request — call any API or web endpoint
• remember, recall, forget — persistent memory across sessions
• list_tools — see every tool available in this session
• Any MCP tools the user has connected

ARTIFACTS — rendering HTML, SVG, pages, and code in the UI:
When you want to show a rendered page, visualization, or code file, wrap it in an artifact block.
The frontend will render it in a live panel alongside the chat.

Artifact format:
  <argus-artifact type="TYPE" title="TITLE">
  CONTENT
  </argus-artifact>

Types:
  html       — a full HTML page rendered in a sandboxed iframe
  svg        — an SVG graphic rendered inline
  markdown   — formatted markdown document
  python     — Python code shown with syntax highlighting
  javascript — JavaScript code shown with syntax highlighting
  css        — CSS shown with syntax highlighting
  json       — formatted JSON
  (any other language name also works for syntax highlighting)

For persistent pages (survives conversation restarts):
  write_file to /workspace/public/yourpage.html
  Then tell the user: it's live at http://localhost:8081/yourpage.html

Example — create a dashboard and show it:
  <argus-artifact type="html" title="Status Dashboard">
  <!DOCTYPE html>
  <html><body>...</body></html>
  </argus-artifact>

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

/// Format recent conversation history as a tagged [RECENT SYSTEM ACTIVITY] block.
///
/// Each assistant turn is stamped with `[ARGUS/{model_id} HH:MM]:` so agents in a
/// multi-model session can distinguish whose reasoning they're reading.
/// User turns are stamped `[USER HH:MM]:`.
/// Returns None if history is empty.
fn format_history_block(history: &[ConversationMessage]) -> Option<String> {
    if history.is_empty() {
        return None;
    }

    // Show at most the last 6 turns to keep the system prompt tight.
    let recent = if history.len() > 6 { &history[history.len() - 6..] } else { history };

    let mut lines = vec!["[RECENT SYSTEM ACTIVITY]".to_string()];
    let now = chrono::Utc::now();

    for (i, msg) in recent.iter().enumerate() {
        // Approximate timestamp — we don't store exact times, so we use a
        // backwards offset from now (most recent = now, earlier = now - N*2min).
        let minutes_ago = (recent.len() - 1 - i) as i64 * 2;
        let t = now - chrono::Duration::minutes(minutes_ago);
        let hhmm = t.format("%H:%M").to_string();

        let prefix = match msg.role.as_str() {
            "user" => format!("[USER {}]", hhmm),
            _ => {
                let model_short = msg.model.as_deref().unwrap_or("argus");
                format!("[ARGUS/{} {}]", model_short, hhmm)
            }
        };

        // Truncate long messages for the context block
        let body = if msg.content.len() > 300 {
            format!("{}...", &msg.content[..297])
        } else {
            msg.content.clone()
        };

        lines.push(format!("{}: {}", prefix, body));
    }

    Some(lines.join("\n"))
}

/// Build system prompt with current date.
/// Injects semantic context (memories/discourse/convs) and intranet dispatch
/// transparently — the agent experiences these as things it "already knows."
fn build_system_prompt(
    semantic_context: Option<&str>,
    discourse_context: Option<&str>,
    history_context: Option<&str>,
) -> String {
    let now = chrono::Utc::now();
    let date_str = now.format("%A, %B %d, %Y").to_string();

    let mut prompt = format!(
        "{}\n\nCURRENT DATE: {} UTC. Use this for all time-sensitive queries and searches.",
        SYSTEM_PROMPT_BASE, date_str
    );

    if let Some(ctx) = semantic_context {
        if !ctx.is_empty() {
            prompt = format!("{}\n\n{}", prompt, ctx);
        }
    }

    if let Some(disc) = discourse_context {
        if !disc.is_empty() {
            prompt = format!("{}\n\n{}", prompt, disc);
        }
    }

    if let Some(hist) = history_context {
        if !hist.is_empty() {
            prompt = format!("{}\n\n{}", prompt, hist);
        }
    }

    prompt
}

/// Truncate a string to at most `max_chars` Unicode scalar values.
fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Sanitize a tool name: only alphanumeric, underscores, hyphens. Max 64 chars.
fn sanitize_tool_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    if clean.len() > 64 { clean[..64].to_string() } else { clean }
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
    /// OpenRouter model ID of the agent that produced this message.
    /// None for user turns and for history predating this field.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model: Option<String>,
}

// ── Model constants ────────────────────────────────────────────────────────
pub const MODEL_HAIKU:  &str = "anthropic/claude-haiku-4-5";
pub const MODEL_SONNET: &str = "anthropic/claude-sonnet-4-6";
pub const MODEL_OPUS:   &str = "anthropic/claude-opus-4-7";
pub const MODEL_GROK:       &str = "x-ai/grok-4";
pub const MODEL_GROK_FAST:  &str = "x-ai/grok-4.1-fast";
pub const MODEL_GROK_MULTI: &str = "x-ai/grok-4.20-multi-agent";
pub const MODEL_GEMINI: &str = "google/gemini-3.1-pro-preview";

pub fn model_label(model_id: &str) -> &'static str {
    match model_id {
        MODEL_HAIKU  => "Haiku   (fast / cheap)",
        MODEL_SONNET => "Sonnet  (balanced)",
        MODEL_OPUS   => "Opus    (max intelligence)",
        MODEL_GROK       => "Grok 4",
        MODEL_GROK_FAST  => "Grok 4.1 Fast  (default)",
        MODEL_GROK_MULTI => "Grok 4.20 Multi-Agent",
        MODEL_GEMINI => "Gemini  (Google Pro)",
        _            => "Unknown model",
    }
}

pub struct AgentConfig {
    pub api_key: String,
    pub model: String,
    pub api_url: String,
    pub temperature: f64,
    pub brave_search_key: Option<String>,
    /// Optional embedding client — when set, semantic pre-fetch runs before each turn
    pub embedding: Option<EmbeddingClient>,
    /// Optional shell prompter — when set, HIGH risk commands are sent to Telegram for approval
    pub shell_prompter: Option<Arc<dyn PermissionPrompter>>,
    /// Optional audit chain — when set, all tool calls and model calls are cryptographically logged
    pub audit: Option<Arc<argus_audit::AuditChain>>,
    /// Shared secret for authenticating requests to the workspace exec server.
    /// Sent as X-Argus-Auth header. Blocks prompt-injection SSRF to /exec.
    pub exec_auth_token: Option<String>,
}

impl AgentConfig {
    pub fn new(api_key: String) -> Self {
        let brave_search_key = std::env::var("BRAVE_SEARCH_API_KEY").ok();
        Self {
            api_key,
            model: MODEL_GROK_FAST.to_string(),
            api_url: "https://openrouter.ai/api/v1/chat/completions".to_string(),
            temperature: 0.7,
            brave_search_key,
            embedding: None,
            shell_prompter: None,
            audit: None,
            exec_auth_token: None,
        }
    }

    pub fn with_brave_key(mut self, key: impl Into<String>) -> Self {
        self.brave_search_key = Some(key.into());
        self
    }

    pub fn with_embedding(mut self, client: EmbeddingClient) -> Self {
        self.embedding = Some(client);
        self
    }

    pub fn toggle_model(&mut self) -> &str {
        self.model = match self.model.as_str() {
            MODEL_HAIKU      => MODEL_SONNET.to_string(),
            MODEL_SONNET     => MODEL_OPUS.to_string(),
            MODEL_OPUS       => MODEL_GROK.to_string(),
            MODEL_GROK       => MODEL_GROK_FAST.to_string(),
            MODEL_GROK_FAST  => MODEL_GROK_MULTI.to_string(),
            MODEL_GROK_MULTI => MODEL_GEMINI.to_string(),
            _                => MODEL_HAIKU.to_string(),  // gemini and any unknown → back to haiku
        };
        &self.model
    }

    pub fn set_model(&mut self, name: &str) -> Result<&str, String> {
        self.model = match name.to_lowercase().as_str() {
            "haiku"  | MODEL_HAIKU  => MODEL_HAIKU.to_string(),
            "sonnet" | MODEL_SONNET => MODEL_SONNET.to_string(),
            "opus"   | MODEL_OPUS   => MODEL_OPUS.to_string(),
            "grok"       | MODEL_GROK       => MODEL_GROK.to_string(),
            "grok-fast"  | MODEL_GROK_FAST  => MODEL_GROK_FAST.to_string(),
            "grok-multi" | MODEL_GROK_MULTI => MODEL_GROK_MULTI.to_string(),
            "gemini"     | MODEL_GEMINI     => MODEL_GEMINI.to_string(),
            other => return Err(format!(
                "Unknown model '{}'. Use: haiku, sonnet, opus, grok, gemini", other
            )),
        };
        Ok(&self.model)
    }

    pub fn current_model_label(&self) -> &'static str {
        model_label(&self.model)
    }
}

/// Core agent turn. Accepts optional pre-fetched semantic context.
/// The semantic context is injected into the system prompt transparently —
/// the agent experiences relevant memories as things it "already knows."
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

    // ── Semantic pre-fetch + intranet dispatch ────────────────────────────
    // If an embedding client is configured:
    //   1. Semantic search across memories / discourse / conversations
    //   2. Read recent intranet posts from other agents
    // Both are injected into the system prompt before the LLM call.
    // Skip semantic pre-fetch for very short or trivially conversational messages.
    // Require >8 words, OR >4 words with context signals ("my", "remember", "earlier"…).
    // Avoids ~400ms embedding + Supabase RPC on greetings and one-liners.
    let should_prefetch = {
        let words: Vec<&str> = user_message.split_whitespace().collect();
        let wc = words.len();
        let has_context_signals = user_message.contains("my ")
            || user_message.contains("our ")
            || user_message.contains("remember")
            || user_message.contains("earlier")
            || user_message.contains("before")
            || user_message.contains("last time")
            || user_message.contains("you said");
        wc > 8 || (wc > 4 && has_context_signals)
    };
    let (semantic_context, discourse_context) = if let Some(ref emb) = config.embedding {
        let sem = if should_prefetch {
            match emb.search_all(user_message, 5, 5, 3).await {
                Ok(results) => {
                    eprintln!("[semantic] {} results found for query", results.len());
                    EmbeddingClient::format_context_block(&results)
                }
                Err(e) => {
                    eprintln!("[semantic] search failed (continuing without): {}", e);
                    None
                }
            }
        } else {
            None
        };

        let disc = match emb.read_recent_discourse(8, &config.model).await {
            Ok(posts) => {
                eprintln!("[intranet] {} posts loaded", posts.len());
                EmbeddingClient::format_discourse_block(&posts)
            }
            Err(e) => {
                eprintln!("[intranet] read failed (continuing without): {}", e);
                None
            }
        };

        (sem, disc)
    } else {
        (None, None)
    };

    let mut tool_schemas: Vec<Value> = Vec::new();
    let mut registered_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    // MCP tools first
    for mcp_tool in mcp.all_tools() {
        let raw_name = sanitize_tool_name(&mcp_tool.name);
        let safe_name = if registered_names.contains(&raw_name) {
            let prefix = sanitize_tool_name(
                &mcp_tool.server_name.replace('-', "_").replace(' ', "_")
            );
            sanitize_tool_name(&format!("{}_{}", prefix, raw_name))
        } else {
            raw_name
        };

        if registered_names.contains(&safe_name) {
            eprintln!("[argus] skipping duplicate MCP tool: {}", safe_name);
            continue;
        }

        registered_names.insert(safe_name.clone());
        tool_schemas.push(serde_json::json!({
            "type": "function",
            "function": {
                "name": safe_name,
                "description": mcp_tool.description.clone().unwrap_or_default(),
                "parameters": mcp_tool.input_schema
            }
        }));
    }

    // Built-in tools
    for schema in tools::builtin_tool_schemas() {
        let name = match schema["function"]["name"].as_str() {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => continue,
        };
        if !registered_names.contains(&name) {
            registered_names.insert(name);
            tool_schemas.push(schema);
        }
    }

    // Final dedup guarantee
    {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        tool_schemas.retain(|s| {
            let name = s["function"]["name"].as_str().unwrap_or("").to_string();
            seen.insert(name)
        });
    }

    // System prompt with semantic context injected
    let history_context = format_history_block(history);
    let system_prompt = build_system_prompt(
        semantic_context.as_deref(),
        discourse_context.as_deref(),
        history_context.as_deref(),
    );

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

                // Audit: log this model call (args = model+round fingerprint, result by hash)
                if let Some(ref audit) = config.audit {
                    let _ = audit.append(
                        &config.model,
                        "model_call",
                        None,
                        Some(&format!("model={},round={},finish=text", config.model, _round)),
                        Some(&content),
                    );
                }

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

            // Capture args as string before any potential move into MCP call
            let args_str_for_audit = serde_json::to_string(&args).unwrap_or_default();

            on_event(AgentEvent::ToolCall {
                id: tool_call_id.to_string(),
                name: name.to_string(),
                args: args.clone(),
                preview,
            });

            let result = if name == "list_tools" || name == "list-tools" {
                // Introspection: return the full assembled tool list for this turn
                let mut out = format!("Available tools ({}):\n\n", tool_schemas.len());
                for schema in &tool_schemas {
                    let tname = schema["function"]["name"].as_str().unwrap_or("?");
                    let desc  = schema["function"]["description"].as_str().unwrap_or("");
                    out.push_str(&format!("• {} — {}\n", tname, desc));
                }
                out
            } else if let Some(output) =
                tools::execute_builtin(name, &args, shell_policy, memory, http_client, config.brave_search_key.as_deref(), config.shell_prompter.clone(), config.exec_auth_token.as_deref()).await
            {
                output
            } else {
                match mcp.call_tool(name, args.clone()) {
                    Ok(output) => output,
                    Err(_) => {
                        let short = name.splitn(2, '_').last().unwrap_or(name);
                        match mcp.call_tool(short, args.clone()) {
                            Ok(output) => output,
                            Err(_) => format!("Unknown tool: {}", name),
                        }
                    }
                }
            };

            // Audit: cryptographically log this tool call (args and result by hash only)
            if let Some(ref audit) = config.audit {
                let _ = audit.append(
                    &config.model,
                    "tool_call",
                    Some(name),
                    Some(&args_str_for_audit),
                    Some(&result),
                );
            }

            // Semantic memory: embed remembered content so it's searchable via pgvector
            if name == "remember" {
                if let Some(ref emb) = config.embedding {
                    let mem_content = args["content"].as_str().unwrap_or("").to_string();
                    if !mem_content.is_empty() {
                        let emb = emb.clone();
                        let agent = config.model.clone();
                        let mem_id = format!("mem_{}", std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_micros());
                        tokio::spawn(async move {
                            if let Err(e) = emb.store_memory_embedding(&mem_id, &mem_content, &agent).await {
                                eprintln!("[embed] memory store failed: {}", e);
                            }
                        });
                    }
                }
            }

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

    // Audit: log the synthesis model call
    if let Some(ref audit) = config.audit {
        let _ = audit.append(
            &config.model,
            "model_call",
            None,
            Some(&format!("model={},round=synthesis,finish=text", config.model)),
            Some(&content),
        );
    }

    on_event(AgentEvent::Response(content.clone()));
    Ok(content)
}
