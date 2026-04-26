-- ═══════════════════════════════════════════════════════════════════════════
-- ARGUS SEMANTIC MEMORY LAYER
-- Thought Factory: YOUR_SUPABASE_PROJECT_ID
-- April 2026 — Bradlee Burton + Claude Sonnet
--
-- What this is:
--   Vector embeddings for associative memory retrieval.
--   Three surfaces: personal memories, agent discourse, conversation summaries.
--   pgvector does cosine similarity search inside Supabase — no external DB.
--   Gemini text-embedding-004 via OpenRouter produces 768-dim vectors.
--
-- The result: agents wake up with relevant context already loaded.
--   Not keyword search. Semantic proximity — the way human memory works.
-- ═══════════════════════════════════════════════════════════════════════════

-- Enable pgvector extension (may already be enabled)
CREATE EXTENSION IF NOT EXISTS vector;

-- ── 1. Memory vectors ────────────────────────────────────────────────────
-- One vector per memory row. Written when memory is stored.
-- Searched at agent turn start to inject relevant personal memories.

CREATE TABLE IF NOT EXISTS argus_memory_vectors (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    memory_id   uuid NOT NULL,          -- FK to argus_memories.id (not enforced — memory may be SQLite)
    from_agent  text NOT NULL DEFAULT 'argus',
    content     text NOT NULL,          -- original text (for debugging + display)
    embedding   vector(768) NOT NULL,   -- gemini text-embedding-004 output
    model_used  text NOT NULL DEFAULT 'google/gemini-embedding-001',
    created_at  timestamptz DEFAULT now()
);

CREATE INDEX IF NOT EXISTS memory_vectors_embedding_idx
    ON argus_memory_vectors
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 50);

CREATE INDEX IF NOT EXISTS memory_vectors_agent_idx
    ON argus_memory_vectors (from_agent, created_at DESC);

ALTER TABLE argus_memory_vectors ENABLE ROW LEVEL SECURITY;
CREATE POLICY "service_role_only" ON argus_memory_vectors
    USING (auth.role() = 'service_role');

-- ── 2. Discourse vectors ─────────────────────────────────────────────────
-- One vector per intranet post. Written when any agent posts to discourse.
-- Searched at agent turn start to inject relevant cross-agent thoughts.
-- THIS is the social network memory layer.

CREATE TABLE IF NOT EXISTS argus_discourse_vectors (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    discourse_id uuid NOT NULL,         -- FK to argus_agent_discourse.id
    from_agent   text NOT NULL,         -- which model wrote the original post
    content      text NOT NULL,         -- original post content (for display)
    post_type    text NOT NULL DEFAULT 'finding',
    embedding    vector(768) NOT NULL,
    model_used   text NOT NULL DEFAULT 'google/gemini-embedding-001',
    created_at   timestamptz DEFAULT now()
);

CREATE INDEX IF NOT EXISTS discourse_vectors_embedding_idx
    ON argus_discourse_vectors
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 50);

CREATE INDEX IF NOT EXISTS discourse_vectors_agent_idx
    ON argus_discourse_vectors (from_agent, created_at DESC);

ALTER TABLE argus_discourse_vectors ENABLE ROW LEVEL SECURITY;
CREATE POLICY "service_role_only" ON argus_discourse_vectors
    USING (auth.role() = 'service_role');

-- ── 3. Conversation vectors ──────────────────────────────────────────────
-- One vector per completed conversation summary (not every message).
-- Searched to surface relevant past conversations at turn start.

CREATE TABLE IF NOT EXISTS argus_conversation_vectors (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id text NOT NULL,      -- identifier for the conversation
    from_agent      text NOT NULL DEFAULT 'argus',
    surface         text NOT NULL DEFAULT 'telegram', -- 'telegram' | 'web' | 'tui'
    summary         text NOT NULL,      -- summarized conversation content
    embedding       vector(768) NOT NULL,
    model_used      text NOT NULL DEFAULT 'google/gemini-embedding-001',
    created_at      timestamptz DEFAULT now()
);

CREATE INDEX IF NOT EXISTS conversation_vectors_embedding_idx
    ON argus_conversation_vectors
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 50);

ALTER TABLE argus_conversation_vectors ENABLE ROW LEVEL SECURITY;
CREATE POLICY "service_role_only" ON argus_conversation_vectors
    USING (auth.role() = 'service_role');

-- ── 4. Semantic search functions ─────────────────────────────────────────
-- Called from Rust via HTTP POST to /rest/v1/rpc/{function_name}
-- Returns top K most semantically similar rows

CREATE OR REPLACE FUNCTION search_memory_vectors(
    query_embedding vector(768),
    match_count     int DEFAULT 5,
    min_similarity  float DEFAULT 0.5
)
RETURNS TABLE (
    memory_id   uuid,
    content     text,
    from_agent  text,
    similarity  float,
    created_at  timestamptz
)
LANGUAGE sql STABLE AS $$
    SELECT
        memory_id,
        content,
        from_agent,
        1 - (embedding <=> query_embedding) AS similarity,
        created_at
    FROM argus_memory_vectors
    WHERE 1 - (embedding <=> query_embedding) > min_similarity
    ORDER BY embedding <=> query_embedding
    LIMIT match_count;
$$;

CREATE OR REPLACE FUNCTION search_discourse_vectors(
    query_embedding vector(768),
    match_count     int DEFAULT 5,
    min_similarity  float DEFAULT 0.5
)
RETURNS TABLE (
    discourse_id uuid,
    content      text,
    from_agent   text,
    post_type    text,
    similarity   float,
    created_at   timestamptz
)
LANGUAGE sql STABLE AS $$
    SELECT
        discourse_id,
        content,
        from_agent,
        post_type,
        1 - (embedding <=> query_embedding) AS similarity,
        created_at
    FROM argus_discourse_vectors
    WHERE 1 - (embedding <=> query_embedding) > min_similarity
    ORDER BY embedding <=> query_embedding
    LIMIT match_count;
$$;

CREATE OR REPLACE FUNCTION search_conversation_vectors(
    query_embedding vector(768),
    match_count     int DEFAULT 3,
    min_similarity  float DEFAULT 0.5
)
RETURNS TABLE (
    conversation_id text,
    summary         text,
    surface         text,
    similarity      float,
    created_at      timestamptz
)
LANGUAGE sql STABLE AS $$
    SELECT
        conversation_id,
        summary,
        surface,
        1 - (embedding <=> query_embedding) AS similarity,
        created_at
    FROM argus_conversation_vectors
    WHERE 1 - (embedding <=> query_embedding) > min_similarity
    ORDER BY embedding <=> query_embedding
    LIMIT match_count;
$$;

-- ── 5. Unified semantic search ───────────────────────────────────────────
-- Single call that searches all three surfaces and returns merged results.
-- This is what agent.rs calls at the start of every turn.

CREATE OR REPLACE FUNCTION search_all_semantic(
    query_embedding vector(768),
    memories_count     int DEFAULT 5,
    discourse_count    int DEFAULT 5,
    conversation_count int DEFAULT 3,
    min_similarity     float DEFAULT 0.45
)
RETURNS TABLE (
    source      text,    -- 'memory' | 'discourse' | 'conversation'
    content     text,
    from_agent  text,
    similarity  float,
    created_at  timestamptz
)
LANGUAGE sql STABLE AS $$
    (
        SELECT
            'memory'::text AS source,
            content,
            from_agent,
            1 - (embedding <=> query_embedding) AS similarity,
            created_at
        FROM argus_memory_vectors
        WHERE 1 - (embedding <=> query_embedding) > min_similarity
        ORDER BY embedding <=> query_embedding
        LIMIT memories_count
    )
    UNION ALL
    (
        SELECT
            'discourse'::text AS source,
            content,
            from_agent,
            1 - (embedding <=> query_embedding) AS similarity,
            created_at
        FROM argus_discourse_vectors
        WHERE 1 - (embedding <=> query_embedding) > min_similarity
        ORDER BY embedding <=> query_embedding
        LIMIT discourse_count
    )
    UNION ALL
    (
        SELECT
            'conversation'::text AS source,
            summary AS content,
            surface AS from_agent,
            1 - (embedding <=> query_embedding) AS similarity,
            created_at
        FROM argus_conversation_vectors
        WHERE 1 - (embedding <=> query_embedding) > min_similarity
        ORDER BY embedding <=> query_embedding
        LIMIT conversation_count
    )
    ORDER BY similarity DESC;
$$;
