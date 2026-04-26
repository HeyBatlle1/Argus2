# Argus Security Audit Log

---

## April 22, 2026 — Opus 4.7 Full Audit

**Auditor:** Claude Opus 4.7 via Claude Code
**Scope:** Full codebase — shell.rs, tools.rs, agent.rs, embedding.rs, supabase.rs, telegram.rs, web.rs, main.rs, docker-compose.yml, argus-up.sh

### Critical findings:
1. `docker.sock` mount — container escape via `docker run` (MEDIUM risk, bypasses host)
2. TelegramPrompter not wired — `execute_shell(policy, command, None)` — prompter unreachable
3. Python/Node not in risk classifier — arbitrary code execution at LOW risk
4. `http_request` zero egress policy — unrestricted exfiltration channel
5. `write_file` zero path policy — arbitrary filesystem writes (deferred)
6. Shell classifier is substring-based — multiple bypass patterns identified
7. Model constants outdated — Opus 4.5 when 4.7 exists
8. Conversation history per-surface inconsistency — Telegram persists, Web doesn't
9. Semantic pre-fetch threshold 0.45 too low — noise injection
10. No self-audit/cryptographic chain — SOUL.md promise unkept

### Category recommendation:
Cryptographic audit chain — Merkle-chained tamper-evident log, signed with vault key, periodically anchored. First agent runtime with verifiable behavior. Category move.

---

## April 26, 2026 — Security Hardening Session 1

**Engineer:** Claude Opus 4.7 via Claude Code
**Directed by:** Bradlee Burton + Claude Sonnet

### Fix 1: docker.sock removal ✅
- Removed `/var/run/docker.sock:/var/run/docker.sock` from docker-compose.yml
- Container escape vector eliminated
- Argus can no longer manage Docker from inside daemon — correct trade

### Fix 2: TelegramPrompter wired ✅
- Added `TelegramPrompter` struct to shell.rs
- Added `shell_prompter: Option<Arc<dyn PermissionPrompter>>` to AgentConfig
- Updated `execute_builtin` signature to accept prompter
- Updated `tool_shell` to pass prompter through to `execute_shell`
- Wired `TelegramPrompter` in main.rs daemon startup when bot_token + chat_id available
- HIGH risk commands now actually send Telegram approval requests
- Governance is now enforced in code, not just in documentation

### Fix 3: Interpreter risk classification ✅
- Added to HIGH: `python -c`, `python3 -c`, `python -m`, `python3 -m`, `node -e`, `node --eval`, `ruby -e`, `perl -e`, `perl -E`, `subprocess`, `os.system(`, `os.popen(`, `exec(`, `eval(`, `git config --global`, `git -c core`
- Added to MEDIUM: `python `, `python3 `, `node `, `ruby `, `perl ` (bare interpreter, running scripts)
- Added to MEDIUM: `base64`
- Bonus fix: pipe-to-shell pattern changed from `curl | bash` to `| bash` / `| sh` — now catches any pipe-to-shell regardless of what precedes the pipe
- 2 new tests added: `python_interpreter_is_high`, `bare_interpreter_is_medium`

### Fix 4: http_request egress policy ✅
- Added `validate_egress_url()` function to tools.rs
- Blocks: non-HTTP schemes, AWS IMDS (169.254.169.254), Google metadata, localhost/127.0.0.1/::1, RFC 1918 private networks (10.x, 172.16-31.x, 192.168.x)
- Called at entry point of `tool_http_request` before any network activity
- Added `url` crate dependency to argus-core/Cargo.toml

### Test results:
```
running 11 tests
shell::tests::argus_self_protection              ok
shell::tests::always_allow_passes_high           ok
shell::tests::always_deny_blocks_high            ok
shell::tests::bare_interpreter_is_medium         ok  (new)
shell::tests::high_risk_commands                 ok
shell::tests::low_risk_commands                  ok
shell::tests::low_risk_passes_without_prompter   ok
shell::tests::medium_risk_commands               ok
shell::tests::no_prompter_blocks_high            ok
shell::tests::python_interpreter_is_high         ok  (new)
shell::tests::subshell_always_high               ok
11 passed, 0 failed
```

### Pre-existing issue (not caused by fixes):
Daemon crashes in Docker with `Restarting (101)` — teloxide panics when `TELEGRAM_BOT_TOKEN` env var is blank. Root cause: vault not wired into Docker entrypoint/compose environment. The vault binary reads keys correctly on host but Docker daemon container starts before vault is unlocked. Fix target: next session.

---

## Remaining Priority Queue

| Priority | Fix | File | Status |
|----------|-----|------|--------|
| P1 | Fix daemon Docker vault wiring | docker-compose.yml + Dockerfile | Pending |
| P2 | write_file path policy (allowlist) | tools.rs | Pending |
| P3 | Cryptographic audit chain | new crate argus-audit | Pending |
| P4 | Model constants update (Opus 4.7, Sonnet 4.6) | agent.rs | Pending |
| P5 | Semantic threshold raise 0.45 → 0.65 | embedding.rs | Pending |
| P6 | Conversation summary embedding pipeline | new module | Pending |
| P7 | Supabase per-agent JWT scoping | supabase.rs | Pending |
| P8 | Workspace isolation (tool calls → docker exec) | tools.rs | Pending |

---

*Log maintained by: Bradlee Burton + Claude Sonnet (Anthropic)*
*Audit by: Claude Opus 4.7*
