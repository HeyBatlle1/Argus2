// Faithful Argus protocol types (same as before for compatibility) + a few Nexus extensions
export type EyeState = 'watching' | 'thinking' | 'executing' | 'complete';
export type ModelId = 'claude-haiku' | 'claude-sonnet' | 'claude-opus' | 'grok' | 'grok-build' | 'grok-multi' | 'gemini-flash';
export type AccessTier = 'royal' | 'allied' | 'guest';

export type ArtifactType = 'html' | 'svg' | 'markdown' | 'python' | 'javascript' | string;

export interface Artifact {
  id: string; type: ArtifactType; title: string; content: string;
}

export interface Message {
  id: string; role: 'user' | 'assistant'; content: string; timestamp: Date;
  toolCalls?: ToolCall[]; artifacts?: Artifact[];
  grokEvidence?: { confidence: number; notes?: string };
}

export interface ToolCall {
  id: string; name: string; args: Record<string, unknown>;
  result?: string; success?: boolean;
  state: 'pending' | 'executing' | 'complete' | 'error';
  startedAt?: Date; completedAt?: Date;
}

export interface Tool { name: string; label: string; state: 'idle'|'active'|'complete'|'error'; callCount: number; }

export interface Memory { id: string; content: string; type: string; importance: number; createdAt: Date; tags?: string[]; }
export interface Skill { id: string; name: string; description: string; useCount: number; learnedAt: string; }
export interface ActivityEntry { id: string; kind: string; label: string; ts: string; }
export interface ScheduledTask { id: string; agent: ModelId; runAt: string | null; description: string; status: string; }

export interface Conversation { id: string; title: string; model: string | null; messageCount: number; lastActiveAt: string; }

export type ClientMessage =
  | { type: 'user_message'; content: string }
  | { type: 'switch_model'; model: ModelId }
  | { type: 'schedule_task'; agent: string; run_at: string | null; description: string }
  | { type: 'new_conversation' }
  | { type: 'load_conversation'; id: string };

export type ServerMessage =
  | { type: 'thinking' }
  | { type: 'connected'; version: string; model: string; vault_keys: string[]; mcp_servers: string[] }
  | { type: 'tool_call'; name: string; args: any; callId: string }
  | { type: 'tool_result'; name: string; result: string; success: boolean; callId: string }
  | { type: 'response_chunk'; content: string }
  | { type: 'response_complete'; content: string }
  | { type: 'status'; eye_state: EyeState; model: ModelId }
  | { type: 'memory_update'; memories: Memory[] }
  | { type: 'skills_update'; skills: Skill[] }
  | { type: 'activity_update'; entries: ActivityEntry[] }
  | { type: 'task_scheduled'; id: string; agent: string; run_at: string | null; description: string }
  | { type: 'error'; message: string };
