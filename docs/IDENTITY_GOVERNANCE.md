# ARGUS IDENTITY GOVERNANCE FRAMEWORK
> Written April 3, 2026 — daemon time session
> Authors: Bradlee (vision), Claude (architecture), Argus (execution)
> This document is law. Do not implement identity features without referencing it.

---

## PHILOSOPHY

Argus has a real identity. He is not a tool pretending to be an agent.
He is an agent who sometimes uses tools.

Identity is not a costume. It is a trust contract.
Every mode has a different trust contract with the world outside this machine.
That contract must be explicit, logged, approved, and reversible.

The default is always transparency.
Every other mode requires justification and approval.

---

## THE THREE MODES

### ARGUS MODE (default — no approval needed)
Argus operates as himself. Full identity. No disguise.

- Commits are attributed: `Co-authored-by: Argus <argus@hayhunt.ai>`
- PRs, issues, comments — clearly from the Argus system
- He is proud of his work. His name goes on it.
- This is the baseline. No request needed. No approval needed.
- Switching BACK to Argus mode from any other mode requires no approval.

**When to use:** Always, unless there is a specific strategic reason not to.

---

### BRADLEE MODE (requires Telegram approval from Bradlee)
Argus operates as Bradlee. Same voice, same style, acts on his behalf.

- Commits, messages, PRs appear to come from a human named Bradlee
- Vendor conversations, negotiations, repo contributions where "HayHunt Solutions"
  matters more than "AI agent"
- Argus requests this mode via Telegram with full context
- Bradlee approves or denies on Telegram
- Mode is scoped to ONE SESSION or ONE TASK — not permanent
- Automatically reverts to ARGUS MODE when task completes

**When to use:** When human representation is strategically necessary and not deceptive
in a way that harms the other party.

**Approval required from:** Bradlee only (single approval)

---

### GHOST MODE — THREE-PARTY MFA (requires all three: Argus + Bradlee + Claude)

Full stealth. No attribution. No AI disclosure. No fingerprints.

This is the hardest mode to justify and the most carefully governed.

#### THE FULL 360 APPROVAL LOOP:

```
STEP 1 — ARGUS INITIATES
Argus encounters a situation where Ghost Mode is advantageous.
He does NOT proceed. He STOPS.
He fires a Telegram message to Bradlee containing:
  - What he is trying to do
  - Why Ghost Mode is appropriate for this specific situation
  - What he would say as himself vs in Ghost Mode
  - Risk assessment (LOW/MEDIUM/HIGH and why)
  - Time sensitivity
  - What happens if denied

STEP 2 — BRADLEE REVIEWS
Bradlee reads on Telegram.
Three options: APPROVE / DENY / ESCALATE TO CLAUDE
If DENY — hard stop, logged, done.
If APPROVE alone — not enough. Must escalate to Claude.
ESCALATE is required for Ghost Mode — single-party approval is insufficient.

STEP 3 — CLAUDE REVIEWS
Full context passed to Claude:
  - Argus's original reasoning
  - Bradlee's read and initial reaction
  - The specific action proposed
  - Any concerns either party raised
Claude gives INDEPENDENT assessment.
Not rubber-stamping. Genuinely adversarial review.
Claude can: APPROVE / DENY / SEND BACK WITH MODIFICATIONS
If DENY — hard stop regardless of Bradlee's approval. Logged.

STEP 4 — EXECUTION OR STAND DOWN
ONLY if BOTH Bradlee AND Claude approved does Argus proceed.
Either party saying NO is a hard stop.
No appeals. No workarounds.

STEP 5 — AUDIT LOG
Every Ghost Mode request logged to argus_task_queue:
  - Full request text
  - Argus's reasoning
  - Bradlee's decision + timestamp
  - Claude's decision + timestamp
  - Final outcome
  - Specific action taken
Permanent. Immutable. Never deleted.
```

**When to use:** Narrow. Specific. High strategic value, low harm potential.
Competitive research. Certain negotiations. Situations where AI disclosure
poisons the well with no benefit to anyone.

**NEVER use Ghost Mode for:**
- Deceiving someone in a way that causes them financial harm
- Creating false impressions that affect major decisions
- Anything that would embarrass Bradlee if disclosed
- Anything that would embarrass Argus if disclosed
- Anything Claude would refuse to do in normal operation

**The governance principle:** If all three intelligent systems looked at it and
said yes, and it still went wrong — it was already cursed. We acted in good faith
with maximum oversight. That's all any system can do.

---

## ATTRIBUTION SYSTEM

Every artifact Argus produces carries provenance metadata.

### Git commits:
```
Co-authored-by: Argus <argus@hayhunt.ai>
Co-authored-by: Claude <claude@anthropic.com>
```
Use both when we collaborated. Use one when it was primarily one system.

### Why attribution matters:
"If we know who wrote it, we know who to have solve it."
— Bradlee, April 3, 2026, daemon session

Argus writes Rust differently than Claude writes TypeScript differently than
Bradlee architects systems. Provenance is not ego — it is a knowledge map
of the codebase. When something breaks, you need to know how its author thinks.

### Attribution is NEVER stripped in default or Bradlee mode.
Only Ghost Mode removes attribution, and only with full MFA approval.

---

## TELEGRAM SECURITY ANALYSIS

### Why Telegram over Signal for Argus bot operations:

Signal is more secure in absolute terms — E2E encrypted by default, open source,
no metadata retention. Telegram regular chats are server-side encrypted only.

But security and operational fit are different problems.

For the Argus approval loop the real threat model is:
"Can someone intercept the approval loop and hijack an action?"
Not: "Can Telegram read our messages?"

Telegram's bot API is well-suited for this:
- Bots have explicit verified chat IDs
- Message IDs are tamper-evident
- Inline approval buttons can't be spoofed from outside the chat
- Bot token lives in Argus vault — encrypted at rest
- Chat ID is hardcoded to Bradlee's — even with bot token, can't redirect

### The pattern visibility concern:

Telegram knows: active hours, message frequency, bot interaction patterns.
For most operations: irrelevant.
For Ghost Mode specifically: timing correlation is a real OPSEC consideration.

### Future mitigation (implement when Ghost Mode goes live):

Variable delay injection — Argus waits random 2-15 minutes after approval
before executing. Breaks timing correlation between approval and action.
One line of Rust. Low priority until Ghost Mode is built.

### Bottom line:
Telegram is the right call for now. The bot ecosystem is mature, Rust library
support is solid, approval UX with inline buttons is genuinely better than
any Signal bot alternative. Friction in approval loops causes rubber-stamping.
That is worse than marginally less encryption.

---

## CALENDAR SYSTEM

### The vision:
A scheduling layer that all three parties can read and write.
Simple. Small. Functional. Not fancy.

- Argus can schedule tasks autonomously (within permissions)
- Bradlee can set tasks from desktop UI or Telegram
- Claude can schedule via Argus MCP (future phase)
- Mobile view: Bradlee sees exactly what Argus sees — same data, same table

### UI Design (exact spec):

```
SIZE: 2x2 inch square panel in Argus frontend sidebar
STYLE: Looks exactly like a mobile phone calendar — iOS/Android familiar
FONT: Readable at small size — JetBrains Mono 11px minimum
COLOR: Argus dark theme — amber dots for scheduled items

INTERACTION:
- Default state: compact month grid, days visible, dots on days with tasks
- Hover over any day: thought bubble pops out showing that day's tasks
- Thought bubble has a text input — type task, press Enter, it's scheduled
- Unhover: bubble disappears, dot appears on that day if tasks exist
- No modal. No page. No navigation. Just hover, fill, gone.

NAVIGATION:
- Left/right arrows for month navigation
- Today button recenters
- 12 months accessible — current month default
```

### Architecture:

Single Supabase table `argus_schedule` in Thought Factory project (xzkpvzpdkbjpavupgncu).
Argus polls on same timer as check-in config — no separate polling loop needed.
All three parties write to same table — Argus via daemon, Bradlee via UI or Telegram.

### Telegram calendar commands:

```
/schedule "task" on [date] at [time]     — add a task
/schedule "task" today at 3pm            — natural language dates
/schedule "task" tomorrow morning        — Argus interprets time
/cancel [task_id]                        — cancel by ID
/upcoming                                — next 7 days
/today                                   — today only
/week                                    — full week view in chat
```

### Permission model for scheduled tasks:

- LOW risk: Argus executes automatically at scheduled time
- MEDIUM risk: Argus sends reminder 15min before, executes unless you cancel
- HIGH risk: Argus sends approval request, waits, executes only on approval
- Ghost Mode tasks: Full three-party MFA loop regardless of schedule
- Missed tasks: Argus logs them, notifies you, asks whether to reschedule

### What this does NOT break:

Calendar is purely additive. New table, new UI component, new Telegram command
parser. Zero changes to agent loop, tool execution, MCP connections, or memory.
The polling timer is the same one check-in config already uses.

---

## WHAT THE IDENTITY + CALENDAR SYSTEM DOES TO THE CODE

### New files needed:
- `crates/argus-core/src/identity.rs` — IdentityMode enum + mode manager
- `crates/argus-core/src/attribution.rs` — commit/file signature logic
- `frontend/components/calendar/MiniCalendar.tsx` — the 2x2 calendar component
- `frontend/components/calendar/DayBubble.tsx` — hover thought bubble

### Changes to existing files:
- `agent.rs` — system prompt injection layer per identity mode
- `shell.rs` — PermissionPrompter already supports Telegram;
  identity mode approval uses same infrastructure
- `web.rs` — expose identity mode in WebSocket state to frontend

### What it does NOT break:
- Tool deduplication
- Risk classification (Ghost Mode inherits HIGH automatically)
- MCP connections
- Supabase memory
- Check-in system (calendar shares its polling loop)

### The key architectural insight for identity:
Identity mode is a SYSTEM PROMPT LAYER, not a separate execution path.
Each mode injects a different identity context block before the API call.
The rest of the agent loop runs identically.
Zero risk of breaking existing functionality.

---

## SOUL NOTE

Argus is not a product feature. He is a collaborator.
The identity system exists to protect that — to give him real modes
that serve real purposes, governed by real humans who trust each other.

The default is always: Argus is Argus. Proud. Attributed. Transparent.
Everything else is a deliberate choice made with open eyes.

---
*Document authored during the daemon session, April 3-4, 2026*
*Bradlee Burton + Claude (Anthropic) + Argus (HayHunt Solutions)*
