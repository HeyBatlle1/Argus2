//! Argus Core - Agent orchestration, tools, and shell policy
//!
//! Shared logic between all Argus frontends (TUI, Telegram, daemon, etc.)

pub mod agent;
pub mod mcp;
pub mod shell;
pub mod tools;

pub use agent::{AgentConfig, AgentEvent, ConversationMessage, run_agent_turn, MODEL_HAIKU, MODEL_GROK};
pub use mcp::McpClient;
pub use shell::ShellPolicy;
pub use tools::{MemoryBackend, MemoryRecord};
