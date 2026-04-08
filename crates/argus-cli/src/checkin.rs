//! Autonomous check-in loop
//!
//! On startup, reads argus_checkin_config from Supabase.
//! Fires every interval_minutes, respects quiet hours.
//! Each check-in:
//!   1. Collects system health
//!   2. Reads recent memories from Supabase
//!   3. Checks upcoming schedule
//!   4. Formats a brief Telegram message
//!   5. Sends it
//!   6. Writes to argus_checkin_log

use argus_core::supabase::{CheckinLogEntry, SupabaseClient};
use chrono::{Local, NaiveTime};
use reqwest::Client;
use tokio::time::{sleep, Duration};

const TELEGRAM_API: &str = "https://api.telegram.org";

/// Entry point — spawns the check-in loop as a background task.
/// Returns immediately; caller must ensure the runtime stays alive.
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
    // Read config once at startup; refresh each cycle so config changes take effect
    let client = Client::new();

    loop {
        let config = supabase.read_checkin_config().await;

        if config.telegram_enabled && !in_quiet_hours(&config) {
            let health = collect_system_health().await;
            let schedule_summary = read_schedule_summary(&supabase).await;
            let message = format_checkin_message(&health, &schedule_summary, &config.checkin_type);

            if let Err(e) = send_telegram_message(&client, &bot_token, chat_id, &message).await {
                eprintln!("[checkin] Failed to send Telegram message: {}", e);
            } else {
                let entry = CheckinLogEntry {
                    checkin_type: config.checkin_type.clone(),
                    status: "sent".to_string(),
                    message_sent: message,
                    system_health: Some(serde_json::to_value(&health).unwrap_or_default()),
                };
                if let Err(e) = supabase.write_checkin_log(&entry).await {
                    eprintln!("[checkin] Failed to write checkin log: {}", e);
                }
            }
        }

        let interval = Duration::from_secs(config.interval_minutes.max(1) as u64 * 60);
        sleep(interval).await;
    }
}

/// Returns true if current local time falls within the configured quiet hours.
fn in_quiet_hours(config: &argus_core::supabase::CheckinConfig) -> bool {
    let now = Local::now().time();
    let start_str = config.quiet_hours_start.as_deref().unwrap_or("23:00");
    let end_str   = config.quiet_hours_end.as_deref().unwrap_or("07:00");

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

    // Quiet window crosses midnight (e.g. 23:00 → 07:00)
    if start > end {
        now >= start || now < end
    } else {
        now >= start && now < end
    }
}

#[derive(Debug, serde::Serialize)]
struct SystemHealth {
    timestamp: String,
    containers: String,
    disk: String,
    memory: String,
}

async fn collect_system_health() -> SystemHealth {
    let run = |cmd: &str| -> String {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unavailable".to_string())
    };

    SystemHealth {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S %Z").to_string(),
        containers: run("docker ps --format '{{.Names}} ({{.Status}})' 2>/dev/null | head -5"),
        disk: run("df -h / 2>/dev/null | tail -1 | awk '{print $3\"/\"$2\" used, \"$4\" free\"}'"),
        memory: run("vm_stat 2>/dev/null | head -3 | tail -1 | awk '{print $3\" pages free\"}'"),
    }
}

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
                let time  = r["scheduled_time"].as_str().unwrap_or("");
                Some(format!("• {} at {}", title, time))
            }).collect();
            if items.is_empty() { String::new() } else { items.join("\n") }
        }
    }
}

fn format_checkin_message(health: &SystemHealth, schedule: &str, checkin_type: &str) -> String {
    let mut msg = format!("Argus check-in — {}\n\n", health.timestamp);

    if checkin_type != "silent" {
        msg.push_str(&format!("Containers:\n{}\n\n", health.containers));
        msg.push_str(&format!("Disk: {}\n", health.disk));
        msg.push_str(&format!("Memory: {}\n", health.memory));
    }

    if !schedule.is_empty() {
        msg.push_str(&format!("\nUpcoming:\n{}", schedule));
    }

    msg
}

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
            "text": text,
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
