# Setting Up Argus

## Prerequisites

- Rust toolchain ([rustup.rs](https://rustup.rs))
- Docker and Docker Compose
- macOS (Keychain), Linux (Secret Service), or Windows (Credential Manager)
- A Supabase account (free tier works) — optional but recommended
- OpenRouter API key ([openrouter.ai](https://openrouter.ai))

## Required API Keys

Get these before starting:

| Key | Required | Purpose |
|-----|----------|---------|
| OpenRouter | Yes | All LLM calls (Claude, Grok, Gemini) |
| Brave Search | Optional | `web_search` tool |
| Telegram Bot | Optional | Mobile interface + shell approval |
| Supabase URL + service key | Optional | Semantic memory, skill persistence, agent discourse |

---

## Quick Start

### 1. Clone and build

```bash
git clone https://github.com/HeyBatlle1/Argus2.git
cd Argus2
cargo build --release
```

### 2. Initialize the vault

```bash
./target/release/argus init
```

This creates the encrypted vault backed by your system keychain. No plaintext secrets on disk.

### 3. Add your API keys

```bash
./target/release/argus vault set openrouter_api_key YOUR_KEY
./target/release/argus vault set brave_search_api_key YOUR_KEY   # optional
./target/release/argus vault set telegram_bot_token YOUR_TOKEN    # optional
./target/release/argus vault set telegram_chat_id YOUR_CHAT_ID    # optional
```

### 4. Configure Supabase (optional but recommended)

Supabase enables semantic memory (pgvector), the skill system, and agent discourse.

1. Create a new Supabase project at [supabase.com](https://supabase.com)
2. Run the schema in `docs/SUPABASE_SCHEMA.sql` in the Supabase SQL editor
3. Add your credentials to the vault:

```bash
./target/release/argus vault set supabase_url https://YOUR_PROJECT_ID.supabase.co
./target/release/argus vault set supabase_service_key YOUR_SERVICE_ROLE_KEY
```

### 5. Launch

```bash
./argus-up.sh
```

Or run individual interfaces:

```bash
# Interactive TUI
./target/release/argus chat

# WebSocket server (powers the web frontend)
./target/release/argus web

# Telegram bot
./target/release/argus telegram

# Discord bot
./target/release/argus discord
```

### 6. Web frontend (optional)

```bash
cd frontend
npm install
NEXT_PUBLIC_WS_URL=ws://localhost:9000/ws npm run dev
```

Access at `http://localhost:3000`.

---

## Docker (recommended for production)

```bash
docker compose up -d
```

Three containers:
- `argus` — the Rust agent runtime
- `frontend` — Next.js web UI
- `db` — local SQLite proxy (Supabase handles persistent memory)

---

## Verify it works

```bash
# Check vault
./target/release/argus vault list

# Run a quick agent turn
./target/release/argus chat
# > hi
```

Expected: the agent responds, eye state transitions to `watching`, tools are available.

---

## Troubleshooting

**Vault unlock fails**: The vault is tied to your system keychain. On macOS, you may need to approve keychain access the first time.

**OpenRouter 401**: Double-check your key with `argus vault get openrouter_api_key`. Keys are stored encrypted — there's no plaintext `.env` to accidentally expose.

**Supabase connection refused**: Confirm the service role key (not the anon key) is in the vault. The service role key starts with `eyJ...` and is found in Project Settings → API.

**WASM sandbox errors**: The sandbox requires a modern CPU with SIMD support. Most machines from 2018+ work fine.
