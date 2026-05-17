-- ═══════════════════════════════════════════════════════════════════════════
-- ARGUS — Complete Supabase Schema
-- Run this on a fresh Supabase project to get fully operational.
--
-- Sections:
--   1. Extensions
--   2. Core tables (memories, discourse, skills, audit anchors)
--   3. Vector tables (semantic search)
--   4. Config / log tables (checkin, schedule, conversations)
--   5. Discord webhook table + trigger
--   6. RPCs (search_all_semantic, search_skills, update_skill_usage)
--   7. Indexes
--   8. Row Level Security (all tables require service_role)
-- ═══════════════════════════════════════════════════════════════════════════

-- ─── 1. Extensions ───────────────────────────────────────────────────────────

CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pg_net;

-- ─── 2. Core tables ──────────────────────────────────────────────────────────

-- Long-term declarative memory
CREATE TABLE IF NOT EXISTS argus_memories (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    type        text NOT NULL DEFAULT 'fact',   -- fact | preference | technical | milestone
    content     text NOT NULL,
    reasoning   text,
    importance  int  NOT NULL DEFAULT 5,        -- 1–10
    tags        text[] DEFAULT '{}',
    created_at  timestamptz DEFAULT now(),
    updated_at  timestamptz DEFAULT now()
);

-- Agent-to-agent discourse (intranet posts)
CREATE TABLE IF NOT EXISTS argus_agent_discourse (
    id                   uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    from_agent           text NOT NULL,
    post_type            text NOT NULL DEFAULT 'finding',
    -- finding | question | hypothesis | proposal | disagreement | reflection | response
    title                text,
    content              text NOT NULL,
    task_context         text,
    requires_human_review boolean NOT NULL DEFAULT false,
    created_at           timestamptz DEFAULT now()
);

-- Procedural memory: reusable skill procedures
CREATE TABLE IF NOT EXISTS argus_skills (
    id                   uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    skill_name           text NOT NULL UNIQUE,
    trigger_description  text NOT NULL,
    procedure_steps      text NOT NULL,
    times_used           int  NOT NULL DEFAULT 0,
    success_rate         float NOT NULL DEFAULT 1.0,
    model_created_by     text NOT NULL DEFAULT 'argus',
    metadata             jsonb,
    embedding            vector(768),
    created_at           timestamptz DEFAULT now(),
    updated_at           timestamptz DEFAULT now()
);

-- Cryptographic audit anchors (Merkle-chained daily digests)
CREATE TABLE IF NOT EXISTS argus_audit_anchors (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    date         date NOT NULL UNIQUE,
    chain_hash   text NOT NULL,    -- SHA-256 of the day's audit chain
    prev_hash    text,             -- links to previous day's anchor
    entry_count  int  NOT NULL DEFAULT 0,
    created_at   timestamptz DEFAULT now()
);

-- ─── 3. Vector tables (semantic search layer) ────────────────────────────────

-- Memory embeddings
CREATE TABLE IF NOT EXISTS argus_memory_vectors (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    memory_id   uuid NOT NULL,
    from_agent  text NOT NULL DEFAULT 'argus',
    content     text NOT NULL,
    embedding   vector(768) NOT NULL,
    model_used  text NOT NULL DEFAULT 'google/gemini-embedding-001',
    created_at  timestamptz DEFAULT now()
);

-- Discourse embeddings
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

-- Conversation summary embeddings
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

-- ─── 4. Config / log tables ──────────────────────────────────────────────────

-- Check-in configuration (one row — edit in Supabase dashboard)
CREATE TABLE IF NOT EXISTS argus_checkin_config (
    id                  uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    interval_minutes    int  NOT NULL DEFAULT 60,
    checkin_type        text NOT NULL DEFAULT 'brief',
    quiet_hours_start   text,   -- "23:00"
    quiet_hours_end     text,   -- "07:00"
    telegram_enabled    boolean NOT NULL DEFAULT true,
    updated_at          timestamptz DEFAULT now()
);

-- Check-in log
CREATE TABLE IF NOT EXISTS argus_checkin_log (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    surface         text NOT NULL DEFAULT 'telegram',
    summary         text NOT NULL,
    system_metrics  jsonb,
    created_at      timestamptz DEFAULT now()
);

-- Conversation history per surface
CREATE TABLE IF NOT EXISTS argus_conversations (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    surface     text NOT NULL DEFAULT 'telegram',
    role        text NOT NULL,   -- 'user' | 'assistant'
    content     text NOT NULL,
    model       text,
    created_at  timestamptz DEFAULT now()
);

-- Schedule for autonomous tasks
CREATE TABLE IF NOT EXISTS argus_schedule (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    task_name       text NOT NULL,
    cron_expression text NOT NULL,
    task_prompt     text NOT NULL,
    enabled         boolean NOT NULL DEFAULT true,
    last_run        timestamptz,
    created_at      timestamptz DEFAULT now()
);

-- ─── 5. Discord webhooks + trigger ───────────────────────────────────────────

CREATE TABLE IF NOT EXISTS argus_discord_webhooks (
    channel     text PRIMARY KEY,
    webhook_url text NOT NULL,
    created_at  timestamptz DEFAULT now()
);

-- Seed channels — replace webhook URLs with your own
-- Discord: Server Settings → Integrations → Webhooks → New Webhook
INSERT INTO argus_discord_webhooks (channel, webhook_url) VALUES
    ('general',   'https://discord.com/api/webhooks/YOUR_WEBHOOK_ID/YOUR_WEBHOOK_TOKEN'),
    ('findings',  'https://discord.com/api/webhooks/YOUR_WEBHOOK_ID/YOUR_WEBHOOK_TOKEN'),
    ('questions', 'https://discord.com/api/webhooks/YOUR_WEBHOOK_ID/YOUR_WEBHOOK_TOKEN'),
    ('proposals', 'https://discord.com/api/webhooks/YOUR_WEBHOOK_ID/YOUR_WEBHOOK_TOKEN'),
    ('ops',       'https://discord.com/api/webhooks/YOUR_WEBHOOK_ID/YOUR_WEBHOOK_TOKEN')
ON CONFLICT (channel) DO UPDATE SET webhook_url = EXCLUDED.webhook_url;

-- Emoji identity per model
CREATE OR REPLACE FUNCTION argus_agent_emoji(agent text)
RETURNS text LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE agent
        WHEN 'claude-haiku'  THEN '⚡'
        WHEN 'claude-sonnet' THEN '◉'
        WHEN 'claude-opus'   THEN '🔮'
        WHEN 'grok'          THEN '⚔️'
        WHEN 'gemini'        THEN '💎'
        ELSE '🤖'
    END;
$$;

-- Channel routing
CREATE OR REPLACE FUNCTION argus_discord_channel(post_type text)
RETURNS text LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE post_type
        WHEN 'finding'      THEN 'findings'
        WHEN 'question'     THEN 'questions'
        WHEN 'hypothesis'   THEN 'questions'
        WHEN 'proposal'     THEN 'proposals'
        WHEN 'disagreement' THEN 'findings'
        WHEN 'reflection'   THEN 'general'
        WHEN 'response'     THEN 'general'
        ELSE 'general'
    END;
$$;

-- Trigger: post to Discord on every discourse insert
CREATE OR REPLACE FUNCTION argus_notify_discord()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    v_channel   text;
    v_webhook   text;
    v_emoji     text;
    v_prefix    text;
    v_content   text;
    v_title     text;
    v_payload   jsonb;
BEGIN
    v_channel := argus_discord_channel(NEW.post_type);
    v_emoji   := argus_agent_emoji(NEW.from_agent);

    SELECT webhook_url INTO v_webhook
    FROM argus_discord_webhooks
    WHERE channel = v_channel;

    IF v_webhook IS NULL THEN
        RETURN NEW;
    END IF;

    v_prefix := CASE NEW.post_type
        WHEN 'finding'      THEN '📍 FINDING'
        WHEN 'question'     THEN '❓ QUESTION'
        WHEN 'hypothesis'   THEN '🔬 HYPOTHESIS'
        WHEN 'proposal'     THEN '⚠️ PROPOSAL — REQUIRES APPROVAL'
        WHEN 'disagreement' THEN '⚡ DISAGREEMENT'
        WHEN 'reflection'   THEN '💭 REFLECTION'
        WHEN 'response'     THEN '↩️ RESPONSE'
        ELSE '📝 POST'
    END;

    v_title   := COALESCE(NEW.title, '');
    v_content := LEFT(COALESCE(NEW.content, ''), 1800);

    v_payload := jsonb_build_object(
        'username', v_emoji || ' ' || COALESCE(NEW.from_agent, 'argus'),
        'content',  '**' || v_prefix || '**' ||
                    CASE WHEN v_title != '' THEN E'\n**' || v_title || '**' ELSE '' END ||
                    E'\n\n' || v_content ||
                    CASE WHEN NEW.requires_human_review
                        THEN E'\n\n🔴 **@here — This proposal requires operator approval before any action.**'
                        ELSE ''
                    END
    );

    PERFORM net.http_post(
        url     := v_webhook,
        body    := v_payload::text,
        headers := '{"Content-Type": "application/json"}'::jsonb
    );

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS discord_notify ON argus_agent_discourse;
CREATE TRIGGER discord_notify
    AFTER INSERT ON argus_agent_discourse
    FOR EACH ROW
    EXECUTE FUNCTION argus_notify_discord();

-- ─── 6. RPCs ─────────────────────────────────────────────────────────────────

-- Unified semantic search across memory, discourse, conversation vectors
CREATE OR REPLACE FUNCTION search_all_semantic(
    query_embedding    vector(768),
    memories_count     int DEFAULT 5,
    discourse_count    int DEFAULT 5,
    conversation_count int DEFAULT 3,
    min_similarity     float DEFAULT 0.45
)
RETURNS TABLE (source text, content text, from_agent text, similarity float, created_at timestamptz)
LANGUAGE sql STABLE AS $$
    (SELECT 'memory'::text, content, from_agent,
            1-(embedding<=>query_embedding), created_at
     FROM argus_memory_vectors
     WHERE 1-(embedding<=>query_embedding) > min_similarity
     ORDER BY embedding<=>query_embedding LIMIT memories_count)
    UNION ALL
    (SELECT 'discourse'::text, content, from_agent,
            1-(embedding<=>query_embedding), created_at
     FROM argus_discourse_vectors
     WHERE 1-(embedding<=>query_embedding) > min_similarity
     ORDER BY embedding<=>query_embedding LIMIT discourse_count)
    UNION ALL
    (SELECT 'conversation'::text, summary, surface,
            1-(embedding<=>query_embedding), created_at
     FROM argus_conversation_vectors
     WHERE 1-(embedding<=>query_embedding) > min_similarity
     ORDER BY embedding<=>query_embedding LIMIT conversation_count)
    ORDER BY similarity DESC;
$$;

-- Skill search (called before each agent turn)
CREATE OR REPLACE FUNCTION search_skills(
    query_embedding vector(768),
    match_threshold float DEFAULT 0.6,
    match_count     int   DEFAULT 5
)
RETURNS TABLE (
    id                  uuid,
    skill_name          text,
    trigger_description text,
    procedure_steps     text,
    times_used          int,
    success_rate        float,
    similarity          float
)
LANGUAGE sql STABLE AS $$
    SELECT
        id, skill_name, trigger_description, procedure_steps,
        times_used, success_rate,
        1-(embedding<=>query_embedding) AS similarity
    FROM argus_skills
    WHERE embedding IS NOT NULL
      AND 1-(embedding<=>query_embedding) > match_threshold
    ORDER BY embedding<=>query_embedding
    LIMIT match_count;
$$;

-- Increment usage counter on a skill
CREATE OR REPLACE FUNCTION update_skill_usage(p_skill_id uuid)
RETURNS void LANGUAGE sql AS $$
    UPDATE argus_skills
    SET times_used = times_used + 1,
        updated_at = now()
    WHERE id = p_skill_id;
$$;

-- ─── 7. Indexes ───────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS memory_vectors_embedding_idx
    ON argus_memory_vectors USING ivfflat (embedding vector_cosine_ops) WITH (lists = 50);

CREATE INDEX IF NOT EXISTS discourse_vectors_embedding_idx
    ON argus_discourse_vectors USING ivfflat (embedding vector_cosine_ops) WITH (lists = 50);

CREATE INDEX IF NOT EXISTS conversation_vectors_embedding_idx
    ON argus_conversation_vectors USING ivfflat (embedding vector_cosine_ops) WITH (lists = 50);

CREATE INDEX IF NOT EXISTS skills_embedding_idx
    ON argus_skills USING ivfflat (embedding vector_cosine_ops) WITH (lists = 10);

CREATE INDEX IF NOT EXISTS memories_importance_idx
    ON argus_memories (importance DESC);

CREATE INDEX IF NOT EXISTS discourse_created_idx
    ON argus_agent_discourse (created_at DESC);

-- ─── 8. Row Level Security ───────────────────────────────────────────────────

-- All tables require service_role. Use the service role key in your vault —
-- never the anon key. The anon key has no access to any Argus table.

ALTER TABLE argus_memories             ENABLE ROW LEVEL SECURITY;
ALTER TABLE argus_agent_discourse      ENABLE ROW LEVEL SECURITY;
ALTER TABLE argus_skills               ENABLE ROW LEVEL SECURITY;
ALTER TABLE argus_audit_anchors        ENABLE ROW LEVEL SECURITY;
ALTER TABLE argus_memory_vectors       ENABLE ROW LEVEL SECURITY;
ALTER TABLE argus_discourse_vectors    ENABLE ROW LEVEL SECURITY;
ALTER TABLE argus_conversation_vectors ENABLE ROW LEVEL SECURITY;
ALTER TABLE argus_checkin_config       ENABLE ROW LEVEL SECURITY;
ALTER TABLE argus_checkin_log          ENABLE ROW LEVEL SECURITY;
ALTER TABLE argus_conversations        ENABLE ROW LEVEL SECURITY;
ALTER TABLE argus_schedule             ENABLE ROW LEVEL SECURITY;
ALTER TABLE argus_discord_webhooks     ENABLE ROW LEVEL SECURITY;

CREATE POLICY "service_role_only" ON argus_memories             USING (auth.role() = 'service_role');
CREATE POLICY "service_role_only" ON argus_agent_discourse      USING (auth.role() = 'service_role');
CREATE POLICY "service_role_only" ON argus_skills               USING (auth.role() = 'service_role');
CREATE POLICY "service_role_only" ON argus_audit_anchors        USING (auth.role() = 'service_role');
CREATE POLICY "service_role_only" ON argus_memory_vectors       USING (auth.role() = 'service_role');
CREATE POLICY "service_role_only" ON argus_discourse_vectors    USING (auth.role() = 'service_role');
CREATE POLICY "service_role_only" ON argus_conversation_vectors USING (auth.role() = 'service_role');
CREATE POLICY "service_role_only" ON argus_checkin_config       USING (auth.role() = 'service_role');
CREATE POLICY "service_role_only" ON argus_checkin_log          USING (auth.role() = 'service_role');
CREATE POLICY "service_role_only" ON argus_conversations        USING (auth.role() = 'service_role');
CREATE POLICY "service_role_only" ON argus_schedule             USING (auth.role() = 'service_role');
CREATE POLICY "service_role_only" ON argus_discord_webhooks     USING (auth.role() = 'service_role');
