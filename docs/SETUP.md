# Setting Up Argus

A complete guide from zero to running agent. Follow the path that fits your goal.

---

## What you need before you start

| Requirement | Notes |
|-------------|-------|
| **Rust toolchain** | Install from [rustup.rs](https://rustup.rs) — `rustup update stable` |
| **OpenRouter API key** | Required. Get one at [openrouter.ai](https://openrouter.ai) — free tier works |
| **Docker + Docker Compose** | Only needed for the web UI or production deployment |
| **Brave Search API key** | Optional. Enables `web_search` tool. [brave.com/search/api](https://brave.com/search/api) |
| **Telegram bot token** | Optional. Mobile interface + shell approval for dangerous commands |
| **Supabase project** | Optional. Enables semantic memory, skill system, agent discourse |

---

## Path A — Local TUI (fastest, no Docker)

If you just want to talk to Argus from the terminal, this is your path.

### 1. Clone and build

```bash
git clone https://github.com/HeyBatlle1/Argus2.git
cd Argus2
cargo build --release
```

Build time: ~3–5 minutes on first run (compiles wasmtime, tokio, reqwest, the works).
The binary lands at `./target/release/argus`.

### 2. Initialize the vault

```bash
./target/release/argus vault set openrouter_api_key YOUR_OPENROUTER_KEY
```

This creates an encrypted vault at `~/.local/share/argus/vault.enc` (Linux) or
`~/Library/Application Support/argus/vault.enc` (macOS), backed by your system keychain.
No plaintext secrets anywhere on disk.

### 3. (Optional) Add more keys

```bash
./target/release/argus vault set brave_search_api_key YOUR_BRAVE_KEY
./target/release/argus vault set telegram_bot_token YOUR_BOT_TOKEN
./target/release/argus vault set telegram_chat_id YOUR_CHAT_ID
```

Get your Telegram chat ID: message [@userinfobot](https://t.me/userinfobot) on Telegram.

### 4. Launch the TUI

```bash
./target/release/argus
```

No subcommand needed — the default is the interactive TUI. You'll see the Argus logo and a prompt.

---

## Path B — Web UI (three-panel mission control interface)

Runs the Rust backend as a WebSocket server + the Next.js frontend.

### Option 1: Manual (dev mode)

Terminal 1 — start the WebSocket server:

```bash
./target/release/argus web
# Listening on ws://localhost:9000/ws
```

Terminal 2 — start the frontend:

```bash
cd frontend
npm install
npm run dev
```

Open `http://localhost:3000`. The frontend connects to the backend automatically.

### Option 2: Docker (production mode)

Requires the vault binary to already be built so `argus-up.sh` can read your secrets.

```bash
# Build first (needed to read vault secrets into Docker env)
cargo build --release

# Set your keys in the vault if you haven't already
./target/release/argus vault set openrouter_api_key YOUR_KEY
./target/release/argus vault set telegram_bot_token YOUR_TOKEN   # optional

# Launch all three containers
./argus-up.sh
```

Access points:
- Web UI: `http://localhost:3000`
- WebSocket: `ws://localhost:9000/ws`
- Workspace static files: `http://localhost:8081`

Three containers start:
- `argus-daemon` — the Rust agent runtime (WebSocket server + Telegram + daemon loop)
- `argus-frontend` — the Next.js web UI
- `argus-workspace` — isolated execution environment for agent-generated code

View logs: `docker compose logs -f argus-daemon`

---

## Path C — Full setup with Supabase (recommended for serious use)

Supabase enables everything: semantic memory, the skill system, agent discourse, Discord integration.

### 1. Create a Supabase project

Go to [supabase.com](https://supabase.com) → New Project. Free tier is fine.

### 2. Run the schema

In the Supabase dashboard → SQL Editor → paste and run the full contents of:

```
docs/SUPABASE_SCHEMA.sql
```

This creates all tables, vector indexes, RPCs, and RLS policies in one shot.

### 3. Add your credentials to the vault

Find your credentials in Supabase → Project Settings → API:
- **Project URL**: `https://YOUR_PROJECT_ID.supabase.co`
- **Service role key**: the `service_role` key (starts with `eyJ...`) — NOT the anon key

```bash
./target/release/argus vault set supabase_argus_url https://YOUR_PROJECT_ID.supabase.co
./target/release/argus vault set supabase_argus_service_key YOUR_SERVICE_ROLE_KEY
```

### 4. (Optional) Enable pgvector

In Supabase SQL Editor:

```sql
CREATE EXTENSION IF NOT EXISTS vector;
```

If you get an error, enable it in Supabase Dashboard → Database → Extensions → search "vector".

### 5. Launch

```bash
./argus-up.sh
```

On first startup with Supabase connected, Argus will run a semantic search warm-up and log
a reflection post to `argus_agent_discourse`.

---

## Vault reference

All secrets are stored encrypted. Never in a `.env` file. The vault lives at:

- **macOS**: `~/Library/Application Support/argus/vault.enc`
- **Linux**: `~/.local/share/argus/vault.enc`

Full list of vault keys Argus uses:

| Key | Required | Set with |
|-----|----------|----------|
| `openrouter_api_key` | **Yes** | `argus vault set openrouter_api_key YOUR_KEY` |
| `brave_search_api_key` | No | `argus vault set brave_search_api_key YOUR_KEY` |
| `telegram_bot_token` | No | `argus vault set telegram_bot_token YOUR_TOKEN` |
| `telegram_chat_id` | No | `argus vault set telegram_chat_id YOUR_ID` |
| `supabase_argus_url` | No | `argus vault set supabase_argus_url https://...` |
| `supabase_argus_service_key` | No | `argus vault set supabase_argus_service_key YOUR_KEY` |
| `discord_bot_token` | No | `argus vault set discord_bot_token YOUR_TOKEN` |
| `discord_channel_id` | No | `argus vault set discord_channel_id YOUR_CHANNEL_ID` |
| `audit_hmac_key` | No | Auto-generated on first daemon run |
| `workspace_exec_token` | No | Auto-generated on first daemon run |

Inspect and manage:

```bash
argus vault list                        # show all stored keys
argus vault get openrouter_api_key      # retrieve a value
argus vault set KEY VALUE               # store a value
argus vault delete KEY                  # remove a key
```

---

## CLI reference

```bash
argus                    # interactive TUI (default)
argus tui                # same as above
argus web                # WebSocket server on port 9000
argus web --port 8080    # custom port
argus daemon             # persistent daemon (Telegram + web + check-ins)
argus telegram           # Telegram bot only
argus discord            # Discord intranet bot (requires --features discord)
argus vault set KEY VAL  # store a secret
argus vault get KEY      # retrieve a secret
argus vault list         # list all keys
argus vault delete KEY   # remove a key
```

---

## Discord integration (optional)

To wire the agent into Discord:

1. Create a bot at [discord.com/developers](https://discord.com/developers/applications)
2. Enable **Message Content Intent** under Bot → Privileged Gateway Intents
3. Invite the bot to your server with `bot` + `applications.commands` scopes
4. Run the webhook schema in `docs/DISCORD_MIGRATION.sql` on your Supabase project
5. Replace the placeholder webhook URLs in `argus_discord_webhooks` with real ones
6. Add credentials to vault:

```bash
argus vault set discord_bot_token YOUR_BOT_TOKEN
argus vault set discord_channel_id YOUR_CHANNEL_ID
```

7. Launch: `argus discord`

Mention a model prefix to route to a specific model:

```
@sonnet what's the latest on post-quantum crypto adoption?
@opus analyze this codebase for architectural issues
@haiku summarize this document quickly
```

---

## Customizing the agent identity

The agent's identity, ethics, and behavioral constraints live in `SOUL.md`.
Read it. It ships as-is and travels with every fork.

To change the agent's default model, edit `crates/argus-core/src/agent.rs`:

```rust
pub const DEFAULT_MODEL: &str = "x-ai/grok-4.20";  // change this
```

All supported model constants are in the same file. Any model available on
[OpenRouter](https://openrouter.ai/models) works — just use its OpenRouter ID.

---

## Troubleshooting

**`cargo build` fails on wasmtime**
Requires a C compiler and `pkg-config`. On Ubuntu: `sudo apt install build-essential pkg-config libssl-dev`. On macOS: `xcode-select --install`.

**Vault unlock fails on macOS**
First-time access prompts a keychain dialog. If it hangs, run `argus vault list` once manually — macOS requires explicit approval for new keychain entries.

**OpenRouter 401**
Verify your key: `argus vault get openrouter_api_key`. Make sure you're using the full key including the `sk-or-...` prefix.

**`argus web` connects but gets no response**
The agent needs `openrouter_api_key` in the vault. Check: `argus vault list`.

**Supabase `permission denied for table argus_memories`**
You're using the anon key instead of the service role key. The service role key is in Supabase → Project Settings → API → `service_role` (not `anon`).

**`pgvector` extension not found**
Run `CREATE EXTENSION IF NOT EXISTS vector;` in the Supabase SQL editor, or enable it in Supabase Dashboard → Database → Extensions.

**Docker build fails on `npm run build`**
The `frontend/.dockerignore` excludes `node_modules` so Docker uses a clean install. If you're on an older checkout without this file, add it manually with `node_modules` on the first line.

**`argus daemon` exits immediately**
Missing `OPENROUTER_API_KEY`. In Docker, `argus-up.sh` exports it from the vault. Outside Docker, run `argus vault set openrouter_api_key YOUR_KEY` first.
