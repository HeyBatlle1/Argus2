//! Argus CLI - The Hundred-Eyed Agent

mod telegram;
mod tui;

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
        None
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

        Some(Commands::Daemon) => {
            println!("Argus daemon starting...");
            let api_key = std::env::var("OPENROUTER_API_KEY")
                .map_err(|_| anyhow::anyhow!("OPENROUTER_API_KEY env var not set"))?;
            let mut config = AgentConfig::new(api_key);
            if let Ok(brave_key) = std::env::var("BRAVE_SEARCH_API_KEY") {
                config.brave_search_key = Some(brave_key);
            }
            let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").ok();
            match bot_token {
                Some(token) => {
                    println!("[+] Telegram bot enabled");
                    telegram::run_telegram_bot(token, config).await;
                }
                None => {
                    println!("[!] No TELEGRAM_BOT_TOKEN set - running idle");
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
