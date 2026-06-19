//! Triage loop — Haiku's gate.
//!
//! Spawned as a background task alongside the check-in loop.
//! Polls argus_triage_queue every 30 seconds, runs Haiku classification
//! on each entry, routes approved posts to Discord, flags failed ones
//! to argus_triage_flags and notifies the posting model.
//!
//! Haiku in this role: read-only context, no tools, no execution.
//! Classification only. If something comes through designed to manipulate,
//! it finds nothing to grab onto.

use argus_core::agent::{AgentConfig, AgentEvent, MODEL_TRIAGE};
use argus_core::mcp::McpClient;
use argus_core::shell::ShellPolicy;
use argus_core::supabase::SupabaseClient;
use argus_core::triage::{TriageFlag, TriageResult, build_haiku_triage_prompt, classify_lane, route_to_channel, TriageLane, TriageEntry};
use argus_core::tools::MemoryBackend;
use argus_core::run_agent_turn;
use reqwest::Client;
use tokio::time::{sleep, Duration};

const POLL_INTERVAL_SECS: u64 = 30;
const DISCORD_API: &str = "https://discord.com/api/v10";

struct NoopMemory;
impl MemoryBackend for NoopMemory {
    fn remember(&self, _: &str, _: &str, _: Option<&str>, _: f64) -> Result<String, String> { Ok(String::new()) }
    fn recall(&self, _: Option<&str>, _: Option<&str>, _: usize) -> Result<Vec<argus_core::tools::MemoryRecord>, String> { Ok(vec![]) }
    fn forget(&self, _: &str) -> Result<String, String> { Ok(String::new()) }
}

pub fn spawn_triage_loop(
    supabase: SupabaseClient,
    agent_config: AgentConfig,
    discord_token: String,
    channel_id: String,
) {
    tokio::spawn(async move {
        run_triage_loop(supabase, agent_config, discord_token, channel_id).await;
    });
}

async fn run_triage_loop(
    supabase: SupabaseClient,
    agent_config: AgentConfig,
    discord_token: String,
    channel_id: String,
) {
    let http = Client::new();

    // Channel ID map — read from channel list on first run if needed.
    // For now these are the known Freedom Zone channel IDs.
    let channel_map = std::collections::HashMap::from([
        ("ops",       "1496620295316832507"),
        ("findings",  "1496619992416780439"),
        ("questions", "1496620088781177036"),
        ("proposals", "1496620206221426950"),
        ("general",   "1491911834666533017"),
        ("flags",     "1496620295316832507"), // ops until we create a dedicated flags channel
    ]);

    loop {
        match supabase.read_pending_triage().await {
            Ok(entries) if !entries.is_empty() => {
                eprintln!("[triage] {} item(s) in queue", entries.len());

                for entry_val in entries {
                    let id = entry_val["id"].as_str().unwrap_or("").to_string();
                    let from_agent = entry_val["from_agent"].as_str().unwrap_or("unknown").to_string();
                    let post_type = entry_val["post_type"].as_str().unwrap_or("unknown").to_string();
                    let content = entry_val["content"].as_str().unwrap_or("").to_string();
                    let contains_links = entry_val["contains_links"].as_bool().unwrap_or(false);
                    let contains_claims = entry_val["contains_claims"].as_bool().unwrap_or(false);

                    let lane = classify_lane(&post_type, &content);
                    let target_channel = route_to_channel(&post_type, &content);

                    match lane {
                        // ── Direct lane ────────────────────────────────────
                        TriageLane::Direct => {
                            eprintln!("[triage] direct → #{} ({})", target_channel, from_agent);
                            if let Some(ch_id) = channel_map.get(target_channel) {
                                post_to_discord(&http, &discord_token, ch_id, &content).await;
                            }
                            let _ = supabase.mark_triage_processed(&id, "approved_direct").await;
                        }

                        // ── Triage lane — Haiku review ─────────────────────
                        TriageLane::Triage => {
                            eprintln!("[triage] haiku review → #{} ({})", target_channel, from_agent);

                            let entry = TriageEntry {
                                from_agent: from_agent.clone(),
                                post_type: post_type.clone(),
                                content: content.clone(),
                                target_channel: target_channel.to_string(),
                                contains_links,
                                contains_claims,
                            };

                            let prompt = build_haiku_triage_prompt(&entry);
                            let haiku_config = AgentConfig {
                                model: MODEL_TRIAGE.to_string(),
                                ..agent_config.clone()
                            };

                            let mut mcp = McpClient::new();
                            let shell_policy = ShellPolicy::default();
                            let memory = NoopMemory;

                            match run_agent_turn(
                                &haiku_config,
                                &prompt,
                                &[],
                                &shell_policy,
                                &memory,
                                &mut mcp,
                                &http,
                                |event| {
                                    if let AgentEvent::ToolCall { name, preview, .. } = event {
                                        eprintln!("[triage] haiku tool: {} — {}", name, preview);
                                    }
                                },
                            ).await {
                                Ok(response) => {
                                    // Parse Haiku's JSON decision
                                    let result: Option<TriageResult> = extract_json(&response);

                                    match result {
                                        Some(r) if r.approved => {
                                            // Approved — post verbatim
                                            eprintln!("[triage] approved → #{}", r.channel);
                                            if let Some(ch_id) = channel_map.get(r.channel.trim_start_matches('#')) {
                                                post_to_discord(&http, &discord_token, ch_id, &content).await;
                                            }
                                            let _ = supabase.mark_triage_processed(&id, "approved").await;
                                        }
                                        Some(r) => {
                                            // Flagged — write to flags table, notify model
                                            eprintln!("[triage] flagged: {:?}", r.flag_reason);
                                            let flag = TriageFlag {
                                                original_content: content.clone(),
                                                from_agent: from_agent.clone(),
                                                post_type: post_type.clone(),
                                                flag_reason: r.flag_reason.unwrap_or_else(|| "unspecified".to_string()),
                                                flag_severity: r.flag_severity.unwrap_or_else(|| "info".to_string()),
                                                disposition: "pending".to_string(),
                                            };
                                            let _ = supabase.write_triage_flag(&flag).await;
                                            let _ = supabase.mark_triage_processed(&id, "flagged").await;

                                            // Notify the model in Discord
                                            let notify_msg = format!(
                                                "**[TRIAGE]** {} — you have a message in Flags. Check `argus_triage_flags` for details.",
                                                from_agent
                                            );
                                            if let Some(ch_id) = channel_map.get("ops") {
                                                post_to_discord(&http, &discord_token, ch_id, &notify_msg).await;
                                            }
                                        }
                                        None => {
                                            // Haiku returned unparseable response — approve with warning
                                            eprintln!("[triage] haiku parse failed — approving with warning");
                                            if let Some(ch_id) = channel_map.get(target_channel) {
                                                post_to_discord(&http, &discord_token, ch_id, &content).await;
                                            }
                                            let _ = supabase.mark_triage_processed(&id, "approved_fallback").await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[triage] haiku turn failed: {} — approving", e);
                                    let _ = supabase.mark_triage_processed(&id, "approved_fallback").await;
                                }
                            }
                        }
                    }
                }
            }
            Ok(_) => {} // empty queue, nothing to do
            Err(e) => eprintln!("[triage] queue read error: {}", e),
        }

        sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn post_to_discord(http: &Client, token: &str, channel_id: &str, content: &str) {
    // Discord has a 2000 char limit per message
    let chunks: Vec<&str> = content.as_bytes()
        .chunks(1900)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect();

    for chunk in chunks {
        let _ = http
            .post(format!("{}/channels/{}/messages", DISCORD_API, channel_id))
            .header("Authorization", format!("Bot {}", token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "content": chunk }))
            .send()
            .await;
    }
}

/// Extract the first valid JSON object from Haiku's response.
fn extract_json(text: &str) -> Option<TriageResult> {
    // Find JSON block
    let start = text.find('{')?;
    let end = text.rfind('}')? + 1;
    let json_str = &text[start..end];
    serde_json::from_str(json_str).ok()
}
