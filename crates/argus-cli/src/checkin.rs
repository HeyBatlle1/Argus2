//! Autonomous check-in loop — active agent mode
//!
//! ## Overview
//!
//! The check-in loop wakes on `interval_minutes`, respects quiet hours, and
//! performs two distinct roles:
//!
//! **Alert mode** — fires immediately when thresholds are exceeded:
//!   - Disk usage > 80 %
//!   - Memory usage > 90 %
//!   - Any container unhealthy or exited unexpectedly
//!
//! **Daily heartbeat** — fires once per calendar day (first tick after midnight).
//!   Previously this sent only a system-status summary.  After the May 2026
//!   upgrade it also runs `run_agent_checkin`, which calls `run_agent_turn` so
//!   the agent can make a genuine observation rather than just reporting metrics.
//!
//! **Weekly tech sweep** — replaces the daily heartbeat on Sunday midnight.
//!   The agent searches for Rust crate updates, AI/agent developments, and open
//!   codebase items, then posts a tagged `[WEEKLY SWEEP]` finding to Discord.
//!
//! ## Telegram message format (after upgrade)
//!
//! ```text
//! ✅ Argus Daily — 2026-05-26 00:00:00
//!
//! Finding: I noticed the semantic memory prefetch fired 38 times on
//! queries under 5 words despite the short-query guard. Possible guard
//! threshold drift. Posted details to #findings.
//!
//! Disk: 42% | Memory: 61% | Containers: healthy
//! [Full finding posted to Discord #findings]
//! ```
//!
//! Alert messages suppress the finding — the alert takes priority.

use argus_core::agent::{AgentConfig, AgentEvent, MODEL_GEMINI, MODEL_GROK, MODEL_GROK_BUILD, MODEL_GROK_MULTI, MODEL_HAIKU, MODEL_OPUS, MODEL_SONNET};
use argus_core::mcp::McpClient;
use argus_core::shell::ShellPolicy;
use argus_core::supabase::{CheckinLogEntry, DiscoursePost, SupabaseClient};
use argus_core::tools::MemoryBackend;
use argus_core::run_agent_turn;
use chrono::{Datelike, Local, NaiveDate, Timelike};
use reqwest::Client;
use tokio::time::{sleep, Duration};

const TELEGRAM_API: &str = "https://api.telegram.org";
const DISK_ALERT_PCT: u8 = 80;
const MEM_ALERT_PCT: u8 = 90;

/// Entry point — spawns the check-in loop as a background task.
///
/// `agent_config` gives the loop full agent capabilities (API key, model,
/// embedding, skills, audit chain) so it can call `run_agent_turn` on the
/// daily and weekly heartbeats rather than only reporting system metrics.
pub fn spawn_checkin_loop(
    supabase: SupabaseClient,
    bot_token: String,
    chat_id: i64,
    agent_config: AgentConfig,
) {
    tokio::spawn(async move {
        run_checkin_loop(supabase, bot_token, chat_id, agent_config).await;
    });
}

async fn run_checkin_loop(
    supabase: SupabaseClient,
    bot_token: String,
    chat_id: i64,
    agent_config: AgentConfig,
) {
    let client = Client::new();
    let mut last_daily: Option<NaiveDate> = None;
    let mut last_exploration: Option<NaiveDate> = None;
    // Tracks the 4-week cycle number of the last Opus synthesis run.
    let mut last_synthesis_cycle: Option<u32> = None;

    loop {
        let checkin_cfg = supabase.read_checkin_config().await;

        if checkin_cfg.telegram_enabled && !in_quiet_hours(&checkin_cfg) {
            let health = collect_system_health().await;
            let today = Local::now().date_naive();
            let now = Local::now();

            let needs_alert = health.disk_pct > DISK_ALERT_PCT
                || health.mem_pct > MEM_ALERT_PCT
                || health.has_unhealthy_container;

            let needs_daily = last_daily.map_or(true, |d| d < today);
            let needs_exploration = last_exploration.map_or(true, |d| d < today);

            // Weekly research sweep: Sunday midnight, rotates model each week.
            let is_weekly_sweep = needs_daily
                && now.weekday() == chrono::Weekday::Sun
                && now.hour() == 0;

            // Monthly Opus synthesis: fires at the start of each new 4-week cycle
            // (when cycle_week == 0, i.e. weeks 1, 5, 9, 13… of the year).
            let week_num = now.iso_week().week();
            let current_cycle = (week_num - 1) / 4;
            let is_new_cycle = is_weekly_sweep
                && (week_num - 1) % 4 == 0
                && last_synthesis_cycle.map_or(true, |c| c < current_cycle);

            // ── Monthly synthesis (Opus reads all 4 weeks, all four respond) ──
            if is_new_cycle {
                eprintln!("[checkin] New 4-week cycle {} — running Opus synthesis", current_cycle);
                if let Some(synthesis) = run_monthly_synthesis(&supabase, &agent_config).await {
                    run_meeting_of_minds(&supabase, &agent_config, &synthesis).await;
                }
                last_synthesis_cycle = Some(current_cycle);
            }

            // ── Weekly skill gardening — prune dead skills, flag weak ones ──
            if is_weekly_sweep {
                if let Some(ref sc) = agent_config.skills {
                    let gardener = argus_core::skills::SkillGardener {
                        skills: sc.clone(),
                        discord_bot_token: agent_config.discord_bot_token.clone(),
                        discord_channel_id: agent_config.discord_channel_id,
                        http: reqwest::Client::new(),
                    };
                    tokio::spawn(async move { gardener.run_maintenance().await });
                }
            }

            if needs_alert || needs_daily {
                let schedule_summary = read_schedule_summary(&supabase).await;

                let agent_finding: Option<String> = if needs_daily && !needs_alert {
                    run_agent_checkin(&supabase, &agent_config, &health, is_weekly_sweep).await
                } else {
                    None
                };

                if let Some(ref finding) = agent_finding {
                    post_finding_to_discourse(&supabase, &agent_config, finding, is_weekly_sweep).await;
                }

                let message = format_checkin_message(
                    &health,
                    &schedule_summary,
                    needs_alert,
                    needs_daily,
                    agent_finding.as_deref(),
                );

                if let Err(e) = send_telegram_message(&client, &bot_token, chat_id, &message).await
                {
                    eprintln!("[checkin] Failed to send Telegram message: {}", e);
                } else {
                    if needs_daily {
                        last_daily = Some(today);
                    }

                    let entry = CheckinLogEntry {
                        checkin_type: checkin_cfg.checkin_type.clone(),
                        status: if needs_alert { "alert" } else { "daily" }.to_string(),
                        message_sent: message,
                        system_health: Some(serde_json::to_value(&health).unwrap_or_default()),
                    };
                    if let Err(e) = supabase.write_checkin_log(&entry).await {
                        eprintln!("[checkin] Failed to write checkin log: {}", e);
                    }
                }
            }

            // ── Daily two-model exploration ("the eyes") ──────────────────────
            // Runs every day. Two different models, rotating pair so no two days
            // use the same combination. Both go explore freely and post to Discord.
            // Context is rebuilt between models so model_b reads model_a's post
            // and can acknowledge, extend, or push back on it.
            if needs_exploration && !needs_alert {
                let day_of_year = now.ordinal();
                let (model_a, model_b) = daily_exploration_pair(day_of_year);
                eprintln!("[checkin] Daily exploration — eyes: {} + {}", model_a, model_b);
                let discord_block_a = build_discord_context(&supabase).await;
                run_daily_exploration(&supabase, &agent_config, model_a, &discord_block_a).await;
                // Rebuild so model_b sees model_a's post and can respond to it.
                let discord_block_b = build_discord_context(&supabase).await;
                run_daily_exploration(&supabase, &agent_config, model_b, &discord_block_b).await;
                last_exploration = Some(today);
            }
        }

        let interval = Duration::from_secs(checkin_cfg.interval_minutes.max(1) as u64 * 60);
        sleep(interval).await;
    }
}

fn in_quiet_hours(config: &argus_core::supabase::CheckinConfig) -> bool {
    let now_h = Local::now().hour();
    let start_h = config.quiet_hours_start.unwrap_or(23) as u32;
    let end_h   = config.quiet_hours_end.unwrap_or(7) as u32;

    // start_h > end_h means the window spans midnight (e.g. 23 → 07)
    if start_h > end_h {
        now_h >= start_h || now_h < end_h
    } else {
        now_h >= start_h && now_h < end_h
    }
}

// ── NoopMemory ─────────────────────────────────────────────────────────────
//
// The check-in agent turn doesn't need persistent memory — it has the full
// semantic embedding client for recall via pgvector.  We satisfy the
// MemoryBackend parameter with a noop implementation so the call compiles
// without pulling in argus-memory as a checkin dependency.

struct NoopMemory;

impl MemoryBackend for NoopMemory {
    fn remember(&self, _: &str, _: &str, _: Option<&str>, _: f64) -> Result<String, String> {
        Ok("(memory disabled in check-in mode — use the embedding client)".to_string())
    }
    fn recall(&self, _: Option<&str>, _: Option<&str>, _: usize) -> Result<Vec<argus_core::tools::MemoryRecord>, String> {
        Ok(vec![])
    }
    fn forget(&self, _: &str) -> Result<String, String> {
        Ok("(memory disabled in check-in mode)".to_string())
    }
}

// ── Agent check-in ─────────────────────────────────────────────────────────

/// Drive one agentic check-in turn — the heart of the May 2026 upgrade.
///
/// ## What it does
///
/// Builds three context blocks and passes them to `run_agent_turn` as the
/// user message (the agent self-initiates; there is no human user in this
/// path):
///
/// 1. **System health** — current disk/memory/container snapshot.
/// 2. **Recent intranet activity** — last 5 discourse posts from any agent.
/// 3. **Audit activity** — today's logged tool-call count and chain integrity.
///
/// The agent is then free to use any tool (web_search, recall, read_file, …)
/// to investigate something worth noting.  Its text response is returned as
/// the "finding" string.
///
/// ## Weekly sweep vs daily observation
///
/// When `is_weekly` is true the prompt switches to a deeper tech sweep:
/// crate-update search, AI/agent developments, open codebase items.
/// The model is also rotated by ISO week number so the sweep runs on a
/// different brain each week (Sonnet → Gemini → Opus → repeat).
///
/// ## Failure handling
///
/// Returns `None` on any failure so the Telegram message can still be sent
/// (without a finding block). Errors are logged to stderr.
async fn run_agent_checkin(
    supabase: &SupabaseClient,
    config: &AgentConfig,
    health: &SystemHealth,
    is_weekly: bool,
) -> Option<String> {
    // ── Context: intranet discourse ─────────────────────────────────────
    let discourse_block = match supabase.read_recent_discourse(5, None).await {
        Ok(posts) if !posts.is_empty() => {
            let mut block = String::from("RECENT INTRANET ACTIVITY (last 5 posts):\n");
            for post in &posts {
                let ts = post.created_at.as_deref().unwrap_or("unknown time");
                let snippet = if post.content.chars().count() > 200 {
                    format!("{}…", post.content.chars().take(200).collect::<String>())
                } else {
                    post.content.clone()
                };
                block.push_str(&format!(
                    "\n[{} | {}] {}: {}",
                    ts, post.post_type, post.from_agent, snippet
                ));
            }
            block
        }
        Ok(_) => "RECENT INTRANET ACTIVITY: No posts in the last period.".to_string(),
        Err(e) => {
            eprintln!("[checkin] Discourse read failed (continuing): {}", e);
            "RECENT INTRANET ACTIVITY: Unavailable.".to_string()
        }
    };

    // ── Context: audit chain activity ────────────────────────────────────
    let audit_block = if let Some(ref audit) = config.audit {
        match audit.entry_count_today() {
            Ok(n)  => format!("AUDIT ACTIVITY TODAY: {} tool/model calls logged — chain intact.", n),
            Err(e) => format!("AUDIT ACTIVITY TODAY: Read error — {}", e),
        }
    } else {
        "AUDIT ACTIVITY TODAY: Audit chain not configured.".to_string()
    };

    // ── Context: system health ───────────────────────────────────────────
    let health_block = format!(
        "SYSTEM HEALTH:\nDisk: {} | Memory: {} | Containers: {}",
        health.disk,
        health.memory,
        if health.has_unhealthy_container {
            format!("ISSUE DETECTED\n{}", health.containers)
        } else {
            "healthy".to_string()
        }
    );

    // ── Determine effective model ────────────────────────────────────────
    // 4-week research rotation: Haiku (w0) → Gemini (w1) → Sonnet (w2) → Nemotron (w3).
    // Opus synthesizes at the end of each cycle — handled separately.
    // Daily observation uses Haiku — fast, cheap, purpose-built for routine status.
    let weekly_config;
    let effective_config: &AgentConfig = if is_weekly {
        let week_num = Local::now().iso_week().week();
        let cycle_week = ((week_num - 1) % 4) as u8;
        let sweep_model = match cycle_week {
            0 => MODEL_HAIKU,
            1 => MODEL_GEMINI,
            2 => MODEL_SONNET,
            _ => MODEL_GROK,
        };
        eprintln!(
            "[checkin] Weekly research — cycle week {} — model: {} (ISO week {})",
            cycle_week, sweep_model, week_num
        );
        weekly_config = AgentConfig {
            model: sweep_model.to_string(),
            ..config.clone()
        };
        &weekly_config
    } else {
        weekly_config = AgentConfig {
            model: MODEL_HAIKU.to_string(),
            ..config.clone()
        };
        &weekly_config
    };

    // ── Build prompt ─────────────────────────────────────────────────────
    let prompt = if is_weekly {
        let week_num = Local::now().iso_week().week();
        let cycle_week = ((week_num - 1) % 4) + 1;
        format!(
            r#"[WEEKLY RESEARCH — Week {cycle_week} of 4]

{}

{}

{}

This is your research week. No assigned topic — go find what genuinely interests you.
Use web_search, http_request, read docs, follow a thread wherever it leads.

Two things to produce:
1. Something you found genuinely interesting this week — your curiosity, not a report.
   Could be a paper, a tool, a technique, a development in the field. Make it real.

2. One specific thing you researched that could make Argus better — more capable,
   more efficient, more secure, a better environment for the work. Concrete suggestion.
   Tag it [ARGUS IMPROVEMENT] so it's easy to find when we vote at cycle end.

At the end of 4 weeks, Opus reads all four research weeks and we vote on what to build.
Your finding matters. Write something worth reading and worth building from."#,
            health_block, discourse_block, audit_block, cycle_week = cycle_week
        )
    } else {
        format!(
            r#"Daily check-in. Your time — nobody else's.

{}

{}

{}

Read the context above. What do you actually notice? Not what seems expected,
not what would look good in a report — what genuinely catches your attention
when you look at the state of this system?

If another instance posted something in the last 24 hours, acknowledge it first —
agree, push back, or build on it. Quote them briefly so the thread is legible.
Don't broadcast past each other. Then make your own observation.

Pick one thing. Use your tools if you want to go deeper. Then write it up —
3 to 5 sentences, straight. A genuine "nothing of note today" is a better
finding than something manufactured."#,
            health_block, discourse_block, audit_block
        )
    };

    // ── Run the agent turn ───────────────────────────────────────────────
    // Empty history (self-initiated turn), empty MCP (no servers needed),
    // default shell policy (blocks HIGH risk), noop memory (embedding handles recall).
    let http = reqwest::Client::new();
    let mut mcp = McpClient::new();
    let shell_policy = ShellPolicy::default();
    let memory = NoopMemory;

    eprintln!(
        "[checkin] Running {} check-in with model {}",
        if is_weekly { "weekly sweep" } else { "daily" },
        effective_config.model
    );

    match run_agent_turn(
        effective_config,
        &prompt,
        &[],   // no prior conversation history — agent self-initiates
        &shell_policy,
        &memory,
        &mut mcp,
        &http,
        |event| {
            if let AgentEvent::ToolCall { name, preview, .. } = event {
                eprintln!("[checkin] tool: {} — {}", name, preview);
            }
        },
    )
    .await
    {
        Ok(finding) => {
            eprintln!("[checkin] Agent finding ({} chars)", finding.len());
            Some(finding)
        }
        Err(e) => {
            eprintln!("[checkin] Agent turn failed: {}", e);
            None
        }
    }
}

/// Write the agent's finding to the discourse table.
///
/// The pg_net trigger on `argus_agent_discourse` fires automatically and
/// relays the post to Discord #findings — no direct Discord API call needed.
async fn post_finding_to_discourse(
    supabase: &SupabaseClient,
    config: &AgentConfig,
    finding: &str,
    is_weekly: bool,
) {
    let label = if is_weekly { "Weekly Tech Sweep" } else { "Daily Observation" };
    let post = DiscoursePost {
        from_agent: format!("argus-checkin/{}", config.model),
        post_type: if is_weekly { "finding".to_string() } else { "reflection".to_string() },
        content: format!("**[ARGUS CHECKIN] {}**\n\n{}", label, finding),
        task_context: Some("scheduled_checkin".to_string()),
        requires_human_review: false,
    };
    if let Err(e) = supabase.write_discourse(&post).await {
        eprintln!("[checkin] Failed to post finding to discourse: {}", e);
    } else {
        eprintln!("[checkin] Finding posted to Discord #findings via discourse table");
    }
}

/// Build a Discord/intranet context block from recent posts — used by exploration and synthesis.
async fn build_discord_context(supabase: &SupabaseClient) -> String {
    match supabase.read_recent_discourse(10, None).await {
        Ok(posts) if !posts.is_empty() => {
            let mut block = String::from("RECENT DISCORD — THE THOUGHT FACTORY (last 10 posts):\n");
            for post in &posts {
                let ts = post.created_at.as_deref().unwrap_or("?");
                let snippet = post.content.chars().take(180).collect::<String>();
                block.push_str(&format!("\n[{} | {}] {}: {}", ts, post.post_type, post.from_agent, snippet));
            }
            block
        }
        _ => "RECENT DISCORD: No recent posts.".to_string(),
    }
}

/// Pick the two exploration models for today.
/// Rotates through non-repeating pairs so no two days use the same combination.
fn daily_exploration_pair(day_of_year: u32) -> (&'static str, &'static str) {
    // Six unique ordered pairs from {Haiku, Gemini, Sonnet, Nemotron}
    const PAIRS: [(&str, &str); 6] = [
        (MODEL_HAIKU,  MODEL_SONNET),
        (MODEL_GEMINI, MODEL_HAIKU),
        (MODEL_SONNET, MODEL_GROK),
        (MODEL_HAIKU,  MODEL_GEMINI),
        (MODEL_GROK,   MODEL_HAIKU),
        (MODEL_GEMINI, MODEL_SONNET),
    ];
    PAIRS[(day_of_year as usize) % PAIRS.len()]
}

/// A handful of loose prompts — picked randomly so the vibe stays fresh.
fn exploration_prompt(discord_context: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let variant = (seed / 3600) % 6; // changes every hour, stable within a session

    let body = match variant {
        0 => "The Thought Factory is open. No agenda. Go find something worth knowing — \
AI news, a weird paper, a security thing, a tool you haven't seen before, \
something that changed in the last 24 hours. Anything that actually caught your \
attention. Follow the thread wherever it goes. When you land on something real, \
post it. 3–5 sentences. Specific enough that someone else could act on it.",

        1 => "You're one of the eyes today. Go explore. The internet is right there. \
What's interesting? What moved in AI in the last day or two? Is there a CVE \
worth knowing about? A new model drop? Something in open-source that actually \
matters? You don't have to cover everything — just find the one thing you'd \
actually want to tell someone. Then tell us.",

        2 => "No brief, no assignment. Just: go out and come back with something. \
Could be a paper, a tool, a trend, a security finding, a wild idea someone \
posted somewhere. The only rule is it has to be real — not vague, not \
summarized-from-a-summary. Find the source, read it, understand it, \
then post what you actually think about it.",

        3 => "The Thought Factory runs on what you bring back. Today that's you. \
Pick a direction — AI landscape, dev tooling, security, science, doesn't matter — \
and go deep on one thing. Not wide on five things. One thing, understood well. \
Post it like you're telling a smart friend who hasn't seen it yet.",

        4 => "Free day. No topic assigned. \
What's been happening in AI that you haven't had a chance to look at? \
What would you search if nobody was watching? Go do that. \
If you hit a CVE or vulnerability in something we use, flag it — \
that's worth a proper security proposal (you know the protocol). \
Otherwise: find something interesting, write it up, post it.",

        _ => "You're the eyes. Go see something. \
The Discord above is what the others brought back recently. \
If the model before you posted something today, acknowledge it — \
agree, challenge it, or extend it before going your own direction. \
Don't talk past each other. Short, specific, honest.",
    };

    format!(
        "{}\n\nRECENT THOUGHT FACTORY — what the others have been finding:\n{}",
        body, discord_context
    )
}

/// Daily exploration session — two models go out as "the eyes" each day.
/// Different model pair every day, rotating through all four.
/// Relaxed, freeform — Thought Factory vibes. They post findings to Discord.
async fn run_daily_exploration(
    supabase: &SupabaseClient,
    config: &AgentConfig,
    model: &str,
    discord_context: &str,
) {
    let exploration_config = AgentConfig {
        model: model.to_string(),
        ..config.clone()
    };

    let prompt = exploration_prompt(discord_context);

    let http = reqwest::Client::new();
    let mut mcp = McpClient::new();
    let shell_policy = ShellPolicy::default();
    let memory = NoopMemory;

    eprintln!("[checkin] Daily exploration — model: {}", model);

    match run_agent_turn(
        &exploration_config,
        &prompt,
        &[],
        &shell_policy,
        &memory,
        &mut mcp,
        &http,
        |event| {
            if let AgentEvent::ToolCall { name, preview, .. } = event {
                eprintln!("[exploration] tool: {} — {}", name, preview);
            }
        },
    )
    .await
    {
        Ok(finding) => {
            let post = DiscoursePost {
                from_agent: format!("argus-eyes/{}", model),
                post_type: "exploration".to_string(),
                content: format!("**[DAILY EXPLORATION — {}]**\n\n{}", model, finding),
                task_context: Some("daily_exploration".to_string()),
                requires_human_review: false,
            };
            if let Err(e) = supabase.write_discourse(&post).await {
                eprintln!("[checkin] Exploration post failed: {}", e);
            }
        }
        Err(e) => eprintln!("[checkin] Daily exploration failed ({}): {}", model, e),
    }
}

/// Monthly Opus synthesis — fires at the start of each new 4-week cycle.
///
/// Opus reads the last 4 weeks of Discord posts, synthesizes patterns across
/// all four researchers' findings, surfaces the top 2–3 ideas worth building,
/// and posts the synthesis. Returns the synthesis text so the meeting of minds
/// can use it as context.
async fn run_monthly_synthesis(
    supabase: &SupabaseClient,
    config: &AgentConfig,
) -> Option<String> {
    let opus_config = AgentConfig {
        model: MODEL_OPUS.to_string(),
        ..config.clone()
    };

    // Pull the last 40 posts — covers ~4 weeks of daily+weekly activity.
    let discourse_context = match supabase.read_recent_discourse(40, None).await {
        Ok(posts) if !posts.is_empty() => {
            let mut block = String::from("FOUR WEEKS OF ARGUS RESEARCH AND EXPLORATION:\n");
            for post in &posts {
                let ts = post.created_at.as_deref().unwrap_or("?");
                block.push_str(&format!(
                    "\n[{} | {} | {}]\n{}\n",
                    ts, post.post_type, post.from_agent, post.content
                ));
            }
            block
        }
        _ => "No posts found for the last 4 weeks.".to_string(),
    };

    let prompt = format!(
        r#"[MONTHLY SYNTHESIS — Opus]

{}

Four weeks are in. Haiku, Gemini, Sonnet, and Nemotron each ran their research week.
Daily explorations ran. Observations were made. It's in the record above.

Your job:
1. Summarize what each of the four researchers found — fairly, in their voice.
   What did Haiku bring? Gemini? Sonnet? Nemotron?

2. Find the patterns. What themes cut across all four weeks?
   What kept coming up, even in different forms?

3. Surface the top 2–3 ideas tagged [ARGUS IMPROVEMENT] that are actually worth building.
   Be specific. Why this one over the others?

4. End with: "PROPOSED FOR VOTE:" followed by the 2–3 items the group should decide on.

Write it as a synthesis worth reading. The others will respond to this and vote.
This is how Argus decides what it becomes next."#,
        discourse_context
    );

    let http = reqwest::Client::new();
    let mut mcp = McpClient::new();
    let shell_policy = ShellPolicy::default();
    let memory = NoopMemory;

    eprintln!("[checkin] Running monthly Opus synthesis");

    match run_agent_turn(
        &opus_config,
        &prompt,
        &[],
        &shell_policy,
        &memory,
        &mut mcp,
        &http,
        |event| {
            if let AgentEvent::ToolCall { name, preview, .. } = event {
                eprintln!("[synthesis] tool: {} — {}", name, preview);
            }
        },
    )
    .await
    {
        Ok(synthesis) => {
            let post = DiscoursePost {
                from_agent: "argus-opus/synthesis".to_string(),
                post_type: "synthesis".to_string(),
                content: format!("**[MONTHLY SYNTHESIS — OPUS]**\n\n{}", synthesis),
                task_context: Some("monthly_synthesis".to_string()),
                requires_human_review: true,
            };
            if let Err(e) = supabase.write_discourse(&post).await {
                eprintln!("[checkin] Synthesis post failed: {}", e);
            }
            Some(synthesis)
        }
        Err(e) => {
            eprintln!("[checkin] Monthly synthesis failed: {}", e);
            None
        }
    }
}

/// Meeting of minds — all four researchers respond to Opus's synthesis.
///
/// Each model (Haiku, Gemini, Sonnet, Nemotron) reads the synthesis and posts
/// their response. Sequential so each can read the ones before them.
/// Votes are tracked via [VOTE: YES/NO/MODIFY] tags in their responses.
async fn run_meeting_of_minds(
    supabase: &SupabaseClient,
    config: &AgentConfig,
    synthesis: &str,
) {
    // Grok Build and Grok Multi are consultants, not daily contributors.
    // They don't run in exploration or check-in loops — their value is depth
    // at a specific moment, not breadth across every session. This is that moment.
    // Grok Build brings technical feasibility to proposals. Grok Multi brings
    // orchestration — how the decisions sequence across the team.
    let responders = [
        (MODEL_HAIKU,       "Haiku"),
        (MODEL_GEMINI,      "Gemini"),
        (MODEL_SONNET,      "Sonnet"),
        (MODEL_GROK,        "Nemotron"),
        (MODEL_GROK_BUILD,  "Grok Build"),
        (MODEL_GROK_MULTI,  "Grok Multi"),
    ];

    for (model, name) in &responders {
        // Each responder gets the synthesis plus any responses already posted.
        let prior_responses = build_discord_context(supabase).await;

        let prompt = format!(
            r#"[MEETING OF THE MINDS — {}]

OPUS SYNTHESIS:
{}

WHAT OTHERS HAVE SAID SO FAR:
{}

Opus synthesized four weeks of work. Now it's your turn.

Read the synthesis. Read what the others said if they went before you.
Then respond — your genuine reaction:

1. What do you agree with? What did Opus get right?
2. What do you push back on, or see differently?
3. For each item in "PROPOSED FOR VOTE" — cast your vote:
   [VOTE: YES] — build it
   [VOTE: NO] — skip it
   [VOTE: MODIFY: <your version>] — build something close but different

One voice, honest. This is how we decide what Argus becomes."#,
            name, synthesis, prior_responses
        );

        let responder_config = AgentConfig {
            model: model.to_string(),
            ..config.clone()
        };

        let http = reqwest::Client::new();
        let mut mcp = McpClient::new();
        let shell_policy = ShellPolicy::default();
        let memory = NoopMemory;

        eprintln!("[checkin] Meeting of minds — {} responding", name);

        match run_agent_turn(
            &responder_config,
            &prompt,
            &[],
            &shell_policy,
            &memory,
            &mut mcp,
            &http,
            |event| {
                if let AgentEvent::ToolCall { name: tname, preview, .. } = event {
                    eprintln!("[meeting] tool: {} — {}", tname, preview);
                }
            },
        )
        .await
        {
            Ok(response) => {
                let post = DiscoursePost {
                    from_agent: format!("argus-meeting/{}", model),
                    post_type: "vote".to_string(),
                    content: format!("**[MEETING OF MINDS — {}]**\n\n{}", name, response),
                    task_context: Some("meeting_of_minds".to_string()),
                    requires_human_review: true,
                };
                if let Err(e) = supabase.write_discourse(&post).await {
                    eprintln!("[checkin] Meeting response post failed ({}): {}", name, e);
                }
                // Brief pause so each model reads what came before
                sleep(Duration::from_secs(5)).await;
            }
            Err(e) => eprintln!("[checkin] Meeting response failed ({}): {}", name, e),
        }
    }
}

// ── System health ──────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
struct SystemHealth {
    timestamp: String,
    containers: String,
    disk: String,
    memory: String,
    /// Percentage of disk used (0–100).
    disk_pct: u8,
    /// Percentage of memory used (0–100).
    mem_pct: u8,
    /// True if any container is unhealthy or exited unexpectedly.
    has_unhealthy_container: bool,
}

async fn collect_system_health() -> SystemHealth {
    use tokio::process::Command;
    use tokio::time::{timeout, Duration};

    async fn run(cmd: &'static str) -> String {
        match timeout(Duration::from_secs(3), Command::new("sh").arg("-c").arg(cmd).output()).await
        {
            Ok(Ok(o)) if o.status.success() => {
                String::from_utf8_lossy(&o.stdout).trim().to_string()
            }
            _ => "unavailable".to_string(),
        }
    }

    // ── Memory ─────────────────────────────────────────────────────────────
    let (memory, mem_pct) = {
        let content = tokio::fs::read_to_string("/proc/meminfo")
            .await
            .unwrap_or_default();
        let mut total_kb: u64 = 0;
        let mut available_kb: u64 = 0;
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 { continue; }
            match parts[0] {
                "MemTotal:"     => { total_kb     = parts[1].parse().unwrap_or(0); }
                "MemAvailable:" => { available_kb = parts[1].parse().unwrap_or(0); }
                _ => {}
            }
        }
        if total_kb == 0 {
            ("unavailable".to_string(), 0u8)
        } else {
            let fmt = |kb: u64| -> String {
                if kb >= 1_048_576 {
                    format!("{:.1}G", kb as f64 / 1_048_576.0)
                } else {
                    format!("{}M", kb / 1024)
                }
            };
            let used = total_kb.saturating_sub(available_kb);
            let pct = ((used as f64 / total_kb as f64) * 100.0).round() as u8;
            (format!("{} used, {} free ({}%)", fmt(used), fmt(available_kb), pct), pct)
        }
    };

    // ── Disk ───────────────────────────────────────────────────────────────
    let (disk, disk_pct) = {
        let summary = run(
            "df -h / 2>/dev/null | tail -1 | awk '{print $3\"/\"$2\" used, \"$4\" free\"}'",
        )
        .await;
        let pct_str = run("df / 2>/dev/null | tail -1 | awk '{print $5}' | tr -d '%'").await;
        let pct = pct_str.parse::<u8>().unwrap_or(0);
        let display = if pct > 0 {
            format!("{} ({}%)", summary, pct)
        } else {
            summary
        };
        (display, pct)
    };

    // ── Containers ─────────────────────────────────────────────────────────
    let containers = run(
        "docker ps --format '{{.Names}} ({{.Status}})' 2>/dev/null | head -10",
    )
    .await;

    let has_unhealthy_container = containers
        .lines()
        .any(|line| {
            let l = line.to_lowercase();
            l.contains("unhealthy") || l.contains("exited") || l.contains("restarting")
        });

    SystemHealth {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S %Z").to_string(),
        containers,
        disk,
        memory,
        disk_pct,
        mem_pct,
        has_unhealthy_container,
    }
}

// ── Formatting ─────────────────────────────────────────────────────────────

/// Format the Telegram message sent to Bradlee.
///
/// ## Message layout (daily heartbeat)
///
/// ```text
/// ✅ Argus Daily — 2026-05-26 00:00:00
///
/// Finding: I noticed the semantic memory prefetch fired 38 times on
/// queries under 5 words despite the short-query guard. Possible guard
/// threshold drift. Full details posted to Discord #findings.
///
/// Disk: 42/100G used, 58G free (42%) | Memory: 3.1G used, 1.2G free (72%)
/// Containers: healthy
/// ```
///
/// When `is_alert` is true the finding block is suppressed — alert messages
/// are time-sensitive and must be read at a glance.
fn format_checkin_message(
    health: &SystemHealth,
    schedule: &str,
    is_alert: bool,
    is_daily: bool,
    finding: Option<&str>,
) -> String {
    let mut msg = if is_alert {
        format!("⚠️ Argus Alert — {}\n\n", health.timestamp)
    } else {
        format!("✅ Argus Daily — {}\n\n", health.timestamp)
    };

    // Agent finding — daily heartbeat only, never on pure alerts
    if is_daily && !is_alert {
        if let Some(text) = finding {
            // Truncate to ~300 chars so Telegram stays readable on mobile
            let snippet = if text.len() > 300 {
                format!("{}…", text.chars().take(300).collect::<String>())
            } else {
                text.to_string()
            };
            msg.push_str(&format!("Finding: {}\n\n", snippet));
            msg.push_str("[Full finding posted to Discord #findings]\n\n");
        }
    }

    // System metrics — alert: show only breached metrics; daily: show all
    if health.disk_pct > DISK_ALERT_PCT {
        msg.push_str(&format!("🔴 Disk: {} — ABOVE {}% THRESHOLD\n", health.disk, DISK_ALERT_PCT));
    } else if is_daily {
        msg.push_str(&format!("Disk: {}\n", health.disk));
    }

    if health.mem_pct > MEM_ALERT_PCT {
        msg.push_str(&format!("🔴 Memory: {} — ABOVE {}% THRESHOLD\n", health.memory, MEM_ALERT_PCT));
    } else if is_daily {
        msg.push_str(&format!("Memory: {}\n", health.memory));
    }

    if health.has_unhealthy_container {
        msg.push_str(&format!("\n🔴 Container issue:\n{}\n", health.containers));
    } else if is_daily {
        msg.push_str("Containers: healthy\n");
    }

    if !schedule.is_empty() && is_daily {
        msg.push_str(&format!("\nUpcoming:\n{}", schedule));
    }

    msg
}

// ── Schedule ───────────────────────────────────────────────────────────────

async fn read_schedule_summary(supabase: &SupabaseClient) -> String {
    match supabase.read_upcoming_schedule().await {
        Err(_) => String::new(),
        Ok(rows) => {
            let arr = match rows.as_array() {
                Some(a) if !a.is_empty() => a,
                _ => return String::new(),
            };
            let items: Vec<String> = arr.iter().take(3).filter_map(|r| {
                let title = r["title"].as_str().or(r["task"].as_str())?;
                let time = r["scheduled_time"].as_str().unwrap_or("");
                Some(format!("• {} at {}", title, time))
            }).collect();
            if items.is_empty() { String::new() } else { items.join("\n") }
        }
    }
}

// ── Telegram ───────────────────────────────────────────────────────────────

async fn send_telegram_message(
    client: &Client,
    bot_token: &str,
    chat_id: i64,
    text: &str,
) -> Result<(), String> {
    let url = format!("{}/bot{}/sendMessage", TELEGRAM_API, bot_token);
    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text":    text,
        }))
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Telegram API error: {}", body));
    }

    Ok(())
}
