-- Triage queue: holds posts awaiting Haiku review before Discord or memory.
-- Nothing is deleted from here — disposition tracks the outcome.
create table if not exists argus_triage_queue (
    id           uuid primary key default gen_random_uuid(),
    from_agent   text        not null,
    post_type    text        not null,
    content      text        not null,
    target_channel text      not null default 'ops',
    contains_links  boolean  not null default false,
    contains_claims boolean  not null default false,
    disposition  text        not null default 'pending',
    created_at   timestamptz not null default now(),
    processed_at timestamptz
);

create index if not exists triage_queue_pending
    on argus_triage_queue (disposition, created_at)
    where disposition = 'pending';

-- Triage flags: posts that failed review.
-- The model that posted is notified via Discord.
-- Nothing is deleted. Disposition tracks human review outcome.
create table if not exists argus_triage_flags (
    id               uuid primary key default gen_random_uuid(),
    original_content text        not null,
    from_agent       text        not null,
    post_type        text        not null,
    flag_reason      text        not null,
    flag_severity    text        not null default 'info',
    disposition      text        not null default 'pending',
    reviewed_by      text,
    created_at       timestamptz not null default now(),
    reviewed_at      timestamptz
);

create index if not exists triage_flags_pending
    on argus_triage_flags (disposition, created_at)
    where disposition = 'pending';

comment on table argus_triage_queue is
    'Posts awaiting Haiku triage review. Direct-lane posts are fast-pathed; triage-lane posts require Haiku classification before reaching Discord.';

comment on table argus_triage_flags is
    'Posts that failed triage. Permanent record. Model notified on creation. Human reviews and sets disposition.';
