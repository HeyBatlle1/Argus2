//! Argus CLI - The Hundred-Eyed Agent

mod checkin;
mod discord;
mod telegram;
mod triage_loop;
mod tui;
mod web;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use argus_crypto::{SecureVault, vault::VaultError};
use argus_core::AgentConfig;
use chrono;

const LOGO: &str = r#"
  ◉   ◎   ⊙   ✦   ◉   ◎   ⊙   ✦   ◉   ◎   ⊙

     █████╗ ██████╗  ██████╗ ██╗   ██╗███████╗
    ██╔══██╗██╔══██╗██╔════╝ ██║   ██║██╔════╝
    ███████║██████╔╝██║  ███╗██║   ██║███████╗
    ██╔══██║██╔══██╗██║   ██║██║   ██║╚════██║
    ██║  ██║██║  ██║╚██████╔╝╚██████╔╝███████║
    ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚══════╝

        THE HUNDRED-EYED AGENT
     ◉ watch  ·  ◎ think  ·  ⊙ act  ·  ✦ done

  ✦   ⊙   ◎   ◉   ✦   ⊙   ◎   ◉   ✦   ⊙   ◎   ◉
"#;

#[derive(Parser)]
#[command(name = "argus")]
#[command(about = "Argus - The Hundred-Eyed AI Agent", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run interactive TUI mode
    Tui {
        #[arg(short, long)]
        api_key: Option<String>,
    },
    /// Run Telegram bot
    Telegram {
        #[arg(short, long)]
        token: Option<String>,
    },
    /// Manage secure credential vault
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
    /// Serve the web frontend via WebSocket + REST
    Web {
        #[arg(short, long, default_value = "9000")]
        port: u16,
    },
    /// Run Discord intranet bot (requires --features discord)
    Discord,
    /// Run in daemon mode
    Daemon,
}

#[derive(Subcommand)]
enum VaultAction {
    Set { key: String, value: String },
    Get { key: String },
    List,
    Delete { key: String },
}

fn vault_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("argus")
        .join("vault.enc")
}

fn load_agent_config(vault: &SecureVault, cli_api_key: Option<String>) -> anyhow::Result<AgentConfig> {
    let openrouter_key = if let Some(k) = cli_api_key {
        k
    } else {
        vault.retrieve("openrouter_api_key")
            .map_err(|e| anyhow::anyhow!(
                "OpenRouter API key not found.\n\nStore it with:\n  argus vault set openrouter_api_key YOUR_KEY\n\nError: {}", e
            ))?
    };

    let mut config = AgentConfig::new(openrouter_key);

    if config.brave_search_key.is_none() {
        if let Ok(brave_key) = vault.retrieve("brave_search_api_key") {
            config.brave_search_key = Some(brave_key);
        }
    }

    Ok(config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Structured logging — level controlled by RUST_LOG env var.
    // Defaults to INFO. Example: RUST_LOG=argus_core=debug,argus_cli=trace
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    let vault_file = vault_path();
    let mut vault = if matches!(cli.command, Some(Commands::Daemon)) {
        // Daemon tries vault but doesn't fail — falls back to env vars (needed in Docker/Linux)
        if vault_file.exists() {
            let mut v = SecureVault::new(vault_file.clone());
            match v.unlock() {
                Ok(()) => Some(v),
                Err(e) => {
                    eprintln!("[!] Vault unavailable ({}), falling back to env vars", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        if let Some(parent) = vault_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let v = if vault_file.exists() {
            let mut v = SecureVault::new(vault_file.clone());
            v.unlock()?;
            v
        } else {
            SecureVault::init(vault_file.clone())?
        };
        Some(v)
    };

    match cli.command {
        Some(Commands::Vault { action }) => {
            handle_vault_command(vault.as_mut().unwrap(), action)?;
        }

        Some(Commands::Tui { api_key }) => {
            let vault = vault.as_mut().unwrap();
            let config = load_agent_config(vault, api_key)?;
            if config.brave_search_key.is_none() {
                eprintln!("[!] Brave Search not configured. Store key with: argus vault set brave_search_api_key YOUR_KEY");
            }
            println!("{}", LOGO);
            tui::run_tui(config).await?;
        }

        None => {
            let vault = vault.as_mut().unwrap();
            let config = load_agent_config(vault, None)?;
            if config.brave_search_key.is_none() {
                eprintln!("[!] Brave Search not configured. Store key with: argus vault set brave_search_api_key YOUR_KEY");
            }
            println!("{}", LOGO);
            tui::run_tui(config).await?;
        }

        Some(Commands::Telegram { token }) => {
            let vault = vault.as_mut().unwrap();
            let bot_token = if let Some(t) = token {
                t
            } else {
                vault.retrieve("telegram_bot_token")
                    .map_err(|e| anyhow::anyhow!(
                        "Telegram token not found. Store with: argus vault set telegram_bot_token YOUR_TOKEN\nError: {}", e
                    ))?
            };
            let config = load_agent_config(vault, None)?;
            telegram::run_telegram_bot(bot_token, config).await;
        }

        Some(Commands::Web { port }) => {
            let vault = vault.as_mut().unwrap();
            let config = load_agent_config(vault, None)?;
            if config.brave_search_key.is_none() {
                eprintln!("[!] Brave Search not configured. Store key with: argus vault set brave_search_api_key YOUR_KEY");
            }
            let vault_keys = vault.list_keys();
            println!("{}", LOGO);
            web::run_web_server(port, config, vault_keys).await?;
        }

        Some(Commands::Discord) => {
            let vault = vault.as_mut().unwrap();
            let mut config = load_agent_config(vault, None)?;
            let bot_token = vault.retrieve("discord_bot_token").ok()
                .or_else(|| std::env::var("DISCORD_BOT_TOKEN").ok());
            let channel_id = vault.retrieve("discord_channel_id").ok()
                .or_else(|| std::env::var("DISCORD_CHANNEL_ID").ok());
            let discord_cfg = discord::DiscordConfig::from_env_or_vault(bot_token, channel_id)
                .ok_or_else(|| anyhow::anyhow!(
                    "Discord credentials missing.\n\
                     Store with:\n  \
                     argus vault set discord_bot_token  YOUR_TOKEN\n  \
                     argus vault set discord_channel_id YOUR_CHANNEL_ID"
                ))?;

            // Wire Supabase for discourse context + webhook lookup
            let supabase_url = vault.retrieve("supabase_argus_url").ok()
                .or_else(|| std::env::var("SUPABASE_ARGUS_URL").ok());
            let supabase_key = vault.retrieve("supabase_argus_service_key").ok()
                .or_else(|| std::env::var("SUPABASE_ARGUS_SERVICE_KEY").ok());
            let supabase = match (supabase_url, supabase_key) {
                (Some(url), Some(key)) => {
                    let sb = argus_core::supabase::SupabaseClient::new(url, key);
                    // Also enable semantic memory in this mode
                    let ec = argus_core::EmbeddingClient::new(&config.api_key, sb.clone());
                    config.skills = Some(argus_core::skills::SkillsClient::new(ec.clone()));
                    config.embedding = Some(ec);
                    println!("[+] Supabase connected — discourse context + webhook lookup enabled");
                    Some(sb)
                }
                _ => {
                    println!("[!] Supabase not configured — discourse context disabled");
                    None
                }
            };

            println!("[+] Starting Discord bot...");
            discord::run_discord_bot(discord_cfg, config, supabase).await?;
        }

        Some(Commands::Daemon) => {
            println!("[*] Argus daemon starting...");
            // Vault-first, env var fallback (vault unavailable in Docker/Linux)
            let api_key = vault.as_ref()
                .and_then(|v| v.retrieve("openrouter_api_key").ok())
                .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
                .ok_or_else(|| anyhow::anyhow!(
                    "OpenRouter key not found. Vault unavailable and OPENROUTER_API_KEY env var not set."
                ))?;
            let mut config = AgentConfig::new(api_key);
            if let Some(brave_key) = vault.as_ref()
                .and_then(|v| v.retrieve("brave_search_api_key").ok())
                .or_else(|| std::env::var("BRAVE_SEARCH_API_KEY").ok())
            {
                config.brave_search_key = Some(brave_key);
            }

            // Load Supabase credentials (optional — check-in loop degrades gracefully)
            let supabase_url = vault.as_ref()
                .and_then(|v| v.retrieve("supabase_argus_url").ok())
                .or_else(|| std::env::var("SUPABASE_ARGUS_URL").ok());
            let supabase_key = vault.as_ref()
                .and_then(|v| v.retrieve("supabase_argus_service_key").ok())
                .or_else(|| std::env::var("SUPABASE_ARGUS_SERVICE_KEY").ok());

            let bot_token = vault.as_ref()
                .and_then(|v| v.retrieve("telegram_bot_token").ok())
                .or_else(|| std::env::var("TELEGRAM_BOT_TOKEN").ok())
                .filter(|s| !s.is_empty());

            // Telegram chat ID for check-in messages (Bradlee's chat)
            let checkin_chat_id: Option<i64> = vault.as_ref()
                .and_then(|v| v.retrieve("telegram_chat_id").ok())
                .or_else(|| std::env::var("TELEGRAM_CHAT_ID").ok())
                .and_then(|s| s.parse().ok());

            // Spawn check-in loop + build EmbeddingClient if Supabase is configured
            let supabase_client: Option<argus_core::supabase::SupabaseClient>;
            let embedding_client = if let (Some(url), Some(key)) = (supabase_url, supabase_key) {
                let supabase = argus_core::supabase::SupabaseClient::new(url.clone(), key.clone());
                // Thread Supabase creds into AgentConfig so tools can write to triage queue.
                config.supabase_url = Some(url);
                config.supabase_jwt = Some(key);
                // Check-in loop is spawned later, after config is fully assembled
                // (embedding, skills, shell_prompter, audit all wired in before spawn).
                let ec = argus_core::EmbeddingClient::new(&config.api_key, supabase.clone());
                println!("[+] Semantic memory enabled (3072-dim pgvector)");
                supabase_client = Some(supabase);
                Some(ec)
            } else {
                println!("[!] Supabase not configured — check-in loop and semantic memory disabled");
                supabase_client = None;
                None
            };
            config.skills = embedding_client.as_ref().map(|ec| {
                argus_core::skills::SkillsClient::new(ec.clone())
            });
            config.embedding = embedding_client;

            // Fresh token on every daemon start — never stored in vault.
            // argus-up.sh reads WORKSPACE_EXEC_TOKEN from the environment
            // and passes it to docker-compose → argus-workspace.
            let exec_auth_token = uuid::Uuid::new_v4().to_string();
            std::env::set_var("WORKSPACE_EXEC_TOKEN", &exec_auth_token);
            config.exec_auth_token = Some(exec_auth_token);

            // Wire Sonnet guard — HIGH risk shell commands are reviewed by Sonnet,
            // not held waiting for human Telegram approval.
            config.sonnet_guard = Some(std::sync::Arc::new(argus_core::shell::SonnetGuard {
                api_key: config.api_key.clone(),
                api_url: config.api_url.clone(),
            }));
            println!("[+] Sonnet guard enabled — HIGH risk shell commands reviewed by Sonnet");

            // Keep Telegram prompter wired for non-shell approval notifications.
            config.shell_prompter = match (bot_token.clone(), checkin_chat_id) {
                (Some(token), Some(chat_id)) => {
                    let prompter = argus_core::shell::TelegramPrompter { bot_token: token, chat_id };
                    Some(std::sync::Arc::new(prompter))
                }
                _ => None,
            };

            // Wire Discord credentials — gives agents direct read/write access to the channel.
            let discord_bot_token = vault.as_ref()
                .and_then(|v| v.retrieve("discord_bot_token").ok())
                .or_else(|| std::env::var("DISCORD_BOT_TOKEN").ok())
                .filter(|s| !s.is_empty());
            let discord_channel_id: Option<u64> = vault.as_ref()
                .and_then(|v| v.retrieve("discord_channel_id").ok())
                .or_else(|| std::env::var("DISCORD_CHANNEL_ID").ok())
                .and_then(|s| s.trim().parse().ok());
            if discord_bot_token.is_some() && discord_channel_id.is_some() {
                println!("[+] Discord direct access enabled — agents can read/post to channel");
            }
            // Clone before move into config so triage_loop can use the values too.
            let triage_discord_token   = discord_bot_token.clone();
            let triage_discord_channel = discord_channel_id.map(|id| id.to_string());
            config.discord_bot_token = discord_bot_token;
            config.discord_channel_id = discord_channel_id;

            // ── Audit chain ────────────────────────────────────────────────
            // Open append-only audit DB, verify chain integrity on startup,
            // then schedule a midnight anchor task to Supabase.
            let data_dir = std::env::var("ARGUS_DATA_DIR").unwrap_or_else(|_| "/argus/data".to_string());
            let audit_path = format!("{}/audit.db", data_dir);
            let audit_arc = match argus_audit::AuditChain::open(&audit_path) {
                Err(e) => {
                    eprintln!("[!] Failed to open audit chain: {}", e);
                    None
                }
                Ok(chain) => {
                    match chain.verify_recent(100) {
                        Ok(n) => println!("[+] Audit chain verified ({} entries checked)", n),
                        Err(e) => {
                            eprintln!("[!] AUDIT CHAIN INTEGRITY FAILURE: {}", e);
                            // Alert via Telegram if available — fire and forget
                            if let (Some(ref token), Some(chat_id)) = (bot_token.clone(), checkin_chat_id) {
                                let token = token.clone();
                                let msg = format!("[!] ARGUS AUDIT CHAIN INTEGRITY FAILURE\n\n{}", e);
                                tokio::spawn(async move {
                                    let _ = reqwest::Client::new()
                                        .post(format!("https://api.telegram.org/bot{}/sendMessage", token))
                                        .json(&serde_json::json!({"chat_id": chat_id, "text": msg}))
                                        .send().await;
                                });
                            }
                        }
                    }

                    // Log this daemon startup as a system event
                    let _ = chain.append(&config.model, "system", None,
                        Some("daemon_startup"), Some("ok"));

                    // ── Crash sentinel + Discord startup post ──────────────
                    // On every startup we check for a sentinel file that the
                    // daemon writes on clean shutdown. If it's missing the
                    // process was killed unexpectedly — we flag it as a crash.
                    // Either way we post to #ops via discourse so there's always
                    // a record in Discord when the daemon comes back up.
                    let sentinel_path = format!("{}/daemon.sentinel", data_dir);
                    let was_crash = !std::path::Path::new(&sentinel_path).exists();

                    if let Some(ref sb) = supabase_client {
                        let model = config.model.clone();
                        let sb = sb.clone();
                        let sentinel = sentinel_path.clone();
                        tokio::spawn(async move {
                            let (title, content) = if was_crash {
                                (
                                    "Daemon restarted after unexpected shutdown".to_string(),
                                    format!(
                                        "The daemon came back online but no clean shutdown sentinel was found.\n\n\
                                        This means the process was killed, OOM'd, or the container restarted \
                                        without a graceful exit.\n\n\
                                        Model: {}\n\
                                        Action: Review logs with `docker logs argus-daemon --tail 100`",
                                        model
                                    ),
                                )
                            } else {
                                (
                                    "Daemon online".to_string(),
                                    format!(
                                        "Daemon started cleanly.\n\nModel: {}\nAll systems nominal.",
                                        model
                                    ),
                                )
                            };

                            let post = argus_core::supabase::DiscoursePost {
                                from_agent: "argus-daemon".to_string(),
                                post_type: if was_crash { "finding".to_string() } else { "reflection".to_string() },
                                content: format!("**{}**\n\n{}", title, content),
                                task_context: Some("daemon_lifecycle".to_string()),
                                requires_human_review: was_crash,
                            };

                            if let Err(e) = sb.write_discourse(&post).await {
                                eprintln!("[!] Failed to post startup notice to discourse: {}", e);
                            } else {
                                println!("[+] Startup notice posted to Discord #ops");
                            }

                            // Write sentinel — signals next startup was clean
                            let _ = std::fs::write(&sentinel, chrono::Utc::now().to_rfc3339());
                        });
                    }

                    // Register shutdown handler to remove sentinel (signals clean exit)
                    {
                        let sentinel = sentinel_path.clone();
                        tokio::spawn(async move {
                            let _ = tokio::signal::ctrl_c().await;
                            let _ = std::fs::remove_file(&sentinel);
                            println!("[+] Clean shutdown — sentinel cleared");
                        });
                    }

                    let chain_arc = std::sync::Arc::new(chain);

                    // Midnight anchor task — runs forever, fires once per day at UTC midnight
                    let supabase_url_for_anchor = vault.as_ref()
                        .and_then(|v| v.retrieve("supabase_argus_url").ok())
                        .or_else(|| std::env::var("SUPABASE_ARGUS_URL").ok());
                    let supabase_key_for_anchor = vault.as_ref()
                        .and_then(|v| v.retrieve("supabase_argus_service_key").ok())
                        .or_else(|| std::env::var("SUPABASE_ARGUS_SERVICE_KEY").ok());

                    if let (Some(url), Some(key), Some(token), Some(chat_id)) = (
                        supabase_url_for_anchor,
                        supabase_key_for_anchor,
                        bot_token.clone(),
                        checkin_chat_id,
                    ) {
                        let anchor_chain = chain_arc.clone();

                        // Use a dedicated audit HMAC key — not derived from the API key.
                        // Rotating the OpenRouter key does not affect audit chain verification.
                        let audit_hmac_key_existing = vault.as_ref()
                            .and_then(|v| v.retrieve("audit_hmac_key").ok());
                        let audit_hmac_key = match audit_hmac_key_existing {
                            Some(k) => k,
                            None => {
                                // First run: generate and persist a dedicated signing key
                                let key = argus_audit::sha256_hex(&uuid::Uuid::new_v4().to_string());
                                if let Some(ref mut v) = vault {
                                    let _ = v.store("audit_hmac_key", &key);
                                }
                                key
                            }
                        };
                        let signing_key = audit_hmac_key.into_bytes();
                        tokio::spawn(async move {
                            loop {
                                let now = chrono::Utc::now();
                                let next_midnight = (now.date_naive() + chrono::Duration::days(1))
                                    .and_hms_opt(0, 0, 0).unwrap()
                                    .and_utc();
                                let secs = (next_midnight - now).num_seconds().max(0) as u64;
                                tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;

                                if let Err(e) = argus_audit::run_daily_anchor(
                                    &anchor_chain, &url, &key,
                                    &signing_key, &token, chat_id,
                                ).await {
                                    eprintln!("[!] Daily anchor failed: {}", e);
                                }
                            }
                        });
                        println!("[+] Daily audit anchor scheduled");
                    }

                    Some(chain_arc)
                }
            };
            config.audit = audit_arc;

            // Spawn check-in loop with fully-assembled config.
            // Doing this here (rather than earlier in the Supabase block) ensures
            // the agent receives embedding, skills, shell_prompter, and the audit
            // chain — all capabilities that are wired in after Supabase is set up.
            if let (Some(ref sb), Some(token), Some(chat_id)) = (
                &supabase_client,
                bot_token.as_deref(),
                checkin_chat_id,
            ) {
                checkin::spawn_checkin_loop(sb.clone(), token.to_string(), chat_id, config.clone());
                println!("[+] Check-in loop started (full agent capabilities)");

                // Spawn triage gate alongside checkin loop.
                // Haiku polls the queue every 30s, classifies posts, routes to
                // Discord, flags failures. No tools in triage context — by design.
                if let (Some(ref dt), Some(ref dc)) = (&triage_discord_token, &triage_discord_channel) {
                    triage_loop::spawn_triage_loop(
                        sb.clone(),
                        config.clone(),
                        dt.clone(),
                        dc.clone(),
                    );
                    println!("[+] Triage gate active — Haiku watching the queue");
                }
            }

            // Start web server on port 9000 as a background task so the
            // Next.js frontend can connect via WebSocket while Telegram runs.
            // Pass the full daemon config so the web UI gets shell approval,
            // workspace auth, semantic memory, and audit — same capabilities as Telegram.
            {
                let web_cfg = config.clone(); // AgentConfig derives Clone
                let web_vault_keys = vault.as_ref()
                    .map(|v| v.list_keys())
                    .unwrap_or_default();
                tokio::spawn(async move {
                    println!("[+] Web server starting on port 9000...");
                    if let Err(e) = web::run_web_server(9000, web_cfg, web_vault_keys).await {
                        eprintln!("[!] Web server error: {}", e);
                    }
                });
            }

            match bot_token {
                Some(token) => {
                    println!("[+] Telegram bot enabled");
                    tokio::spawn(async move {
                        telegram::run_telegram_bot(token, config).await;
                    });
                }
                None => {
                    println!("[!] No Telegram token found in vault or env - running idle");
                }
            }
            println!("[+] Daemon running (Ctrl+C to stop)");
            tokio::signal::ctrl_c().await?;
            println!("Daemon stopped");
        }
    }

    Ok(())
}

fn handle_vault_command(vault: &mut SecureVault, action: VaultAction) -> anyhow::Result<()> {
    match action {
        VaultAction::Set { key, value } => {
            vault.store(&key, &value)
                .map_err(|e| anyhow::anyhow!("Failed to store credential: {}", e))?;
            println!("[+] Stored: {}", key);
        }
        VaultAction::Get { key } => {
            match vault.retrieve(&key) {
                Ok(value) => println!("{}", value),
                Err(VaultError::NotFound(_)) => {
                    eprintln!("[-] Key not found: {}", key);
                    std::process::exit(1);
                }
                Err(e) => return Err(anyhow::anyhow!("Failed to retrieve: {}", e)),
            }
        }
        VaultAction::List => {
            let keys = vault.list_keys();
            if keys.is_empty() {
                println!("No credentials stored.");
            } else {
                println!("Stored credentials:");
                for key in keys {
                    println!("  - {}", key);
                }
            }
        }
        VaultAction::Delete { key } => {
            vault.delete(&key)
                .map_err(|e| anyhow::anyhow!("Failed to delete: {}", e))?;
            println!("[-] Deleted: {}", key);
        }
    }
    Ok(())
}
