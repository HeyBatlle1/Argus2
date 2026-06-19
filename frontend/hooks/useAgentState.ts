'use client';

import { create } from 'zustand';
import {
  EyeState, ModelId, AccessTier,
  Message, Tool, ToolCall,
  Memory, Curiosity, InnerTruth, PartnershipDynamic, Breakthrough,
  Conversation, Skill, ActivityEntry, ScheduledTask,
  ServerMessage,
} from '@/lib/types';
import { ArgusConnection } from '@/lib/connection';
import { parseArtifacts } from '@/lib/artifacts';
import { DEFAULT_TOOLS, WS_URL } from '@/lib/constants';
import { getModelTier } from '@/lib/models';
import { PRIMARY_CODER } from '@/lib/builder';
import { RealConnection } from './useWebSocket';

// ─── Dev-mode seed data (only loaded when no WS_URL is set) ───────────────
// Import paths from lib/dev/ are intentionally isolated — never referenced
// outside this conditional block.

const IS_DEV = !WS_URL;

function loadDevData() {
  if (!IS_DEV) return { memories: [], curiosities: [], innerTruths: [], dynamics: [], breakthroughs: [], messages: [] };
  // Dynamic import of dev data — tree-shaken in prod builds
  const {
    DEV_MEMORIES, DEV_CURIOSITIES, DEV_INNER_TRUTHS, DEV_DYNAMICS, DEV_BREAKTHROUGHS,
  } = require('@/lib/dev/mock-connection');

  const seedMessages: Message[] = [
    {
      id: 'seed-1',
      role: 'user',
      content: 'Argus, what is your current operational status?',
      timestamp: new Date(Date.now() - 5 * 60 * 1000),
    },
    {
      id: 'seed-tc-1',
      role: 'assistant',
      content: '',
      timestamp: new Date(Date.now() - 4 * 60 * 1000),
      toolCalls: [{
        id: 'tc-seed-1',
        name: 'recall',
        args: { query: 'system status', limit: 5 },
        result: 'Found 5 memories: vault configured, model set, memory backend online.',
        success: true,
        state: 'complete' as const,
        startedAt: new Date(Date.now() - 4 * 60 * 1000 - 10000),
        completedAt: new Date(Date.now() - 4 * 60 * 1000),
      }],
    },
    {
      id: 'seed-2',
      role: 'assistant',
      content: `## Status: Dev Mode\n\n**Runtime:** argus-cli v0.1.0\n**Model:** Claude Haiku (Royal tier) via OpenRouter\n**Memory:** ${DEV_MEMORIES.length} dev entries loaded\n**Mode:** Mock — set \`NEXT_PUBLIC_WS_URL\` to connect the real backend\n\nThe hundred eyes are open. What needs doing?`,
      timestamp: new Date(Date.now() - 4 * 60 * 1000),
    },
  ];

  return {
    memories: DEV_MEMORIES as Memory[],
    curiosities: DEV_CURIOSITIES as Curiosity[],
    innerTruths: DEV_INNER_TRUTHS as InnerTruth[],
    dynamics: DEV_DYNAMICS as PartnershipDynamic[],
    breakthroughs: DEV_BREAKTHROUGHS as Breakthrough[],
    messages: seedMessages,
  };
}

const devData = loadDevData();

// ─── Store interface ───────────────────────────────────────────────────────

interface AgentStore {
  // Connection
  connected: boolean;
  wsLatency: number;
  startTime: Date;

  // Agent state
  eyeState: EyeState;
  activeModel: ModelId;
  accessTier: AccessTier;
  isStreaming: boolean;

  // Conversation
  messages: Message[];
  streamingContent: string;

  // Tools
  tools: Tool[];
  activeToolCalls: ToolCall[];

  // Memory (populated from backend in prod; dev data in dev mode)
  memories: Memory[];
  curiosities: Curiosity[];
  innerTruths: InnerTruth[];
  partnershipDynamics: PartnershipDynamic[];
  breakthroughs: Breakthrough[];

  // Vault + MCP live state (populated from Connected WS message)
  vaultKeys: string[];
  mcpServers: string[];

  // Conversation history
  conversations: Conversation[];
  currentConversationId: string | null;
  currentConversationTitle: string;

  // Skills + Activity
  skills: Skill[];
  activity: ActivityEntry[];

  // Tool toggles (allied models only; anthropic always on)
  toolsEnabled: Record<string, boolean>;

  // Task scheduler
  scheduledTasks: ScheduledTask[];

  // NexusCore pulse intensity (0-14, driven by tool activity)
  corePulse: number;

  // UI chrome (command palette + layout)
  mindView: 'mind' | 'field' | 'flow' | 'schedule';
  eyesCollapsed: boolean;
  mindCollapsed: boolean;

  // Internal
  _ws: ArgusConnection | null;

  // Actions
  sendMessage: (content: string) => void;
  setMindView: (view: 'mind' | 'field' | 'flow' | 'schedule') => void;
  setEyesCollapsed: (collapsed: boolean) => void;
  setMindCollapsed: (collapsed: boolean) => void;
  expandAllPanels: () => void;
  summonBuilder: () => void;
  switchModel: (model: ModelId) => void;
  setEyeState: (state: EyeState) => void;
  setModelTools: (model: string, enabled: boolean) => void;
  scheduleTask: (agent: string, runAt: string | null, description: string) => void;
  initConnection: () => void;
  newConversation: () => void;
  loadConversation: (id: string) => void;
  _handleServerMessage: (msg: ServerMessage) => void;
}

// ─── Store ─────────────────────────────────────────────────────────────────

export const useAgentStore = create<AgentStore>((set, get) => ({
  // Connection
  connected: false,
  wsLatency: 12,
  startTime: new Date(),

  // Agent state — prod starts watching, dev same
  eyeState: 'watching',
  activeModel: 'grok-build',
  accessTier: 'royal',
  isStreaming: false,

  // Conversation — empty in prod, seeded in dev
  messages: devData.messages,
  streamingContent: '',

  // Tools
  tools: DEFAULT_TOOLS,
  activeToolCalls: [],

  // Memory — empty in prod (backend populates), dev data otherwise
  memories: devData.memories,
  curiosities: devData.curiosities,
  innerTruths: devData.innerTruths,
  partnershipDynamics: devData.dynamics,
  breakthroughs: devData.breakthroughs,

  vaultKeys: [],
  mcpServers: [],

  conversations: [],
  currentConversationId: null,
  currentConversationTitle: '',

  skills: [],
  activity: [],

  toolsEnabled: {
    'grok': true,
    'grok-build': true,
    'grok-multi': true,
    'gemini-flash': true,
  },
  scheduledTasks: [],
  corePulse: 4,

  mindView: 'mind',
  eyesCollapsed: false,
  mindCollapsed: false,

  _ws: null,

  // ─── Server message handler ────────────────────────────────────────────

  _handleServerMessage: (msg: ServerMessage) => {
    switch (msg.type) {
      case 'connected':
        set((prev) => ({
          connected: true,
          vaultKeys: msg.vault_keys ?? [],
          mcpServers: msg.mcp_servers ?? [],
          // Sync to whatever model the daemon actually started with
          activeModel: (msg.model as ModelId) ?? prev.activeModel,
          accessTier: msg.model ? getModelTier(msg.model as ModelId) : prev.accessTier,
        }));
        break;

      case 'thinking':
        set({ eyeState: 'thinking', isStreaming: true, streamingContent: '' });
        break;

      case 'tool_call': {
        // Rust sends call_id (snake_case); TypeScript declared callId — handle both
        const callId = (msg as any).call_id ?? (msg as any).callId ?? 'unknown';
        const tc: ToolCall = {
          id: callId,
          name: msg.name,
          args: msg.args,
          state: 'executing',
          startedAt: new Date(),
        };
        const now = new Date();
        const entry: ActivityEntry = {
          id: 'act-' + callId,
          kind: 'tool',
          label: msg.name,
          ts: now.toISOString(),
        };
        set((prev) => ({
          eyeState: 'executing',
          activeToolCalls: [...prev.activeToolCalls, tc],
          corePulse: Math.min(14, prev.corePulse + 3),
          activity: [entry, ...prev.activity].slice(0, 50),
          tools: prev.tools.map((t) =>
            t.name === msg.name ? { ...t, state: 'active' as const } : t
          ),
          messages: [
            ...prev.messages,
            {
              id: 'tc-' + callId,
              role: 'assistant' as const,
              content: '',
              timestamp: now,
              toolCalls: [tc],
            },
          ],
        }));
        break;
      }

      case 'tool_result': {
        const now = new Date();
        const callId = (msg as any).call_id ?? (msg as any).callId ?? 'unknown';
        set((prev) => ({
          activeToolCalls: prev.activeToolCalls.map((tc) =>
            tc.id === callId
              ? { ...tc, result: msg.result, success: msg.success, state: 'complete' as const, completedAt: now }
              : tc
          ),
          corePulse: Math.max(2, prev.corePulse - 1),
          messages: prev.messages.map((m) => {
            if (!m.toolCalls?.some((tc) => tc.id === callId)) return m;
            return {
              ...m,
              toolCalls: m.toolCalls.map((tc) =>
                tc.id === callId
                  ? { ...tc, result: msg.result, success: msg.success, state: 'complete' as const, completedAt: now }
                  : tc
              ),
            };
          }),
          tools: prev.tools.map((t) =>
            t.name === msg.name
              ? { ...t, state: 'complete' as const, callCount: t.callCount + 1, lastCall: now }
              : t
          ),
        }));
        break;
      }

      case 'response_chunk':
        set((prev) => ({
          streamingContent: prev.streamingContent + msg.content,
          eyeState: 'thinking',
        }));
        break;

      case 'response_complete': {
        const { cleanText, artifacts } = parseArtifacts(msg.content);
        set((prev) => ({
          messages: [
            ...prev.messages,
            {
              id: 'resp-' + Date.now(),
              role: 'assistant' as const,
              content: cleanText,
              timestamp: new Date(),
              artifacts: artifacts.length > 0 ? artifacts : undefined,
            },
          ],
          streamingContent: '',
          isStreaming: false,
          eyeState: 'complete',
          activeToolCalls: [],
          corePulse: 3,
        }));
        setTimeout(() => set({ eyeState: 'watching' }), 1500);
        break;
      }

      case 'status':
        set({ eyeState: msg.eye_state, activeModel: msg.model, accessTier: getModelTier(msg.model) });
        break;

      case 'memory_update':
        set({ memories: msg.memories });
        break;

      case 'skills_update':
        set({ skills: msg.skills });
        break;

      case 'activity_update':
        set((prev) => ({ activity: [...msg.entries, ...prev.activity].slice(0, 50) }));
        break;

      case 'conversations_list':
        set({ conversations: msg.conversations });
        break;

      case 'conversation_started':
        set({
          currentConversationId: msg.id,
          currentConversationTitle: msg.title,
          messages: [],
        });
        break;

      case 'conversation_history': {
        // Rebuild Message objects from persisted role/content pairs.
        const loaded: Message[] = msg.messages.map((m, i) => ({
          id: `hist-${msg.id}-${i}`,
          role: m.role as 'user' | 'assistant',
          content: m.content,
          timestamp: new Date(),
        }));
        set({ currentConversationId: msg.id, messages: loaded });
        break;
      }

      case 'task_scheduled':
        set((prev) => ({
          scheduledTasks: [
            {
              id: msg.id,
              agent: msg.agent as ModelId,
              runAt: msg.run_at,
              description: msg.description,
              status: 'pending' as const,
              createdAt: new Date().toISOString(),
            },
            ...prev.scheduledTasks,
          ],
        }));
        break;

      case 'error':
        set((prev) => ({
          eyeState: 'watching',
          isStreaming: false,
          streamingContent: '',
          messages: [
            ...prev.messages,
            {
              id: 'err-' + Date.now(),
              role: 'assistant' as const,
              content: `**Error:** ${msg.message}`,
              timestamp: new Date(),
            },
          ],
        }));
        break;
    }
  },

  // ─── Init connection ───────────────────────────────────────────────────
  // Called once on mount. Uses real WS in prod, mock in dev.

  initConnection: () => {
    if (get()._ws) return;

    const handler = (msg: ServerMessage) => get()._handleServerMessage(msg);

    if (WS_URL) {
      // Production: real WebSocket to Rust backend
      const ws = new RealConnection(WS_URL, handler, (connected) => set({ connected }));
      set({ _ws: ws });
    } else {
      // Development: mock connection — no real API calls
      const { MockConnection } = require('@/lib/dev/mock-connection');
      const ws = new MockConnection(handler, get().activeModel) as ArgusConnection;
      set({ _ws: ws, connected: true });
    }
  },

  // ─── Actions ───────────────────────────────────────────────────────────

  sendMessage: (content: string) => {
    const store = get();
    if (!store._ws) store.initConnection();
    if (store.isStreaming) return;

    set((prev) => ({
      messages: [
        ...prev.messages,
        { id: 'user-' + Date.now(), role: 'user' as const, content, timestamp: new Date() },
      ],
    }));

    get()._ws?.send({ type: 'user_message', content });
  },

  switchModel: (model: ModelId) => {
    set({ activeModel: model, accessTier: getModelTier(model) });
    get()._ws?.send({ type: 'switch_model', model });
  },

  setEyeState: (state: EyeState) => set({ eyeState: state }),

  setModelTools: (model: string, enabled: boolean) => {
    set((prev) => ({ toolsEnabled: { ...prev.toolsEnabled, [model]: enabled } }));
    get()._ws?.send({ type: 'set_model_tools', model, enabled });
  },

  scheduleTask: (agent: string, runAt: string | null, description: string) => {
    get()._ws?.send({ type: 'schedule_task', agent, run_at: runAt, description });
  },

  newConversation: () => {
    get()._ws?.send({ type: 'new_conversation' });
  },

  loadConversation: (id: string) => {
    get()._ws?.send({ type: 'load_conversation', id });
  },

  setMindView: (view) => set({ mindView: view, mindCollapsed: false }),
  setEyesCollapsed: (collapsed) => set({ eyesCollapsed: collapsed }),
  setMindCollapsed: (collapsed) => set({ mindCollapsed: collapsed }),
  expandAllPanels: () => set({ eyesCollapsed: false, mindCollapsed: false }),

  summonBuilder: () => {
    const store = get();
    store.setModelTools(PRIMARY_CODER, true);
    store.switchModel(PRIMARY_CODER);
    set({ eyesCollapsed: false, mindCollapsed: false });
  },
}));
