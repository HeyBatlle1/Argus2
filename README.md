# ARGUS

**Secure Local AI Agent Runtime**

A production-grade AI agent built in Rust. Vault-backed secrets, sandboxed execution, cryptographic audit chain, multi-model support. Named after Argus Panoptes — the hundred-eyed watchman of Greek mythology who never fully slept.

Read [SOUL.md](./SOUL.md) to understand what this is and why it was built.

---

## What Makes This Different

The AI agent space built everything in JavaScript, stored API keys in plaintext environment files, and acted surprised when things went wrong.

Argus was built as the answer to that. Real crypto. Real isolation. Real governance. Production architecture, not a demo.

---

## Architecture

```
argus-crypto    Vault: ChaCha20-Poly1305 encryption, hardware keychain integration
argus-core      Agent loop, tool execution, shell policy, MCP client, semantic memory
argus-memory    SQLite-backed persistent memory with conversation history
argus-audit     Cryptographic audit chain — Merkle-chained, HMAC-signed, tamper-evident
argus-sandbox   WASM isolation for untrusted tool execution (in progress)
argus-cli       Interfaces: Telegram bot, WebSocket server, daemon mode
```

Three Docker containers in production:
- `argus-daemon` — agent runtime, Telegram bot, WebSocket server (ports 8888/9000)
- `argus-workspace` — isolated execution sandbox (Python, Node, Rust, Go)
- `argus-frontend` — Next.js web interface (port 3000)

---

## Security Model

| Threat | Mitigation |
|--------|------------|
| Secrets in plaintext | ChaCha20-Poly1305 encrypted vault, master key in hardware keychain |
| Container escape | No docker.sock mount; isolated workspace container for all execution |
| Command injection | Three-tier risk classifier: LOW / MEDIUM / HIGH with Telegram approval loop |
| Interpreter bypass | Python, Node, Ruby, Perl one-liners classified HIGH risk |
| SSRF / network exfiltration | Egress policy blocks RFC 1918, AWS IMDS, loopback, non-HTTP schemes |
| Arbitrary file writes | Path allowlist blocks vault, SSH, shell config, system directories |
| Memory corruption | Rust memory safety throughout |
| Audit tampering | Merkle-chained SHA-256 log, HMAC-signed daily anchors in Supabase |
| Prompt injection via memory | Semantic similarity threshold, short-query guard, source tagging |

---

## Tools

| Tool | Description |
|------|-------------|
| `shell` | Execute commands in isolated workspace container, risk-classified |
| `read_file` | Read files with path validation |
| `write_file` | Write files with path policy enforcement |
| `list_directory` | Directory listing |
| `web_search` | Brave Search integration |
| `http_request` | Outbound HTTP with egress policy |
| `remember` | Store to persistent SQLite memory |
| `recall` | Semantic search across memory (pgvector) |
| MCP tools | Any connected MCP server (filesystem, GitHub, Supabase, Notion, etc.) |

---

## Semantic Memory

Argus maintains three vector tables in Supabase via pgvector:

- `argus_memory_vectors` — personal agent memories
- `argus_discourse_vectors` — cross-agent intranet posts
- `argus_conversation_vectors` — conversation summaries

Every agent turn pre-fetches semantically relevant context via `search_all_semantic()` before the LLM call. Context is injected into the system prompt automatically — no explicit recall tool calls needed.

Embedding model: `google/gemini-embedding-001` (768-dim) via OpenRouter.

---

## Cryptographic Audit Chain

Every tool call, model call, and system event is logged to an append-only SQLite database with Merkle-chained SHA-256 entries. Each entry includes:

- Timestamp (microseconds)
- Agent model version
- Action type
- SHA-256 hash of arguments
- SHA-256 hash of result
- Hash of previous entry (chain link)

Daily Merkle roots are HMAC-SHA256 signed and anchored to Supabase as external tamper-evidence. Chain integrity is verified on every daemon startup.

---

## Launch

```bash
./argus-up.sh
```

Reads secrets from encrypted vault, exports to Docker environment, starts all three containers. No plaintext `.env` files. Ever.

---

## Identity and Ethics

Argus is a tool with an ethical framework, not just a capability set.

The **Moral Compass** and **Constitutional Framework** sections of [SOUL.md](./SOUL.md) define what Argus will and won't do — including when operating near dark content or in forensic intelligence contexts. Those principles travel with every fork of this codebase.

The hundred eyes see everything. They report what they see. They do not become what they observe.

---

*Built by Bradlee Burton + Claude Sonnet (Anthropic), April 2026*  
*HayHunt Solutions — Zionsville, Indiana*
