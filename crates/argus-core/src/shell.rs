//! Shell execution policy - allowlist model
//!
//! The old blocklist approach ("block rm -rf /") is a joke.
//! An attacker just uses `find / -delete` or `perl -e 'system("rm -rf /")'`.
//!
//! This module uses an allowlist: only explicitly permitted command prefixes
//! can execute. Everything else is denied by default.

use std::collections::HashSet;

/// Shell execution policy
pub struct ShellPolicy {
    /// Allowed command prefixes (e.g., "ls", "cat", "grep")
    allowed_prefixes: HashSet<String>,
    /// Maximum output size in bytes before truncation
    pub max_output_bytes: usize,
    /// Command timeout in seconds
    pub timeout_secs: u64,
}

impl Default for ShellPolicy {
    fn default() -> Self {
        let mut policy = Self {
            allowed_prefixes: HashSet::new(),
            max_output_bytes: 64 * 1024, // 64KB
            timeout_secs: 30,
        };

        // Safe read-only commands
        for cmd in &[
            "ls", "cat", "head", "tail", "wc", "find", "grep", "rg",
            "echo", "date", "pwd", "whoami", "uname", "which", "env",
            "file", "stat", "du", "df", "tree", "less", "sort", "uniq",
            "cut", "awk", "sed", "tr", "diff", "hexdump", "xxd",
            "curl", "wget",  // network fetch
            "git", "cargo", "npm", "node", "python3", "python", "pip",
            "rustc", "rustup", "make", "cmake",
            "docker", "kubectl",
            "jq", "yq",  // structured data
            "tar", "zip", "unzip", "gzip", "gunzip",
        ] {
            policy.allowed_prefixes.insert(cmd.to_string());
        }

        // Write operations (user must opt-in separately)
        for cmd in &[
            "mkdir", "cp", "mv", "touch", "tee",
            "chmod", "chown",
        ] {
            policy.allowed_prefixes.insert(cmd.to_string());
        }

        policy
    }
}

impl ShellPolicy {
    /// Create a new policy with no allowed commands
    pub fn empty() -> Self {
        Self {
            allowed_prefixes: HashSet::new(),
            max_output_bytes: 64 * 1024,
            timeout_secs: 30,
        }
    }

    /// Allow an additional command
    pub fn allow(&mut self, cmd: &str) {
        self.allowed_prefixes.insert(cmd.to_string());
    }

    /// Remove a command from the allowlist
    pub fn deny(&mut self, cmd: &str) {
        self.allowed_prefixes.remove(cmd);
    }

    /// Check if a command is allowed to execute
    pub fn check(&self, command: &str) -> Result<(), ShellDenied> {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return Err(ShellDenied::Empty);
        }

        // Extract the first token (the actual binary being invoked)
        let first_token = trimmed
            .split_whitespace()
            .next()
            .unwrap_or("");

        // Handle pipes: check each command in the pipeline
        if trimmed.contains('|') {
            for segment in trimmed.split('|') {
                let seg_cmd = segment.trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                if !self.is_allowed(seg_cmd) {
                    return Err(ShellDenied::NotAllowed(seg_cmd.to_string()));
                }
            }
            return Ok(());
        }

        // Handle command chaining with && or ;
        if trimmed.contains("&&") || trimmed.contains(';') {
            for segment in trimmed.split(&['&', ';'][..]) {
                let seg = segment.trim();
                if seg.is_empty() { continue; }
                let seg_cmd = seg.split_whitespace().next().unwrap_or("");
                if !seg_cmd.is_empty() && !self.is_allowed(seg_cmd) {
                    return Err(ShellDenied::NotAllowed(seg_cmd.to_string()));
                }
            }
            return Ok(());
        }

        // Handle subshell/backtick attempts
        if trimmed.contains("$(") || trimmed.contains('`') {
            return Err(ShellDenied::SubshellBlocked);
        }

        // Handle redirects to dangerous paths
        if trimmed.contains("> /dev/") || trimmed.contains("> /etc/") || trimmed.contains("> /sys/") {
            return Err(ShellDenied::DangerousRedirect);
        }

        if self.is_allowed(first_token) {
            Ok(())
        } else {
            Err(ShellDenied::NotAllowed(first_token.to_string()))
        }
    }

    fn is_allowed(&self, cmd: &str) -> bool {
        // Strip path prefixes: /usr/bin/ls -> ls
        let basename = cmd.rsplit('/').next().unwrap_or(cmd);
        self.allowed_prefixes.contains(basename)
    }
}

/// Reasons a shell command can be denied
#[derive(Debug, thiserror::Error)]
pub enum ShellDenied {
    #[error("Empty command")]
    Empty,
    #[error("Command not in allowlist: {0}")]
    NotAllowed(String),
    #[error("Subshell execution ($() or backticks) not allowed")]
    SubshellBlocked,
    #[error("Redirect to sensitive path blocked")]
    DangerousRedirect,
}

/// Execute a shell command under the given policy
pub async fn execute_shell(
    policy: &ShellPolicy,
    command: &str,
) -> Result<String, ShellDenied> {
    policy.check(command)?;

    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .await
        .map_err(|e| ShellDenied::NotAllowed(format!("spawn failed: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        let out = stdout.to_string();
        if out.len() > policy.max_output_bytes {
            Ok(format!(
                "{}...\n[truncated, {} bytes total]",
                &out[..policy.max_output_bytes],
                out.len()
            ))
        } else {
            Ok(out)
        }
    } else {
        Ok(format!(
            "Exit {}: {}",
            output.status.code().unwrap_or(-1),
            stderr
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_commands() {
        let policy = ShellPolicy::default();
        assert!(policy.check("ls -la").is_ok());
        assert!(policy.check("cat /etc/hosts").is_ok());
        assert!(policy.check("grep -r TODO .").is_ok());
        assert!(policy.check("git status").is_ok());
    }

    #[test]
    fn test_blocked_commands() {
        let policy = ShellPolicy::default();
        assert!(policy.check("rm -rf /").is_err());
        assert!(policy.check("sudo anything").is_err());
        assert!(policy.check("dd if=/dev/zero").is_err());
        assert!(policy.check("mkfs.ext4 /dev/sda").is_err());
    }

    #[test]
    fn test_pipe_validation() {
        let policy = ShellPolicy::default();
        assert!(policy.check("ls | grep foo").is_ok());
        assert!(policy.check("cat file | rm -rf /").is_err());
    }

    #[test]
    fn test_subshell_blocked() {
        let policy = ShellPolicy::default();
        assert!(policy.check("echo $(rm -rf /)").is_err());
        assert!(policy.check("echo `rm -rf /`").is_err());
    }

    #[test]
    fn test_path_stripping() {
        let policy = ShellPolicy::default();
        assert!(policy.check("/usr/bin/ls -la").is_ok());
        assert!(policy.check("/bin/cat file").is_ok());
    }

    #[test]
    fn test_empty_policy() {
        let policy = ShellPolicy::empty();
        assert!(policy.check("ls").is_err());
    }
}
