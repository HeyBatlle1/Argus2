//! Argus Core - Agent orchestration, tools, and shell policy
//!
//! This crate contains the shared logic between all Argus frontends
//! (TUI, Telegram, future API server, etc.)

pub mod agent;
pub mod mcp;
pub mod shell;
pub mod tools;

pub use agent::{AgentConfig, AgentEvent, run_agent_turn};
pub use mcp::McpClient;
pub use shell::ShellPolicy;
pub use tools::{MemoryBackend, MemoryRecord};
