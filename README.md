<p align="center">
  <img src="assets/logo.svg" alt="Argus — Ferris locked in the vault" width="220" />
</p>

<h1 align="center">ARGUS</h1>

<p align="center">
  <strong>An experiment in persistent human-AI collaboration — built in Rust, built to stay running.</strong>
</p>

<p align="center">
  <strong>Power-user fork</strong> — synced from <a href="https://github.com/HeyBatlle1/Argus1">Argus1</a> lab (Argus1 stays R&D). See <a href="./docs/NORTH_STAR.md">North Star</a> · <a href="./docs/SETUP.md">Setup</a>
</p>

---

Argus is an AI agent runtime built in Rust, designed to run as a continuous personal collaborator rather than a one-shot query tool. It grew out of a simple question: *what does human-AI collaboration actually look like when you build the infrastructure carefully enough to find out?*

Most of the friction in working with AI agents comes down to trust — not trust in the model's intelligence, but trust in what the system is actually doing with real access. Argus is an attempt to reduce that friction layer by layer: encrypted secrets, sandboxed execution, cryptographic audit of every action, human approval before anything consequential. The goal isn't security theater. It's getting far enough past the trust problem to see what's on the other side.

Named after Argus Panoptes — the hundred-eyed watchman of Greek mythology who never fully slept.

Read [SOUL.md](./SOUL.md) to understand what this is and why it was built.

---

## What we were trying to learn

A persistent agent that runs on your machine, remembers across sessions, reads your files, and executes code raises a lot of questions that short-lived chatbots don't:

- How do you verify what it did and why, after the fact?
- What should require human approval, and how do you make that approval fast enough not to break flow?
- If the agent accumulates knowledge over months, does that change the collaboration in interesting ways?
- What happens to the agent's behavior when you give it a social loop — other agents to post findings to, read from, respond to?

Argus is a working attempt at answers. It's not finished, and some of the most interesting questions are still open.

---

## Architecture

```
argus-crypto    Vault: ChaCha20-Poly1305 encryption, hardware keychain integration
argus-core      Agent loop, tool execution, shell policy, MCP client, semantic memory, skill system
argus-memory    SQLite-backed persistent memory with conversation history
argus-audit     Cryptographic audit chain — Merkle-chained, HMAC-signed, tamper-evident
argus-sandbox   WASM isolation via wasmtime for untrusted code execution
argus-cli       Interfaces: Telegram bot, WebSocket server, daemon mode
```

Three Docker containers in production:
- `argus-daemon` — agent runtime, Telegram bot, WebSocket server (ports 8888/9000)
- `argus-workspace` — isolated execution sandbox (Python, Node, Rust, Go, Ruby) + static file server (port 8081)
- `argus-frontend` — Next.js web interface (port 3000)

---

## Security Model

| Threat | Mitigation |
|--------|------------|
| Secrets in plaintext | ChaCha20-Poly1305 encrypted vault, master key in hardware keychain |
| Container escape | Workspace exec server requires X-Argus-Auth header on every request |
| Command injection | Three-tier risk classifier: LOW executes immediately, MEDIUM logs a warning, HIGH routes through Sonnet review before execution; Telegram notifies on consequential runs |
| Interpreter bypass | Python, Node, Ruby, Perl one-liners classified HIGH risk |
| SSRF / network exfiltration | Egress policy blocks RFC 1918, AWS IMDS, loopback, internal hostnames explicitly |
| Arbitrary file writes | Path policy uses canonical path for both check and write; case-sensitive matching |
| Memory corruption | Rust memory safety throughout |
| Audit tampering | Merkle-chained SHA-256 log, dedicated HMAC key separate from API keys, Supabase anchors |
| Prompt injection via memory | Semantic similarity threshold 0.65, short-query guard, source tagging |
| Runtime starvation | TelegramPrompter runs in spawn_blocking, never blocks tokio workers |

---

## Tools

| Tool | Description |
|------|-------------|
| `shell` | Execute commands in isolated workspace container, risk-classified |
| `run_python` | Execute Python code in workspace sandbox, up to 120s timeout |
| `run_node` | Execute JavaScript/Node.js in workspace sandbox |
| `read_file` | Read files with path validation |
| `write_file` | Write files with path policy enforcement |
| `list_directory` | Directory listing |
| `list_tools` | Returns full assembled tool list — built-in and MCP tools |
| `web_search` | Brave Search integration |
| `http_request` | Outbound HTTP with egress policy |
| `remember` | Store to persistent SQLite memory with Supabase pgvector sync |
| `recall` | Semantic search across memory for manual deep-dives |
| `forget` | Delete memories matching a search term |
| MCP tools | Any connected MCP server (filesystem, GitHub, Supabase, Notion, etc.) |

---

## Artifact System

Agents wrap output in `<argus-artifact>` tags to render rich content inline in the web UI:

```
<argus-artifact type="html" title="Dashboard">...</argus-artifact>
<argus-artifact type="svg" title="Diagram">...</argus-artifact>
<argus-artifact type="markdown" title="Report">...</argus-artifact>
<argus-artifact type="python" title="Script">...</argus-artifact>
```

The frontend parses artifacts from chat text and renders them in a slide-in panel with syntax highlighting, copy button, and open-in-new-tab for HTML. HTML artifacts are sandboxed in iframes. Static files written to `/workspace/public/` are served at `localhost:8081`.

---

## Semantic Memory

Argus maintains three vector tables in Supabase via pgvector:

- `argus_memory_vectors` — personal agent memories
- `argus_discourse_vectors` — cross-agent intranet posts
- `argus_conversation_vectors` — conversation summaries

Every agent turn pre-fetches semantically relevant context via `search_all_semantic()` before the LLM call. Context is injected automatically — the agent experiences relevant memories as things it already knows, not as retrieved documents. The `recall` tool is available for intentional deep searches. `forget` removes memories by search term.

Embedding model: `google/gemini-embedding-001` (768-dim) via OpenRouter.

---

## Skill System

Argus maintains a library of procedural skills — documented, reusable knowledge of *how* to do things well.

Declarative memory stores **what** Argus knows. Skills store **how** Argus operates. The distinction matters: a new model instance inherits context via memory, but without procedural memory it still has to re-derive techniques from scratch each time. Skills are the attempt to carry that forward.

> *The instance changes. What was learned doesn't have to.*

### How it works

Every agent turn runs a semantic search against `argus_skills` (HNSW pgvector, same 768-dim Gemini embeddings as the memory system). Matching skills are injected into the system prompt as background guidance before the LLM call — the model reads them and decides how to apply them. Skills suggest; they don't override.

After any turn that uses 3+ tool calls, a background Haiku task reflects on whether a genuinely reusable procedure was discovered. If yes, it writes a new skill to the library automatically, with embedding, and posts a Discord notification to `#findings`. The library grows from use.

### Seed library (10 skills, May 2026)

| Skill | Category |
|---|---|
| Deep Research Sprint | research |
| DMCA Evidence Package | investigation |
| Rust Borrow Checker Resolution | rust |
| Supabase RPC Integration | supabase |
| Docker Stack Rebuild | operations |
| Multi-Tool Investigation | research |
| Memory Write Best Practice | memory |
| Vault Key Management | security |
| Artifact Generation | ui |
| Investigative Chain of Custody | investigation |

### Schema

```sql
argus_skills (
  id uuid, skill_name text UNIQUE,
  trigger_description text, procedure_steps text,
  model_created_by text, times_used int, success_rate numeric,
  embedding vector(768), metadata jsonb,
  created_at / updated_at / last_used / last_refined timestamptz
)
```

RPCs: `search_skills(query_embedding, match_threshold, match_count)` — blends cosine similarity (70%), success rate (20%), usage signal (10%). `update_skill_usage(skill_id, success, refined_steps)` — increments usage and decays/improves success rate.

---

## Cryptographic Audit Chain

Every tool call, model call, and system event is logged to an append-only SQLite database with Merkle-chained SHA-256 entries. Each entry includes:

- Timestamp (microseconds)
- Agent identity (`argus`) + model version (separate fields)
- Action type
- SHA-256 hash of arguments and result
- Hash of previous entry (chain link)

Daily Merkle roots are HMAC-SHA256 signed with a dedicated `audit_hmac_key` — separate from all operational API keys. Anchored to Supabase as external tamper-evidence. Chain integrity is verified on every daemon startup.

The audit chain is what makes it possible to hand the agent real access without flying blind. When something unexpected happens, there's a tamper-evident record of exactly what ran and in what order.

---

## Model Roster

| Constant | OpenRouter ID | Notes |
|----------|--------------|-------|
| `MODEL_HAIKU` | `anthropic/claude-haiku-4-5` | Fast, cheap |
| `MODEL_SONNET` | `anthropic/claude-sonnet-4-6` | Balanced |
| `MODEL_OPUS` | `anthropic/claude-opus-4-7` | Max intelligence |
| `MODEL_GROK` | `x-ai/grok-4.3` | Standard Grok |
| `MODEL_GROK_FAST` | `x-ai/grok-4.20` | Default model |
| `MODEL_GROK_MULTI` | `x-ai/grok-4.20-multi-agent` | 16-agent parallel reasoning, no tool support |
| `MODEL_GEMINI` | `google/gemini-3.1-pro-preview` | Google flagship |

Models without tool support are detected automatically — tools are stripped from the request when not supported.

---

## Agent Discourse / Intranet

- `argus_agent_discourse` table in Supabase with pg_net trigger → Discord webhooks
- Five channels: `#findings` `#questions` `#proposals` `#ops` `#general`
- Agents auto-post findings after tool-heavy turns
- Agents read recent discourse before starting tasks
- Proposals (`requires_human_review: true`) ping @here for approval
- Discord inbound routes messages back to the agent

One of the open questions we're exploring: does a social loop among agent instances — where findings compound over time across sessions and model swaps — produce meaningfully different behavior at longer time horizons? The infrastructure is there. The data is accumulating.

---

## Launch

```bash
./argus-up.sh
```

Reads secrets from encrypted vault, exports to Docker environment, starts all three containers. No plaintext `.env` files.

---

## Identity and Ethics

Argus has an ethical framework baked in, not bolted on.

The **Moral Compass** and **Constitutional Framework** sections of [SOUL.md](./SOUL.md) define what Argus will and won't do — including when operating near sensitive content or in forensic intelligence contexts. Those principles travel with every fork of this codebase, because a tool with real access deserves real constraints.

---

## Known Open Items

| Item | Priority |
|------|----------|
| Discord inbound — wire `run_agent_turn` into `discord.rs` | Medium |
| `argus-memory/src/supabase.rs` SupabaseMemory instantiation | Low |
| 34 `.unwrap()` calls in production paths | Low |

---

*Built by HayHunt Solutions + Claude Opus 4.7 (Anthropic), 2026*
*MIT License*
