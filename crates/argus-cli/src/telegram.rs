//! Telegram Bot for Argus

use teloxide::prelude::*;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

use argus_memory::sqlite::SqliteMemory;
use argus_core::{AgentConfig, AgentEvent, ConversationMessage, ShellPolicy};

struct ArgusBot {
    config: AgentConfig,
    client: reqwest::Client,
    memory: SqliteMemory,
    mcp: argus_core::mcp::McpClient,
    shell_policy: ShellPolicy,
    histories: HashMap<i64, Vec<ConversationMessage>>,
}

impl ArgusBot {
    fn new(config: AgentConfig) -> Self {
        let mut mcp = argus_core::mcp::McpClient::new();
        let _ = mcp.connect_all();

        let mut shell_policy = ShellPolicy::empty();
        for cmd in &[
            "ls", "cat", "head", "tail", "grep", "find", "wc", "echo", "pwd",
            "date", "whoami", "uname", "df", "du", "ps", "which", "file",
            "stat", "sort", "uniq", "tr", "cut", "awk", "sed",
            "curl", "wget", "git", "cargo", "python3", "node", "npm", "pip",
        ] {
            shell_policy.allow(cmd);
        }

        Self {
            config,
            client: reqwest::Client::new(),
            memory: SqliteMemory::open_default().expect("failed to open memory db"),
            mcp,
            shell_policy,
            histories: HashMap::new(),
        }
    }

    async fn process_message(&mut self, chat_id: i64, user_msg: &str) -> String {
        let history_snapshot = self.histories.entry(chat_id).or_default().clone();

        let mut response_text = String::new();
        let mut tool_log = Vec::new();

        let result = argus_core::run_agent_turn(
            &self.config,
            user_msg,
            &history_snapshot,
            &self.shell_policy,
            &self.memory,
            &mut self.mcp,
            &self.client,
            |event| match event {
                AgentEvent::ToolCall { name, preview } => {
                    let short = if preview.len() > 80 { format!("{}...", &preview[..80]) } else { preview };
                    tool_log.push(format!("\u{1f527} {}: {}", name, short));
                }
                AgentEvent::Response(text) => { response_text = text; }
                AgentEvent::Error(err) => { response_text = format!("\u274c {}", err); }
                _ => {}
            },
        ).await;

        if let Err(e) = result {
            if response_text.is_empty() {
                response_text = format!("Error: {}", e);
            }
        }

        let history = self.histories.entry(chat_id).or_default();
        history.push(ConversationMessage { role: "user".to_string(), content: user_msg.to_string() });
        if !response_text.is_empty() {
            history.push(ConversationMessage { role: "assistant".to_string(), content: response_text.clone() });
        }
        // Keep last 40 messages to stay under token limits
        if history.len() > 40 {
            let drain_to = history.len() - 40;
            history.drain(0..drain_to);
        }

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
                    agent.process_message(chat_id, text).await
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
