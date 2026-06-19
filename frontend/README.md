# Argus Frontend

**Canonical UI for Argus.** Docker builds from this directory (`docker-compose.yml` → `argus-frontend`).

## Dev

```bash
cp .env.example .env.local   # if present
# Set NEXT_PUBLIC_WS_URL=ws://localhost:9000/ws
# Optional dev fallback: NEXT_PUBLIC_WS_TOKEN (prod uses /api/ws-token at runtime)
npm install
npm run dev
```

Without `NEXT_PUBLIC_WS_URL`, the UI runs in mock mode with seeded dev data.

## Layout

| Panel | Role |
|-------|------|
| **Eyes** (left) | Nexus core, signal strip, vault, tools, model selection |
| **Conversation** (center) | Starfield chat atmosphere |
| **Mind** (right) | Memory, knowledge graph, execution flow, schedule |

⌘K opens the command palette.

## Other directories

`grok-argus-frontend/` and `argus-nexus/` are experimental forks — not deployed by `argus-up.sh`.