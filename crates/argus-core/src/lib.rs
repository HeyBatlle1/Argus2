pub mod agent;
pub mod embedding;
pub mod mcp;
pub mod shell;
pub mod skills;
pub mod supabase;
pub mod tools;

pub use agent::{AgentConfig, AgentEvent, ConversationMessage, run_agent_turn, MODEL_HAIKU, MODEL_SONNET, MODEL_OPUS, MODEL_GROK, MODEL_GROK_FAST, MODEL_GROK_MULTI, MODEL_GEMINI, model_label};
pub use embedding::{EmbeddingClient, SemanticResult, EMBEDDING_MODEL};
pub use mcp::McpClient;
pub use shell::{ShellPolicy, PermissionPrompter, TelegramPrompter};
pub use skills::{SkillsClient, NewSkill};
pub use supabase::{SupabaseClient, DiscourseRecord};
pub use tools::{MemoryBackend, MemoryRecord};
