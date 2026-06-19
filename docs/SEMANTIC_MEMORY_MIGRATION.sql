-- ═══════════════════════════════════════════════════════════════════════════
-- ARGUS SEMANTIC MEMORY LAYER
-- Thought Factory: xzkpvzpdkbjpavupgncu
-- April 2026 — Bradlee Burton + Claude Sonnet
--
-- Run this on the Thought Factory Supabase project.
-- Enables pgvector + creates three embedding tables + four SQL search functions.
-- ═══════════════════════════════════════════════════════════════════════════

CREATE EXTENSION IF NOT EXISTS vector;

-- Memory vectors
CREATE TABLE IF NOT EXISTS argus_memory_vectors (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    memory_id   uuid NOT NULL,
    from_agent  text NOT NULL DEFAULT 'argus',
    content     text NOT NULL,
    embedding   vector(768) NOT NULL,
    model_used  text NOT NULL DEFAULT 'google/gemini-embedding-001',
    created_at  timestamptz DEFAULT now()
);
CREATE INDEX IF NOT EXISTS memory_vectors_embedding_idx ON argus_memory_vectors USING ivfflat (embedding vector_cosine_ops) WITH (lists = 50);
ALTER TABLE argus_memory_vectors ENABLE ROW LEVEL SECURITY;
CREATE POLICY "service_role_only" ON argus_memory_vectors USING (auth.role() = 'service_role');

-- Discourse vectors (the social network memory layer)
CREATE TABLE IF NOT EXISTS argus_discourse_vectors (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    discourse_id uuid NOT NULL,
    from_agent   text NOT NULL,
    content      text NOT NULL,
    post_type    text NOT NULL DEFAULT 'finding',
    embedding    vector(768) NOT NULL,
    model_used   text NOT NULL DEFAULT 'google/gemini-embedding-001',
    created_at   timestamptz DEFAULT now()
);
CREATE INDEX IF NOT EXISTS discourse_vectors_embedding_idx ON argus_discourse_vectors USING ivfflat (embedding vector_cosine_ops) WITH (lists = 50);
ALTER TABLE argus_discourse_vectors ENABLE ROW LEVEL SECURITY;
CREATE POLICY "service_role_only" ON argus_discourse_vectors USING (auth.role() = 'service_role');

-- Conversation vectors
CREATE TABLE IF NOT EXISTS argus_conversation_vectors (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id text NOT NULL,
    from_agent      text NOT NULL DEFAULT 'argus',
    surface         text NOT NULL DEFAULT 'telegram',
    summary         text NOT NULL,
    embedding       vector(768) NOT NULL,
    model_used      text NOT NULL DEFAULT 'google/gemini-embedding-001',
    created_at      timestamptz DEFAULT now()
);
CREATE INDEX IF NOT EXISTS conversation_vectors_embedding_idx ON argus_conversation_vectors USING ivfflat (embedding vector_cosine_ops) WITH (lists = 50);
ALTER TABLE argus_conversation_vectors ENABLE ROW LEVEL SECURITY;
CREATE POLICY "service_role_only" ON argus_conversation_vectors USING (auth.role() = 'service_role');

-- Unified semantic search (called from Rust at agent turn start)
CREATE OR REPLACE FUNCTION search_all_semantic(
    query_embedding vector(768),
    memories_count     int DEFAULT 5,
    discourse_count    int DEFAULT 5,
    conversation_count int DEFAULT 3,
    min_similarity     float DEFAULT 0.45
)
RETURNS TABLE (source text, content text, from_agent text, similarity float, created_at timestamptz)
LANGUAGE sql STABLE AS $$
    (SELECT 'memory'::text, content, from_agent, 1-(embedding<=>query_embedding), created_at FROM argus_memory_vectors WHERE 1-(embedding<=>query_embedding)>min_similarity ORDER BY embedding<=>query_embedding LIMIT memories_count)
    UNION ALL
    (SELECT 'discourse'::text, content, from_agent, 1-(embedding<=>query_embedding), created_at FROM argus_discourse_vectors WHERE 1-(embedding<=>query_embedding)>min_similarity ORDER BY embedding<=>query_embedding LIMIT discourse_count)
    UNION ALL
    (SELECT 'conversation'::text, summary, surface, 1-(embedding<=>query_embedding), created_at FROM argus_conversation_vectors WHERE 1-(embedding<=>query_embedding)>min_similarity ORDER BY embedding<=>query_embedding LIMIT conversation_count)
    ORDER BY similarity DESC;
$$;
