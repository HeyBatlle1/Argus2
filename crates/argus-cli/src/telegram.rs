//! Telegram Bot for Argus
//! Uses the shared agent loop from argus-core

use teloxide::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

use argus_memory::sqlite::SqliteMemory;
use argus_core::{AgentConfig, AgentEvent, ShellPolicy};

struct ArgusBot {
    config: AgentConfig,
    client: reqwest::Client,
    memory: SqliteMemory,
    mcp: argus_core::mcp::McpClient,
    shell_policy: ShellPolicy,
}

impl ArgusBot {
    fn new(api_key: String) -> Self {
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
            config: AgentConfig::new(api_key),
            client: reqwest::Client::new(),
            memory: SqliteMemory::open_default().expect("failed to open memory db"),
            mcp,
            shell_policy,
        }
    }

    async fn process_message(&mut self, user_msg: &str) -> String {
        let mut response_text = String::new();
        let mut tool_log = Vec::new();

        let result = argus_core::run_agent_turn(
            &self.config,
            user_msg,
            &self.shell_policy,
            &self.memory,
            &mut self.mcp,
            &self.client,
            |event| {
                match event {
                    AgentEvent::ToolCall { name, preview } => {
                        tool_log.push(format!("🔧 {}: {}", name, 
                            if preview.len() > 80 { format!("{}...", &preview[..80]) } else { preview }
                        ));
                    }
                    AgentEvent::Response(text) => {
                        response_text = text;
                    }
                    AgentEvent::Error(err) => {
                        response_text = format!("❌ {}", err);
                    }
                    _ => {}
                }
            },
        ).await;

        if let Err(e) = result {
            if response_text.is_empty() {
                response_text = format!("Error: {}", e);
            }
        }

        // Prepend tool activity if any
        if !tool_log.is_empty() {
            format!("{}\n\n{}", tool_log.join("\n"), response_text)
        } else {
            response_text
        }
    }
}

pub async fn run_telegram_bot(token: String, api_key: String) {
    println!("Argus Telegram bot starting...");
    let bot = Bot::new(token);
    let argus = Arc::new(Mutex::new(ArgusBot::new(api_key)));

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let argus = Arc::clone(&argus);
        async move {
            if let Some(text) = msg.text() {
                let response = {
                    let mut agent = argus.lock().await;
                    agent.process_message(text).await
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
