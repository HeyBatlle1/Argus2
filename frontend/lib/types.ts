export type EyeState = 'watching' | 'thinking' | 'executing' | 'complete';
export type ModelId = 'claude-haiku' | 'claude-sonnet' | 'claude-opus' | 'grok' | 'gemini-flash';
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

export interface Message {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
  toolCalls?: ToolCall[];
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

export interface McpServer {
  name: string;
  connected: boolean;
  toolCount: number;
}

// WebSocket protocol
export type ClientMessage =
  | { type: 'user_message'; content: string }
  | { type: 'switch_model'; model: ModelId }
  | { type: 'cancel' };

export type ServerMessage =
  | { type: 'thinking' }
  | { type: 'connected'; version: string; model: string }
  | { type: 'tool_call'; name: string; args: Record<string, unknown>; callId: string }
  | { type: 'tool_result'; name: string; result: string; success: boolean; callId: string }
  | { type: 'response_chunk'; content: string }
  | { type: 'response_complete'; content: string }
  | { type: 'error'; message: string }
  | { type: 'status'; eye_state: EyeState; model: ModelId }
  | { type: 'memory_update'; memories: Memory[] };
