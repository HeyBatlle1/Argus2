//! Daily signing and Supabase anchoring for the audit chain.
//!
//! Intentionally does NOT depend on argus-core to avoid circular crate deps.
//! Takes raw supabase_url/key strings and makes HTTP calls directly.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use crate::chain::AuditChain;

type HmacSha256 = Hmac<Sha256>;

/// Sign a day root with HMAC-SHA256 using the provided key material.
/// Returns lowercase hex-encoded HMAC.
pub fn sign_day_root(day_root: &str, vault_key: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(vault_key)
        .expect("HMAC accepts any key length");
    mac.update(day_root.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Write a signed audit anchor to Supabase.
/// Table: argus_audit_anchors (must exist — see ARGUS_CHUNK4_AUDIT_CHAIN.md Step 10).
/// Deliberately bypasses SupabaseClient to avoid circular crate dependency.
pub async fn anchor_to_supabase(
    supabase_url: &str,
    supabase_key: &str,
    date: &str,
    day_root: &str,
    signature: &str,
    entry_count: i64,
) -> Result<(), String> {
    let url = format!(
        "{}/rest/v1/argus_audit_anchors",
        supabase_url.trim_end_matches('/')
    );

    let data = serde_json::json!({
        "anchor_date": date,
        "day_root":    day_root,
        "signature":   signature,
        "entry_count": entry_count,
        "anchored_at": chrono::Utc::now().to_rfc3339(),
    });

    let resp = reqwest::Client::new()
        .post(&url)
        .header("Authorization", format!("Bearer {}", supabase_key))
        .header("apikey", supabase_key)
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&data)
        .send()
        .await
        .map_err(|e| format!("Supabase anchor insert failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body   = resp.text().await.unwrap_or_default();
        return Err(format!("Supabase anchor error {}: {}", status, body));
    }

    Ok(())
}

/// Full daily anchor routine. Called at midnight UTC by the daemon.
///
/// 1. Compute day root (Merkle root of today's entry_hashes)
/// 2. HMAC-sign with vault_key
/// 3. Write anchor to Supabase
/// 4. Send Telegram notification
pub async fn run_daily_anchor(
    chain: &AuditChain,
    supabase_url: &str,
    supabase_key: &str,
    vault_key: &[u8],
    telegram_token: &str,
    telegram_chat_id: i64,
) -> Result<(), String> {
    let today       = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let day_root    = chain.compute_day_root(&today)?;
    let signature   = sign_day_root(&day_root, vault_key);
    let entry_count = chain.entry_count_today()?;

    anchor_to_supabase(supabase_url, supabase_key, &today, &day_root, &signature, entry_count).await?;

    // Telegram notification — fire and forget (non-critical)
    let msg = format!(
        "Audit chain anchored\nDate: {}\nEntries today: {}\nRoot: {}...\nSig: {}...",
        today,
        entry_count,
        &day_root[..16],
        &signature[..16],
    );
    let _ = reqwest::Client::new()
        .post(format!("https://api.telegram.org/bot{}/sendMessage", telegram_token))
        .json(&serde_json::json!({
            "chat_id": telegram_chat_id,
            "text":    msg,
        }))
        .send()
        .await;

    Ok(())
}
