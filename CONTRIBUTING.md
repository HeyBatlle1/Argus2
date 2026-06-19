# Contributing to Argus

First: thank you. Security is a team sport.

## Ground Rules

### Security First
- Every PR gets a security review
- No plaintext secrets, ever
- If you're unsure whether something is secure, ask

### Code Quality
- All code must pass `cargo clippy` with no warnings
- All code must be formatted with `cargo fmt`
- New features need tests
- Public APIs need documentation

## What We're Looking For

### High Priority
- Post-quantum crypto implementation (ML-KEM, ML-DSA)
- WebAssembly sandbox hardening
- Prompt injection detection
- Audit logging

### Medium Priority
- Additional LLM provider support
- TUI improvements
- Performance optimization
- Documentation

### Always Welcome
- Security vulnerability reports (see SECURITY.md)
- Bug fixes
- Test coverage improvements
- Documentation improvements

## Development Setup

```bash
# Clone
git clone https://github.com/burtonstuff/argus
cd argus

# Build
cargo build

# Test
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt

# Build release
cargo build --release
```

## Pull Request Process

1. Fork the repo
2. Create a feature branch (`git checkout -b feature/amazing-thing`)
3. Make your changes
4. Run `cargo test && cargo clippy && cargo fmt`
5. Commit with a clear message
6. Push and open a PR
7. Wait for review (we're thorough, not slow)

## Commit Messages

```
type(scope): description

- type: feat, fix, docs, style, refactor, test, chore
- scope: crypto, sandbox, memory, core, cli
- description: imperative mood, lowercase, no period

Examples:
feat(crypto): implement ML-KEM key encapsulation
fix(sandbox): prevent wasm memory escape via bounds check
docs(readme): add installation instructions
```

## Security Vulnerabilities

Found a vulnerability? **Do not open a public issue.**

Email security@[domain].com with:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if you have one)

We'll acknowledge within 24 hours and work with you on a fix.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

*"In a world of one-click RCEs, be the hundred eyes."*
