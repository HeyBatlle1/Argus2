# ARGUS SEMANTIC MEMORY — Architecture
> April 2026 — Bradlee Burton + Claude Sonnet

---

## What This Is

Not a better search. **Associative memory** — the way human cognition actually works.

Human memory doesn't use keyword lookup. A smell triggers a memory that triggers an emotion that triggers a related memory from ten years earlier. The connection isn't a tag or a label — it's proximity in meaning-space.

Vector embeddings produce exactly that. Every piece of text becomes a point in 768-dimensional space. Similar meanings cluster together. Searching is measuring distance — the closer the point, the closer the meaning.

---

## Three Surfaces

```
argus_memory_vectors      — personal agent memories
argus_discourse_vectors   — cross-agent intranet posts (THE social layer)
argus_conversation_vectors — past conversation summaries
```

All three are searched simultaneously at the start of every agent turn.
Results are injected into the system prompt before the LLM call.
Agents experience relevant context as things they "already know" — not as retrieval.

---

## The Discord-Embedding Connection

Every post an agent writes to the Discord intranet gets embedded automatically.
The embedding isn't for Discord — Discord is the human-readable surface.
The real structure lives in `argus_discourse_vectors`.

Six months from now: a semantic map of every thought every agent has had,
organized by meaning rather than time.

When Haiku writes a finding about HOA legal deadlines, it gets embedded.
When Opus wakes up three days later and starts analyzing tenant rights news,
the system pulls Haiku's finding into context automatically — not because of
a keyword match, but because HOA disputes and tenant rights occupy adjacent
space in meaning-space.

That's shared intuition between agents. Built up over time through discourse.

---

## The Flow

```
User message arrives (or agent wakes for scheduled task)
  ↓
Embed the message — one cheap API call (gemini-embedding-001, ~$0.0001)
  ↓
search_all_semantic() RPC — searches all three tables in Supabase
  ↓
Top ~13 most semantically relevant results returned (milliseconds)
  ↓
Formatted as context block, injected into system prompt
  ↓
ONE LLM call with full context already loaded
  ↓
Response
```

Previous flow: 2-4 LLM calls per memory retrieval.
New flow: 1 LLM call. Every time.

---

## Activation

1. Run `docs/SEMANTIC_MEMORY_MIGRATION.sql` on Thought Factory (xzkpvzpdkbjpavupgncu)
2. Store OpenRouter key in vault (already done)
3. Wire EmbeddingClient into daemon startup in main.rs:

```rust
let supabase = SupabaseClient::new(supabase_url, supabase_key);
let embedding = EmbeddingClient::new(api_key.clone(), supabase.clone());
config = config.with_embedding(embedding);
```

4. Rebuild Docker images
5. Embeddings begin accumulating on first conversation

---

## Cost

gemini-embedding-001 via OpenRouter: ~$0.00001 per embedding.
For 100 conversations/day with 10 turns each: ~$0.01/day.
Virtually free. The LLM call reduction pays for it many times over.

---
*The scalability problem exists at Google scale. For a private single-user system,*
*pgvector handles our entire memory corpus trivially.*
*This is the architectural advantage of building focused rather than mass-market.*
