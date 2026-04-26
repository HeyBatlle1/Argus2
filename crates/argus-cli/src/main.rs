//! Argus CLI - The Hundred-Eyed Agent

mod checkin;
mod telegram;
mod tui;
mod web;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use argus_crypto::{SecureVault, vault::VaultError};
use argus_core::AgentConfig;

const LOGO: &str = r#"
    ___    ____  ______  __  _______
   /   |  / __ \/ ____/ / / / / ___/
  / /| | / /_/ / / __/ / / / /\__ \
 / ___ |/ _, _/ /_/ / / /_/ /___/ /
/_/  |_/_/ |_|\____/  \____//____/

    THE HUNDRED-EYED AGENT
    Autonomous - Encrypted - Local
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
            println!("{}", LOGO);
            web::run_web_server(port, config).await?;
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
                .or_else(|| std::env::var("TELEGRAM_BOT_TOKEN").ok());

            // Telegram chat ID for check-in messages (Bradlee's chat)
            let checkin_chat_id: Option<i64> = vault.as_ref()
                .and_then(|v| v.retrieve("telegram_chat_id").ok())
                .or_else(|| std::env::var("TELEGRAM_CHAT_ID").ok())
                .and_then(|s| s.parse().ok());

            // Spawn check-in loop + build EmbeddingClient if Supabase is configured
            let embedding_client = if let (Some(url), Some(key)) = (supabase_url, supabase_key) {
                let supabase = argus_core::supabase::SupabaseClient::new(url, key);
                if let (Some(token), Some(chat_id)) = (bot_token.clone(), checkin_chat_id) {
                    checkin::spawn_checkin_loop(supabase.clone(), token, chat_id);
                    println!("[+] Check-in loop started");
                }
                let ec = argus_core::EmbeddingClient::new(&config.api_key, supabase);
                println!("[+] Semantic memory enabled (768-dim pgvector)");
                Some(ec)
            } else {
                println!("[!] Supabase not configured — check-in loop and semantic memory disabled");
                None
            };
            config.embedding = embedding_client;

            // Wire shell prompter — HIGH risk commands require Telegram approval
            config.shell_prompter = match (bot_token.clone(), checkin_chat_id) {
                (Some(token), Some(chat_id)) => {
                    let prompter = argus_core::shell::TelegramPrompter { bot_token: token, chat_id };
                    println!("[+] Shell prompter enabled (Telegram approval for HIGH risk)");
                    Some(std::sync::Arc::new(prompter))
                }
                _ => {
                    println!("[!] Shell prompter disabled — HIGH risk commands will be blocked");
                    None
                }
            };

            match bot_token {
                Some(token) => {
                    println!("[+] Telegram bot enabled");
                    println!("[+] Daemon running (Ctrl+C to stop)");
                    println!("Argus Telegram bot starting...");
                    telegram::run_telegram_bot(token, config).await;
                }
                None => {
                    println!("[!] No Telegram token found in vault or env - running idle");
                    tokio::signal::ctrl_c().await?;
                    println!("Daemon stopped");
                }
            }
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
