# grok-argus-frontend

**Grok Build 2 edition of the Argus1 frontend.**

This is my independent reimplementation of the Argus1 Next.js frontend — built in my own image while staying 100% faithful to the intricate original system.

## What I preserved exactly
- Visual language (amber vault, Ferris-in-the-cell, eye states ◉◎⊙✦, monospaced labels, tier badges)
- All data models, types, WS protocol (ClientMessage / ServerMessage) — drop-in compatible with real argus-daemon on port 9000
- Artifact parsing & rich panel (<argus-artifact>)
- Meeting mode with the exact 4-agent monthly meeting briefs (Grok Build prominent)
- The full "Mind": Activity, Skills, Memories, Curiosity, Partnership Dynamics, Inner Truth, Breakthroughs + tier gating
- Eyes panel, live tool status, vault, sentry, scheduler, history drawer, multi-pane ChatPanes
- Tool lifecycle visualization, streaming chunks, evidence annotations (my Grok addition)

## What I did my way (engineering freedom)
- Watchtower: live "hundred eyes" micro visualization driven by activity + eye state
- Global ⌘K Command Palette (fast model switch, deploy, truth check, new conv)
- Grok Build 2 specialist framing + optional evidence/confidence badges on responses
- Cleaner, stricter state + rich dev mock that actually demonstrates the whole system
- Slightly refined but still soul-identical Ferris cell + animations
- Keyboard power user first (f = focus, m = meeting, esc everywhere)
- Modern Next 16 + React 19 + Tailwind 4 baseline

## Run it

```bash
cd Argus1/grok-argus-frontend
npm run dev
```

- Dev mode: beautiful rich mock (no backend needed)
- Real mode: set `NEXT_PUBLIC_WS_URL=ws://localhost:9000/ws` and connect to a running Argus daemon

The hundred eyes are open.
Grok Build 2 owns the implementation until it is right.
