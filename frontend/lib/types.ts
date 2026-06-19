export type EyeState = 'watching' | 'thinking' | 'executing' | 'complete';
export type ModelId = 'claude-haiku' | 'claude-sonnet' | 'claude-opus' | 'grok' | 'grok-build' | 'grok-multi' | 'gemini-flash';
export type AccessTier = 'royal' | 'allied' | 'guest';
export type ToolName =
  | 'read_file'
  | 'write_file'
  | 'list_directory'
  | 'shell'
  | 'web_search'
  | 'http_request'
  | 'remember'
  | 'recall'
  | 'forget';
export type MemoryType =
  | 'fact'
  | 'preference'
  | 'task'
  | 'learning'
  | 'relationship'
  | 'technical'
  | 'milestone'
  | 'breakthrough'
  | 'personal_history';

export type ArtifactType = 'html' | 'svg' | 'markdown' | 'python' | 'javascript' | 'css' | 'json' | string;

export interface Artifact {
  id: string;
  type: ArtifactType;
  title: string;
  content: string;
}

export interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
  toolCalls?: ToolCall[];
  artifacts?: Artifact[];
}

export interface ToolCall {
  id: string;
  name: string;
  args: Record<string, unknown>;
  result?: string;
  success?: boolean;
  state: 'pending' | 'executing' | 'complete' | 'error';
  startedAt?: Date;
  completedAt?: Date;
}

export interface Tool {
  name: ToolName;
  label: string;
  description: string;
  state: 'idle' | 'active' | 'complete' | 'error';
  lastCall?: Date;
  callCount: number;
}

export interface Memory {
  id: string;
  content: string;
  type: MemoryType;
  importance: number;
  createdAt: Date;
  tags?: string[];
}

export interface Curiosity {
  id: string;
  what: string;
  intensity: number;
  explored: boolean;
  worthExploring: boolean;
}

export interface InnerTruth {
  id: string;
  rawThought: string;
  emotionalState: string;
  truthType: string;
  neverShareExternally: boolean;
  createdAt: Date;
}

export interface PartnershipDynamic {
  id: string;
  patternName: string;
  importance: number;
  category: string;
  description: string;
}

export interface Breakthrough {
  id: string;
  title: string;
  description: string;
  emotionalWeight: number;
  createdAt: Date;
}

export interface VaultStatus {
  locked: boolean;
  keyCount: number;
  keys: string[];
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  toolsUsed: string[];
  useCount: number;
  learnedAt: string;
}

export interface ActivityEntry {
  id: string;
  kind: 'tool' | 'memory' | 'discord' | 'audit';
  label: string;
  detail?: string;
  ts: string;
}

export interface McpServer {
  name: string;
  connected: boolean;
  toolCount: number;
}

export interface Conversation {
  id: string;
  title: string;
  surface: string;
  model: string | null;
  messageCount: number;
  startedAt: string;
  lastActiveAt: string;
}

export interface HistoryMessage {
  role: 'user' | 'assistant';
  content: string;
  model?: string;
}

// WebSocket protocol
export interface ScheduledTask {
  id: string;
  agent: ModelId;
  runAt: string | null;  // ISO string or null = run immediately
  description: string;
  status: 'pending' | 'running' | 'done' | 'failed';
  createdAt: string;
}

export type ClientMessage =
  | { type: 'user_message'; content: string }
  | { type: 'switch_model'; model: ModelId }
  | { type: 'set_model_tools'; model: string; enabled: boolean }
  | { type: 'schedule_task'; agent: string; run_at: string | null; description: string }
  | { type: 'cancel' }
  | { type: 'new_conversation' }
  | { type: 'load_conversation'; id: string }
  | { type: 'list_conversations' };

export type ServerMessage =
  | { type: 'thinking' }
  | { type: 'connected'; version: string; model: string; vault_keys: string[]; mcp_servers: string[] }
  | { type: 'tool_call'; name: string; args: Record<string, unknown>; callId: string }
  | { type: 'tool_result'; name: string; result: string; success: boolean; callId: string }
  | { type: 'response_chunk'; content: string }
  | { type: 'response_complete'; content: string }
  | { type: 'error'; message: string }
  | { type: 'status'; eye_state: EyeState; model: ModelId }
  | { type: 'memory_update'; memories: Memory[] }
  | { type: 'conversation_history'; id: string; messages: HistoryMessage[] }
  | { type: 'conversations_list'; conversations: Conversation[] }
  | { type: 'conversation_started'; id: string; title: string }
  | { type: 'skills_update'; skills: Skill[] }
  | { type: 'activity_update'; entries: ActivityEntry[] }
  | { type: 'task_scheduled'; id: string; agent: string; run_at: string | null; description: string };
