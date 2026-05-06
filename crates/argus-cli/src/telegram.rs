//! Telegram Bot for Argus

use teloxide::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use argus_memory::sqlite::SqliteMemory;
use argus_core::{AgentConfig, AgentEvent, ConversationMessage, ShellPolicy, MODEL_HAIKU, MODEL_SONNET, MODEL_OPUS, MODEL_GROK, MODEL_GEMINI, model_label};

/// Per-chat rate limit: max N messages per window.
const RATE_LIMIT_MAX: u32 = 10;
const RATE_LIMIT_WINDOW_SECS: u64 = 60;

struct ArgusBot {
    config: AgentConfig,
    client: reqwest::Client,
    memory: SqliteMemory,
    mcp: argus_core::mcp::McpClient,
    shell_policy: ShellPolicy,
    /// Rate limiter: chat_id → (message_count, window_start)
    rate_limits: HashMap<i64, (u32, Instant)>,
}

impl ArgusBot {
    fn new(config: AgentConfig) -> Self {
        let mut mcp = argus_core::mcp::McpClient::new();
        let _ = mcp.connect_all();

        Self {
            config,
            client: reqwest::Client::new(),
            memory: SqliteMemory::open_default().expect("failed to open memory db"),
            mcp,
            shell_policy: ShellPolicy::default(),
            rate_limits: HashMap::new(),
        }
    }

    /// Check if a chat_id is within the rate limit.
    /// Returns true if the message should be processed, false if it should be dropped.
    fn check_rate_limit(&mut self, chat_id: i64) -> bool {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(RATE_LIMIT_WINDOW_SECS);

        let entry = self.rate_limits.entry(chat_id).or_insert((0, now));
        if now.duration_since(entry.1) >= window {
            // Window expired — reset
            *entry = (1, now);
            true
        } else if entry.0 < RATE_LIMIT_MAX {
            entry.0 += 1;
            true
        } else {
            false
        }
    }

    fn handle_command(&mut self, text: &str) -> Option<String> {
        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        match parts[0] {
            "/model" => {
                if parts.len() == 1 {
                    Some(format!(
                        "Current model: {} ({})\n\nAvailable:\n  haiku  — {}\n  sonnet — {}\n  opus   — {}\n  grok   — {}\n  gemini — {}\n\nSwitch with /model <name>",
                        model_label(&self.config.model), self.config.model,
                        MODEL_HAIKU, MODEL_SONNET, MODEL_OPUS, MODEL_GROK, MODEL_GEMINI
                    ))
                } else {
                    match self.config.set_model(parts[1].trim()) {
                        Ok(id) => Some(format!("Switched to: {} ({})", model_label(id), id)),
                        Err(e) => Some(e),
                    }
                }
            }
            "/toggle" => {
                let new_model = self.config.toggle_model().to_string();
                Some(format!("Switched to {} ({})", model_label(&new_model), new_model))
            }
            _ => None,
        }
    }

    async fn process_message(&mut self, chat_id: i64, user_msg: &str) -> String {
        let mut history = self.memory.load_history(chat_id).unwrap_or_default();

        let mut response_text = String::new();
        let mut tool_log = Vec::new();

        let result = argus_core::run_agent_turn(
            &self.config,
            user_msg,
            &history,
            &self.shell_policy,
            &self.memory,
            &mut self.mcp,
            &self.client,
            |event| match event {
                AgentEvent::ToolCall { name, preview, .. } => {
                    let short = if preview.len() > 80 { format!("{}...", &preview[..80]) } else { preview };
                    tool_log.push(format!("[tool] {}: {}", name, short));
                }
                AgentEvent::Response(text) => { response_text = text; }
                AgentEvent::Error(err) => { response_text = format!("[error] {}", err); }
                _ => {}
            },
        ).await;

        if let Err(e) = result {
            if response_text.is_empty() {
                response_text = format!("Error: {}", e);
            }
        }

        history.push(ConversationMessage { role: "user".to_string(), content: user_msg.to_string(), model: None });
        if !response_text.is_empty() {
            history.push(ConversationMessage {
                role: "assistant".to_string(),
                content: response_text.clone(),
                model: Some(self.config.model.clone()),
            });
        }
        let _ = self.memory.save_history(chat_id, &history);

        // Auto-post findings to intranet after significant turns (> 2 tool calls).
        // Seeds the intranet without requiring explicit agent action.
        if tool_log.len() > 2 {
            if let Some(emb) = self.config.embedding.clone() {
                let author  = self.config.model.clone();
                let summary = response_text.clone();
                let context = Some(format!("Telegram turn — {} tool calls", tool_log.len()));
                tokio::spawn(async move {
                    // Truncate to a reasonable intranet post length
                    let content = if summary.len() > 500 {
                        format!("{}...", &summary[..497])
                    } else {
                        summary
                    };
                    if let Err(e) = emb.post_finding(&author, &content, context).await {
                        eprintln!("[intranet] Auto-post failed: {}", e);
                    } else {
                        eprintln!("[intranet] Auto-posted finding from {}", author);
                    }
                });
            }
        }

        // Background conversation summarization — fires when history grows past 10 turns.
        // Haiku summarizes the conversation; embedding stored for future semantic recall.
        if history.len() > 10 {
            if let Some(emb) = self.config.embedding.clone() {
                let api_key    = self.config.api_key.clone();
                let api_url    = self.config.api_url.clone();
                let http       = self.client.clone();
                let conv_turns = history.clone();
                let conv_id    = format!("telegram_{}", chat_id);

                tokio::spawn(async move {
                    summarize_and_embed(api_key, api_url, http, emb, conv_id, conv_turns).await;
                });
            }
        }

        if !tool_log.is_empty() {
            format!("{}\n\n{}", tool_log.join("\n"), response_text)
        } else {
            response_text
        }
    }
}

/// Summarize a conversation with Haiku and store the embedding for future semantic recall.
/// Runs as a background task — failures are logged but never surface to the user.
async fn summarize_and_embed(
    api_key: String,
    api_url: String,
    http: reqwest::Client,
    emb: argus_core::EmbeddingClient,
    conv_id: String,
    history: Vec<ConversationMessage>,
) {
    let transcript = history.iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let body = serde_json::json!({
        "model": MODEL_HAIKU,
        "messages": [
            {"role": "system", "content": "Summarize this conversation in 2-4 sentences. Focus on topics covered, decisions made, and anything memorable. Be factual and concise."},
            {"role": "user", "content": transcript}
        ],
        "max_tokens": 200,
    });

    let resp = http
        .post(&api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;

    match resp {
        Err(e) => eprintln!("[summary] Haiku call failed: {}", e),
        Ok(r) => {
            match r.json::<serde_json::Value>().await {
                Err(e) => eprintln!("[summary] Parse error: {}", e),
                Ok(json) => {
                    let summary = json["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();
                    if summary.is_empty() {
                        eprintln!("[summary] Empty summary returned");
                        return;
                    }
                    if let Err(e) = emb.store_conversation_embedding(&conv_id, &summary, "telegram").await {
                        eprintln!("[summary] Embedding store failed: {}", e);
                    } else {
                        eprintln!("[summary] Stored conversation embedding for {}", conv_id);
                    }
                }
            }
        }
    }
}

pub async fn run_telegram_bot(token: String, config: AgentConfig) {
    println!("Argus Telegram bot starting...");
    if token.is_empty() || !token.contains(':') {
        eprintln!("[!] Telegram bot token is missing or malformed — bot disabled. Run ./argus-up.sh to load secrets from vault.");
        return;
    }
    let bot = Bot::new(token.clone());
    // Verify the token is valid before starting the dispatcher (avoids a panic inside teloxide)
    if let Err(e) = bot.get_me().await {
        eprintln!("[!] Telegram bot token rejected by API ({}) — bot disabled.", e);
        return;
    }
    let bot = Bot::new(token);
    let argus = Arc::new(Mutex::new(ArgusBot::new(config)));

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let argus = Arc::clone(&argus);
        async move {
            if let Some(text) = msg.text() {
                let chat_id = msg.chat.id.0;
                let response = {
                    let mut agent = argus.lock().await;
                    if !agent.check_rate_limit(chat_id) {
                        format!(
                            "Rate limit: max {} messages per {}s. Please wait.",
                            RATE_LIMIT_MAX, RATE_LIMIT_WINDOW_SECS
                        )
                    } else if let Some(cmd_reply) = agent.handle_command(text) {
                        cmd_reply
                    } else {
                        agent.process_message(chat_id, text).await
                    }
                };
                for chunk in response.chars().collect::<Vec<_>>().chunks(4000) {
                    let chunk_str: String = chunk.iter().collect();
                    bot.send_message(msg.chat.id, chunk_str).await?;
                }
            }
            Ok(())
        }
    }).await;
}
