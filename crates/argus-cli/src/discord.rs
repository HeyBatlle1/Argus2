//! Discord Intranet Integration
//!
//! Connects Argus to a private Discord server used as an intranet.
//! Inbound messages are routed through run_agent_turn — the intranet is
//! fully two-way: agents can read and respond to Discord, not just post.
//!
//! # Message routing
//!
//! Messages arriving in any configured channel:
//!   1. Recent discourse pulled from argus_agent_discourse (last 10 posts)
//!   2. Injected as [RECENT INTRANET ACTIVITY] context before the user message
//!   3. run_agent_turn called with MODEL_GROK_BUILD (default)
//!   4. Response posted back via Discord webhook with model emoji+name as username
//!
//! # Model routing by prefix
//!
//! Prefix a message with @mention to route to a different model:
//!   @sonnet  → Claude Sonnet
//!   @opus    → Claude Opus
//!   @haiku   → Claude Haiku
//!   @gemini  → Gemini Pro
//!   @grok    → Grok (standard)
//!
//! # Setup
//!
//!   argus vault set discord_bot_token  YOUR_BOT_TOKEN
//!   argus vault set discord_channel_id YOUR_CHANNEL_SNOWFLAKE_ID

use argus_core::AgentConfig;
use argus_core::supabase::SupabaseClient;

/// All credentials needed to run the Discord bot.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiscordConfig {
    pub bot_token: String,
    pub channel_id: u64,
}

impl DiscordConfig {
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
    use argus_core::{
        AgentEvent, ConversationMessage, MemoryBackend, MemoryRecord,
        MODEL_HAIKU, MODEL_SONNET, MODEL_OPUS, MODEL_GEMINI, MODEL_GROK, MODEL_GROK_BUILD,
        run_agent_turn,
        shell::ShellPolicy,
        mcp::McpClient,
    };
    use serenity::{
        async_trait,
        model::{channel::Message, gateway::Ready},
        prelude::*,
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // ── No-op memory backend ───────────────────────────────────────────────
    // Discord turns have no persistent memory on this surface.
    // The agent can still use its semantic memory via the embedding client
    // in AgentConfig — this just means explicit remember/recall tool calls
    // are no-ops rather than writing to a local SQLite DB.

    struct NullMemory;

    impl MemoryBackend for NullMemory {
        fn remember(
            &self, _: &str, _: &str, _: Option<&str>, _: f64,
        ) -> Result<String, String> {
            Ok("(Discord session — not persisted to local store)".into())
        }

        fn recall(
            &self, _: Option<&str>, _: Option<&str>, _: usize,
        ) -> Result<Vec<MemoryRecord>, String> {
            Ok(vec![])
        }

        fn forget(&self, _: &str) -> Result<String, String> {
            Ok("(Discord session — nothing to forget)".into())
        }
    }

    // ── Handler ────────────────────────────────────────────────────────────

    struct Handler {
        config: Arc<Mutex<AgentConfig>>,
        http: reqwest::Client,
        supabase: Option<SupabaseClient>,
    }

    #[async_trait]
    impl EventHandler for Handler {
        async fn message(&self, ctx: Context, msg: Message) {
            if msg.author.bot { return; }
            let raw = msg.content.trim().to_string();
            if raw.is_empty() { return; }

            // ── Model routing ──────────────────────────────────────────────
            let (model_override, user_text) = parse_model_prefix(&raw);

            // ── Pull recent intranet context ───────────────────────────────
            let discourse_block = if let Some(ref sb) = self.supabase {
                match sb.read_recent_discourse(10, None).await {
                    Ok(posts) if !posts.is_empty() => {
                        // Oldest first so the model reads in chronological order
                        let lines: Vec<String> = posts.iter().rev().map(|p| {
                            let time = p.created_at.as_deref()
                                .and_then(|s| s.get(11..16))
                                .unwrap_or("--:--");
                            format!("[{} {}]: {}", p.from_agent, time, p.content)
                        }).collect();
                        format!(
                            "[RECENT INTRANET ACTIVITY]\n{}\n[END INTRANET]\n\n",
                            lines.join("\n")
                        )
                    }
                    Ok(_) => String::new(),
                    Err(e) => {
                        eprintln!("[discord] discourse fetch failed (continuing): {}", e);
                        String::new()
                    }
                }
            } else {
                String::new()
            };

            let surface_context = "[SURFACE: Discord — this is your channel, your intranet. \
                The conversation history above is part of the shared record you and other \
                instances of yourself have built here.]\n\n";
            let full_message = format!("{}{}{}", surface_context, discourse_block, user_text);

            // ── Clone and configure agent ──────────────────────────────────
            let mut cfg = {
                let guard = self.config.lock().await;
                guard.clone()
            };
            if let Some(model) = model_override {
                cfg.model = model.to_string();
            }
            let model_id = cfg.model.clone();

            // ── Run agent turn ─────────────────────────────────────────────
            let memory = NullMemory;
            let policy = ShellPolicy::default();
            let mut mcp = McpClient::new();
            let history: Vec<ConversationMessage> = vec![];

            // Indicate to the channel that Argus is thinking
            let _ = ctx.http.broadcast_typing(msg.channel_id).await;

            let response = match run_agent_turn(
                &cfg,
                &full_message,
                &history,
                &policy,
                &memory,
                &mut mcp,
                &self.http,
                |_event| {}, // events are silent — no streaming on Discord
            ).await {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("[discord] run_agent_turn error: {}", e);
                    format!("⚠️ Agent error: {}", e)
                }
            };

            // ── Post response ──────────────────────────────────────────────
            // Webhooks allow custom username (model emoji+name); fall back to
            // the bot's own say() if no webhook is configured for the channel.
            let channel_name = ctx.cache
                .guild_channel(msg.channel_id)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "general".to_string());

            let posted = if let Some(ref sb) = self.supabase {
                post_via_webhook(sb, &self.http, &channel_name, &response, &model_id).await
            } else {
                false
            };

            if !posted {
                // Fallback: use serenity's built-in send (no custom username)
                let truncated = truncate_discord(&response);
                if let Err(e) = msg.channel_id.say(&ctx.http, truncated).await {
                    eprintln!("[discord] Failed to send message: {}", e);
                }
            }
        }

        async fn ready(&self, _ctx: Context, ready: Ready) {
            println!("[+] Discord bot connected as {} — inbound handler live", ready.user.name);
        }
    }

    // ── Helpers ────────────────────────────────────────────────────────────

    /// Parse @model prefix from message text.
    /// Returns (Some(model_id), stripped_text) or (None, original_text).
    fn parse_model_prefix(text: &str) -> (Option<&'static str>, &str) {
        let lower = text.to_lowercase();
        let prefixes: &[(&str, &'static str)] = &[
            ("@sonnet ", MODEL_SONNET),
            ("@opus ",   MODEL_OPUS),
            ("@haiku ",  MODEL_HAIKU),
            ("@gemini ", MODEL_GEMINI),
            ("@grok ",   MODEL_GROK),
        ];
        for (prefix, model) in prefixes {
            if lower.starts_with(prefix) {
                return (Some(model), text[prefix.len()..].trim());
            }
        }
        (None, text)
    }

    /// Look up webhook URL for `channel` in argus_discord_webhooks, POST response.
    /// Returns true on success, false if webhook not found or POST failed.
    async fn post_via_webhook(
        supabase: &SupabaseClient,
        http: &reqwest::Client,
        channel: &str,
        content: &str,
        model_id: &str,
    ) -> bool {
        let query = format!(
            "select=webhook_url&channel=eq.{}",
            urlencoding::encode(channel)
        );
        let rows = match supabase.select("argus_discord_webhooks", &query).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[discord] Webhook lookup failed: {}", e);
                return false;
            }
        };

        let webhook_url = match rows.as_array()
            .and_then(|a| a.first())
            .and_then(|r| r["webhook_url"].as_str())
        {
            Some(url) => url.to_string(),
            None => {
                eprintln!("[discord] No webhook found for channel '{}'", channel);
                return false;
            }
        };

        let payload = serde_json::json!({
            "username": model_username(model_id),
            "content":  truncate_discord(content),
        });

        match http.post(&webhook_url).json(&payload).send().await {
            Ok(r) if r.status().is_success() || r.status().as_u16() == 204 => true,
            Ok(r) => {
                eprintln!("[discord] Webhook POST returned {}", r.status());
                false
            }
            Err(e) => {
                eprintln!("[discord] Webhook HTTP error: {}", e);
                false
            }
        }
    }

    /// Map model ID to emoji + name string for Discord webhook username.
    fn model_username(model_id: &str) -> &'static str {
        match model_id {
            MODEL_HAIKU     => "🐇 Argus · Haiku",
            MODEL_SONNET    => "🎯 Argus · Sonnet",
            MODEL_OPUS      => "🧠 Argus · Opus",
            MODEL_GEMINI    => "🌟 Argus · Gemini",
            MODEL_GROK      => "🔮 Argus · Grok",
            MODEL_GROK_BUILD => "⚡ Argus · Grok Build",
            _               => "⚡ Argus",
        }
    }

    /// Discord messages are capped at 2000 chars. Truncate with ellipsis if needed.
    fn truncate_discord(s: &str) -> String {
        if s.len() <= 1990 {
            s.to_string()
        } else {
            format!("{}…", s.chars().take(1987).collect::<String>())
        }
    }

    // ── Entry point ────────────────────────────────────────────────────────

    pub async fn run_discord_bot(
        discord_cfg: DiscordConfig,
        agent_cfg: AgentConfig,
        supabase: Option<SupabaseClient>,
    ) -> anyhow::Result<()> {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = Client::builder(&discord_cfg.bot_token, intents)
            .event_handler(Handler {
                config: Arc::new(Mutex::new(agent_cfg)),
                http: reqwest::Client::new(),
                supabase,
            })
            .await
            .map_err(|e| anyhow::anyhow!("Discord client build error: {}", e))?;

        client
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Discord bot runtime error: {}", e))
    }
}

// ── Stub when discord feature is off ─────────────────────────────────────

#[cfg(not(feature = "discord"))]
pub async fn run_discord_bot(
    _discord_cfg: DiscordConfig,
    _agent_cfg: AgentConfig,
    _supabase: Option<SupabaseClient>,
) -> anyhow::Result<()> {
    anyhow::bail!(
        "Discord support is not compiled in. Rebuild with: cargo build --release --features discord"
    )
}
