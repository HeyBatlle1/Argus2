import { EyeState, ToolName, Tool } from './types';

export const EYE_SYMBOLS: Record<EyeState, string> = {
  watching:  '◉',
  thinking:  '◎',
  executing: '⊙',
  complete:  '✦',
};

export const EYE_LABELS: Record<EyeState, string> = {
  watching:  'WATCHING',
  thinking:  'THINKING',
  executing: 'EXECUTING',
  complete:  'COMPLETE',
};

export const EYE_COLORS: Record<EyeState, string> = {
  watching:  '#4a7c59',
  thinking:  '#c9a84c',
  executing: '#4a8fc4',
  complete:  '#ffffff',
};

export const TOOL_BORDER_COLORS = {
  executing: '#4a8fc4',
  complete:  '#4a7c59',
  error:     '#8b1a1a',
  pending:   '#1a1a2e',
};

export const MEMORY_TYPE_COLORS: Record<string, string> = {
  fact:             '#c9a84c',
  preference:       '#4a7c59',
  task:             '#4a8fc4',
  learning:         '#8b5cf6',
  relationship:     '#ec4899',
  technical:        '#06b6d4',
  milestone:        '#f59e0b',
  breakthrough:     '#ef4444',
  personal_history: '#a78bfa',
};

export const DEFAULT_TOOLS: Tool[] = [
  { name: 'read_file',      label: 'read_file',      description: 'Read file from filesystem',   state: 'idle', callCount: 0 },
  { name: 'write_file',     label: 'write_file',     description: 'Write file to filesystem',    state: 'idle', callCount: 0 },
  { name: 'list_directory', label: 'list_directory', description: 'List directory contents',     state: 'idle', callCount: 0 },
  { name: 'shell',          label: 'shell',          description: 'Execute shell command',        state: 'idle', callCount: 0 },
  { name: 'web_search',     label: 'web_search',     description: 'Search web via Brave',         state: 'idle', callCount: 0 },
  { name: 'http_request',   label: 'http_request',   description: 'Call any HTTP endpoint',       state: 'idle', callCount: 0 },
  { name: 'remember',       label: 'remember',       description: 'Store to persistent memory',   state: 'idle', callCount: 0 },
  { name: 'recall',         label: 'recall',         description: 'Search persistent memory',     state: 'idle', callCount: 0 },
  { name: 'forget',         label: 'forget',         description: 'Remove from memory',           state: 'idle', callCount: 0 },
];

export const WS_URL = process.env.NEXT_PUBLIC_WS_URL ?? null;
