//! Telegram Bot for Argus

use teloxide::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

use argus_memory::sqlite::SqliteMemory;
use argus_core::{AgentConfig, AgentEvent, ConversationMessage, ShellPolicy, MODEL_HAIKU, MODEL_SONNET, MODEL_OPUS, MODEL_GROK, model_label};

struct ArgusBot {
    config: AgentConfig,
    client: reqwest::Client,
    memory: SqliteMemory,
    mcp: argus_core::mcp::McpClient,
    shell_policy: ShellPolicy,
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
        }
    }

    fn handle_command(&mut self, text: &str) -> Option<String> {
        let parts: Vec<&str> = text.splitn(2, ' ').collect();
        match parts[0] {
            "/model" => {
                if parts.len() == 1 {
                    Some(format!(
                        "Current model: {} ({})\n\nAvailable:\n  haiku  — {}\n  sonnet — {}\n  opus   — {}\n  grok   — {}\n\nSwitch with /model haiku, /model sonnet, /model opus, or /model grok",
                        model_label(&self.config.model), self.config.model,
                        MODEL_HAIKU, MODEL_SONNET, MODEL_OPUS, MODEL_GROK
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

        history.push(ConversationMessage { role: "user".to_string(), content: user_msg.to_string() });
        if !response_text.is_empty() {
            history.push(ConversationMessage { role: "assistant".to_string(), content: response_text.clone() });
        }
        let _ = self.memory.save_history(chat_id, &history);

        if !tool_log.is_empty() {
            format!("{}\n\n{}", tool_log.join("\n"), response_text)
        } else {
            response_text
        }
    }
}

pub async fn run_telegram_bot(token: String, config: AgentConfig) {
    println!("Argus Telegram bot starting...");
    let bot = Bot::new(token);
    let argus = Arc::new(Mutex::new(ArgusBot::new(config)));

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let argus = Arc::clone(&argus);
        async move {
            if let Some(text) = msg.text() {
                let chat_id = msg.chat.id.0;
                let response = {
                    let mut agent = argus.lock().await;
                    if let Some(cmd_reply) = agent.handle_command(text) {
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
