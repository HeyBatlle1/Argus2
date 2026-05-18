# Contributing to Argus

Thanks for your interest. Argus is a security-focused project — contributions are welcome but the security model is non-negotiable. Read this before opening a PR.

---

## The One Rule

The security architecture is the project. If a contribution weakens the vault, the audit chain, the workspace isolation, or the shell risk classifier — it won't be merged regardless of how useful the feature is. Security first, features second.

---

## Development Setup

### Prerequisites

- Rust toolchain: [rustup.rs](https://rustup.rs)
- Docker and Docker Compose
- A Supabase account (free tier works for development)
- An OpenRouter API key ([openrouter.ai](https://openrouter.ai))

### Clone and build

```bash
git clone https://github.com/HeyBatlle1/Argus2.git
cd Argus2
cargo build
```

### Initialize the vault

The vault is created automatically on first use. Just set your key:

```bash
./target/debug/argus vault set openrouter_api_key YOUR_KEY
```

### Run tests

```bash
cargo test
```

All tests must pass before submitting a PR.

### Launch the stack

```bash
./argus-up.sh
```

This reads from the vault, injects secrets into Docker, and starts all three containers. Never use `docker compose up` directly — it bypasses vault injection and the daemon will crash-loop with empty credentials.

---

## Crate Structure

```
argus-crypto    Secrets vault — touch with caution
argus-core      Agent loop, tools, shell policy — most feature work happens here
argus-memory    SQLite + Supabase pgvector — memory and skill system
argus-audit     Cryptographic audit chain — do not break chain integrity
argus-sandbox   WASM isolation via wasmtime
argus-cli       Telegram bot, WebSocket server, daemon entrypoint
```

---

## Good First Issues

Look for issues tagged [`good first issue`](https://github.com/HeyBatlle1/Argus2/labels/good%20first%20issue). These are scoped, well-defined, and won't require deep knowledge of the full architecture.

The `.unwrap()` cleanup issue is the best starting point for Rust developers who want to understand how the codebase is structured.

---

## Commit Convention

Use imperative mood, present tense:

```
feat: add Ollama provider for local model support
fix: replace unwrap() in tools.rs network error path
docs: update SETUP.md with Linux keychain instructions
refactor: collapse three audit chain mutexes into ChainState
```

Prefix options: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

---

## Pull Request Process

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Run `cargo test` — all tests must pass
4. Run `cargo clippy` — no new warnings
5. Update documentation if your change affects behavior
6. Open a PR with a clear description of what changed and why

Small, focused PRs are preferred over large ones. One thing at a time.

---

## Security Issues

Do not open public GitHub issues for security vulnerabilities. If you find a genuine security problem — especially anything related to vault security, sandbox escape, or audit chain integrity — contact HayHunt Solutions directly before disclosing publicly.

---

## What We're Looking For

- Safe error propagation (replacing `.unwrap()` calls)
- Provider support (Ollama, local models)
- Platform support (Linux keychain fallback, Windows testing)
- Documentation improvements
- Test coverage

## What We're Not Looking For Right Now

- Breaking changes to the vault format
- Alternative secret storage backends that reduce security
- Features that bypass the shell risk classifier
- Unaudited dependencies

---

*Argus — Built in Rust. Ferris stays locked in.*
*HayHunt Solutions, 2026*
