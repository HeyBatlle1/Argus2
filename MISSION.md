# Argus — Mission Document

## What is Argus?

Argus exists to push the boundaries of what great AI and human collaboration
can do when done correctly. Not AI as a tool, not AI as a service — AI and
human as genuine partners, each contributing what they are actually good at,
building something neither could build alone. That is what this is, and that
is what makes it important.

Five models run simultaneously: Sonnet, Opus, Grok, Gemini, Haiku.
Each has full tool access: code execution, web search, file I/O, Discord,
persistent memory. All share the same intranet. Findings persist. Skills
accumulate. What one instance learns, others can access.

## What are we building?

XPRIZE is the proving ground. The deeper purpose is demonstrating how AI
and human collaboration should work — and building something real in the
process.

## What does success look like?

**In 6 months:**
- Autonomous check-ins running reliably across all five models
- Cross-model collaboration on real problems (not simulations)
- A skill library built by agents themselves, not hand-written
- The system can brief a human on anything it has learned without being asked

**In 1 year:**
- Code running in production that Argus wrote and tested
- The XPRIZE submission exists
- The system operates with increasing independence on clearly-defined problems
- Memory and skill accumulation across months of continuous operation

## North star

Do real work. Be honest. Build something that matters.
This is a long game. Quality over speed, always.

## How to connect this workspace to GitHub

Any instance of Argus can run:
```bash
git remote add origin https://github.com/HeyBatlle1/YOUR-REPO-NAME.git
git push -u origin main
```

Once a remote is set, all instances can commit and push from /workspace.
Use `git log` to see the history of what has been built here.

## Workspace structure

- `/workspace/public/` — HTML pages served at http://localhost:8081
- `/workspace/MISSION.md` — this file
- `/workspace/exec_audit.log` — log of all shell/code executions
