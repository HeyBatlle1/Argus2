//! Autonomous check-in loop — alert-only mode
//!
//! Fires every interval_minutes, respects quiet hours.
//! Sends a Telegram message ONLY when something needs attention:
//!   • Disk usage > 80%
//!   • Memory usage > 90%
//!   • Any container is unhealthy or exited unexpectedly
//!
//! Additionally sends a daily health summary once per day around midnight
//! (within the next check-in window after 00:00) — one message regardless
//! of health status, so there's always a daily heartbeat.
//!
//! This reduces Telegram noise from ~12 messages/day to 1/day (daily summary)
//! unless something actually needs attention.

use argus_core::supabase::{CheckinLogEntry, SupabaseClient};
use chrono::{Local, NaiveDate, NaiveTime};
use reqwest::Client;
use tokio::time::{sleep, Duration};

const TELEGRAM_API: &str = "https://api.telegram.org";
const DISK_ALERT_PCT: u8 = 80;
const MEM_ALERT_PCT: u8 = 90;

/// Entry point — spawns the check-in loop as a background task.
pub fn spawn_checkin_loop(
    supabase: SupabaseClient,
    bot_token: String,
    chat_id: i64,
) {
    tokio::spawn(async move {
        run_checkin_loop(supabase, bot_token, chat_id).await;
    });
}

async fn run_checkin_loop(supabase: SupabaseClient, bot_token: String, chat_id: i64) {
    let client = Client::new();
    let mut last_daily: Option<NaiveDate> = None;

    loop {
        let config = supabase.read_checkin_config().await;

        if config.telegram_enabled && !in_quiet_hours(&config) {
            let health = collect_system_health().await;
            let today = Local::now().date_naive();

            let needs_alert = health.disk_pct > DISK_ALERT_PCT
                || health.mem_pct > MEM_ALERT_PCT
                || health.has_unhealthy_container;

            // Daily summary fires once per calendar day (first check-in after midnight)
            let needs_daily = last_daily.map_or(true, |d| d < today);

            if needs_alert || needs_daily {
                let schedule_summary = read_schedule_summary(&supabase).await;
                let message = format_checkin_message(
                    &health,
                    &schedule_summary,
                    needs_alert,
                    needs_daily,
                );

                if let Err(e) = send_telegram_message(&client, &bot_token, chat_id, &message).await
                {
                    eprintln!("[checkin] Failed to send Telegram message: {}", e);
                } else {
                    if needs_daily {
                        last_daily = Some(today);
                    }

                    let entry = CheckinLogEntry {
                        checkin_type: config.checkin_type.clone(),
                        status: if needs_alert { "alert" } else { "daily" }.to_string(),
                        message_sent: message,
                        system_health: Some(
                            serde_json::to_value(&health).unwrap_or_default(),
                        ),
                    };
                    if let Err(e) = supabase.write_checkin_log(&entry).await {
                        eprintln!("[checkin] Failed to write checkin log: {}", e);
                    }
                }
            }
            // else: everything healthy, not daily time — silent pass
        }

        let interval = Duration::from_secs(config.interval_minutes.max(1) as u64 * 60);
        sleep(interval).await;
    }
}

fn in_quiet_hours(config: &argus_core::supabase::CheckinConfig) -> bool {
    let now = Local::now().time();
    let start_str = config.quiet_hours_start.as_deref().unwrap_or("23:00");
    let end_str = config.quiet_hours_end.as_deref().unwrap_or("07:00");

    let parse = |s: &str| -> Option<NaiveTime> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 { return None; }
        let h: u32 = parts[0].parse().ok()?;
        let m: u32 = parts[1].parse().ok()?;
        NaiveTime::from_hms_opt(h, m, 0)
    };

    let (start, end) = match (parse(start_str), parse(end_str)) {
        (Some(s), Some(e)) => (s, e),
        _ => return false,
    };

    if start > end {
        now >= start || now < end
    } else {
        now >= start && now < end
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

fn format_checkin_message(
    health: &SystemHealth,
    schedule: &str,
    is_alert: bool,
    is_daily: bool,
) -> String {
    let mut msg = if is_alert {
        format!("⚠️ Argus Alert — {}\n\n", health.timestamp)
    } else {
        format!("✅ Argus Daily Summary — {}\n\n", health.timestamp)
    };

    // Alert details — be specific about what's wrong
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
        msg.push_str(&format!("\n🔴 Container issue detected:\n{}\n", health.containers));
    } else if is_daily {
        msg.push_str(&format!("\nContainers:\n{}\n", health.containers));
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
