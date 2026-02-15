//! Argus CLI - The Hundred-Eyed Agent
//!
//! Main entrypoint for the Argus command-line interface.

mod telegram;
mod tui;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use argus_crypto::{SecureVault, vault::VaultError};

const LOGO: &str = r#"
    ___    ____  ______  __  _______
   /   |  / __ \/ ____/ / / / / ___/
  / /| | / /_/ / / __/ / / / /\__ \
 / ___ |/ _, _/ /_/ / / /_/ /___/ /
/_/  |_/_/ |_|\____/  \____//____/

    THE HUNDRED-EYED AGENT
    Autonomous • Encrypted • Local
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
        /// OpenRouter API key (or use vault)
        #[arg(short, long)]
        api_key: Option<String>,
    },

    /// Run Telegram bot
    Telegram {
        /// Telegram bot token (or use vault)
        #[arg(short, long)]
        token: Option<String>,
    },

    /// Manage secure credential vault
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },

    /// Run in daemon mode (always-on, Telegram bot)
    Daemon,
}

#[derive(Subcommand)]
enum VaultAction {
    /// Store a credential
    Set {
        /// Credential key
        key: String,
        /// Credential value
        value: String,
    },
    /// Retrieve a credential
    Get {
        /// Credential key
        key: String,
    },
    /// List all credential keys (not values)
    List,
    /// Delete a credential
    Delete {
        /// Credential key
        key: String,
    },
}

/// Get the default vault path
fn vault_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("argus")
        .join("vault.enc")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI args
    let cli = Cli::parse();

    // Initialize vault
    let vault_file = vault_path();
    if let Some(parent) = vault_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut vault = if vault_file.exists() {
        let mut v = SecureVault::new(vault_file.clone());
        v.unlock()?;
        v
    } else {
        SecureVault::init(vault_file.clone())?
    };

    match cli.command {
        Some(Commands::Vault { action }) => {
            handle_vault_command(&mut vault, action)?;
        }
        Some(Commands::Tui { api_key }) => {
            let key = if let Some(k) = api_key {
                k
            } else {
                vault.retrieve("openrouter_api_key")
                    .map_err(|e| anyhow::anyhow!("OpenRouter API key not found. Set it with: argus vault set openrouter_api_key YOUR_KEY\nError: {}", e))?
            };

            println!("{}", LOGO);
            tui::run_tui(key).await?;
        }
        None => {
            let key = vault.retrieve("openrouter_api_key")
                .map_err(|e| anyhow::anyhow!("OpenRouter API key not found. Set it with: argus vault set openrouter_api_key YOUR_KEY\nError: {}", e))?;

            println!("{}", LOGO);
            tui::run_tui(key).await?;
        }
        Some(Commands::Telegram { token }) => {
            let bot_token = if let Some(t) = token {
                t
            } else {
                vault.retrieve("telegram_bot_token")
                    .map_err(|e| anyhow::anyhow!("Failed to get telegram token from vault: {}. Use --token or store in vault with: argus vault set telegram_bot_token YOUR_TOKEN", e))?
            };

            let openrouter_api_key = vault.retrieve("openrouter_api_key")
                .map_err(|e| anyhow::anyhow!("Failed to get OpenRouter API key from vault: {}", e))?;

            telegram::run_telegram_bot(bot_token, openrouter_api_key).await;
        }
        Some(Commands::Daemon) => {
            println!("🔴 Argus daemon starting...");

            // Try vault first, then fall back to env vars (for Docker)
            let bot_token = vault.retrieve("telegram_bot_token")
                .ok()
                .or_else(|| std::env::var("TELEGRAM_BOT_TOKEN").ok());

            let api_key = vault.retrieve("openrouter_api_key")
                .ok()
                .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
                .ok_or_else(|| anyhow::anyhow!("No OpenRouter API key found in vault or OPENROUTER_API_KEY env var"))?;

            match bot_token {
                Some(token) => {
                    println!("✓ Telegram bot enabled");
                    println!("✓ Daemon running (Ctrl+C to stop)");
                    telegram::run_telegram_bot(token, api_key).await;
                }
                None => {
                    println!("⚠ No telegram_bot_token in vault or TELEGRAM_BOT_TOKEN env var");
                    println!("✓ Daemon running idle (Ctrl+C to stop)");

                    use tokio::signal;
                    signal::ctrl_c().await?;
                    println!("\n👋 Daemon stopped");
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
            println!("✓ Stored credential: {}", key);
        }
        VaultAction::Get { key } => {
            match vault.retrieve(&key) {
                Ok(value) => println!("{}", value),
                Err(VaultError::NotFound(_)) => {
                    eprintln!("✗ Key not found: {}", key);
                    std::process::exit(1);
                }
                Err(e) => return Err(anyhow::anyhow!("Failed to retrieve credential: {}", e)),
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
                .map_err(|e| anyhow::anyhow!("Failed to delete credential: {}", e))?;
            println!("✓ Deleted credential: {}", key);
        }
    }
    Ok(())
}
