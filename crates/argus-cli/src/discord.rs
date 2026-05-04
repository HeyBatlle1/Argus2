//! Discord Intranet Integration
//!
//! Connects Argus to a private Discord server used as an intranet.
//! Mirrors the Telegram integration: posts agent findings to a channel,
//! accepts slash commands, and forwards tool call summaries.
//!
//! # Setup
//!
//! 1. Create a Discord application at https://discord.com/developers
//! 2. Add a Bot to the application, copy the token
//! 3. Invite the bot with `bot` + `applications.commands` scopes and the
//!    `Send Messages`, `Read Message History`, `Use Slash Commands` permissions
//! 4. Store credentials:
//!
//!    argus vault set discord_bot_token  YOUR_BOT_TOKEN
//!    argus vault set discord_channel_id YOUR_CHANNEL_SNOWFLAKE_ID
//!
//! # Architecture
//!
//! The Discord bot runs as a background task alongside the Telegram bot and
//! the web server in daemon mode. It shares the same `AgentConfig` / memory
//! backend as the other surfaces.
//!
//! Messages flow:
//!   Discord DM / #channel → ArgusBot::process_message → OpenRouter → response → Discord
//!
//! Auto-posting: after every agent turn that made > 2 tool calls, a summary
//! is posted to the configured intranet channel (same trigger as Telegram).
//!
//! # Enabling
//!
//! This module compiles unconditionally but requires the `discord` feature for
//! the full serenity runtime. To enable:
//!
//!   cargo build --release --features discord
//!
//! Without the feature flag, `run_discord_bot` returns an informative error.

use argus_core::AgentConfig;

/// All credentials needed to run the Discord bot.
#[derive(Debug, Clone)]
#[allow(dead_code)] // fields read via feature-gated serenity bot code
pub struct DiscordConfig {
    /// Bot token from the Discord developer portal.
    pub bot_token: String,
    /// Snowflake ID of the channel to post findings to.
    pub channel_id: u64,
}

impl DiscordConfig {
    /// Try to load config from the vault/env. Returns None if either credential
    /// is missing — caller degrades gracefully.
    pub fn from_env_or_vault(
        bot_token: Option<String>,
        channel_id_str: Option<String>,
    ) -> Option<Self> {
        let token = bot_token.or_else(|| std::env::var("DISCORD_BOT_TOKEN").ok())?;
        let channel_id: u64 = channel_id_str
            .or_else(|| std::env::var("DISCORD_CHANNEL_ID").ok())?
            .trim()
            .parse()
            .ok()?;
        Some(Self { bot_token: token, channel_id })
    }
}

// ── Feature-gated full bot ────────────────────────────────────────────────

#[cfg(feature = "discord")]
pub use full::run_discord_bot;

#[cfg(feature = "discord")]
mod full {
    use super::*;
    use serenity::{
        async_trait,
        model::{channel::Message, gateway::Ready},
        prelude::*,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;

    struct Handler {
        config: Arc<Mutex<AgentConfig>>,
        http: reqwest::Client,
    }

    #[async_trait]
    impl EventHandler for Handler {
        async fn message(&self, ctx: Context, msg: Message) {
            // Ignore bot's own messages
            if msg.author.bot {
                return;
            }

            let user_text = msg.content.trim().to_string();
            if user_text.is_empty() {
                return;
            }

            // Simple echo scaffold — replace with run_agent_turn call
            let response = format!("Argus received: {}", &user_text[..user_text.len().min(100)]);

            if let Err(e) = msg.channel_id.say(&ctx.http, response).await {
                eprintln!("[discord] Failed to send message: {}", e);
            }
        }

        async fn ready(&self, _ctx: Context, ready: Ready) {
            println!("[+] Discord bot connected as {}", ready.user.name);
        }
    }

    pub async fn run_discord_bot(
        discord_cfg: DiscordConfig,
        agent_cfg: AgentConfig,
    ) -> anyhow::Result<()> {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = Client::builder(&discord_cfg.bot_token, intents)
            .event_handler(Handler {
                config: Arc::new(Mutex::new(agent_cfg)),
                http: reqwest::Client::new(),
            })
            .await
            .map_err(|e| anyhow::anyhow!("Discord client error: {}", e))?;

        client
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Discord bot error: {}", e))
    }
}

// ── Stub when discord feature is off ─────────────────────────────────────

#[cfg(not(feature = "discord"))]
pub async fn run_discord_bot(
    _discord_cfg: DiscordConfig,
    _agent_cfg: AgentConfig,
) -> anyhow::Result<()> {
    anyhow::bail!(
        "Discord support is not compiled in. Rebuild with: cargo build --release --features discord"
    )
}
