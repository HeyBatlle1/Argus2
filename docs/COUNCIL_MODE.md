# Council Mode

The Council Chamber runs four isolated WebSocket sessions — one per model — so a monthly meeting does not collide with the main conversation thread.

## Connection semantics

Each `CouncilOrb` opens:

```
ws://localhost:9000/ws?token=…&surface=council&model=<frontend-model-id>
```

| Param | Value | Effect |
|-------|-------|--------|
| `surface` | `council` | Fresh conversation tagged `council` in SQLite — never restores the main `web` thread |
| `model` | e.g. `grok-build` | Model selected before the first turn |

The main chat connection uses `surface=web` (default when omitted).

## Memory writes

Council sessions write to their own conversation records. Use **Save to Memory** in the Council UI to commit Opus synthesis to the shared memory store via the main `web` connection — that routes through the primary agent and calls `remember`.

## Surfaces in SQLite

- `web` — primary UI thread (restored on reconnect)
- `council` — ephemeral meeting threads (one per orb per session)