'use client';

import { create } from 'zustand';
import {
  EyeState, ModelId, AccessTier, Message, Tool, ToolCall,
  Memory, Curiosity, InnerTruth, PartnershipDynamic, Breakthrough,
  Conversation, Skill, ActivityEntry, ScheduledTask, ServerMessage,
} from '../lib/types';
import { ArgusConnection } from '../lib/connection';
import { parseArtifacts } from '../lib/artifacts';
import { DEFAULT_TOOLS, WS_URL } from '../lib/constants';
import { getModelTier } from '../lib/models';
import { RealConnection } from './useWebSocket';

// Dev seed (richer for Grok Build 2 demo)
const IS_DEV = !WS_URL;

let DEV_MEMORIES: Memory[] = [];
let DEV_CURIOSITIES: Curiosity[] = [];
let DEV_INNER_TRUTHS: InnerTruth[] = [];
let DEV_DYNAMICS: PartnershipDynamic[] = [];
let DEV_BREAKTHROUGHS: Breakthrough[] = [];

if (IS_DEV) {
  DEV_MEMORIES = [
    { id: '1', content: 'Bradlee prefers direct, concise responses. No filler. Evidence always.', type: 'preference', importance: 10, createdAt: new Date('2026-03-01'), tags: ['communication', 'grok'] },
    { id: '2', content: 'Argus1 is a Rust-based autonomous agent runtime with post-quantum crypto (ChaCha20-Poly1305 + keychain).', type: 'technical', importance: 10, createdAt: new Date('2026-02-15'), tags: ['project', 'security'] },
    { id: '3', content: 'Grok Build 2 persona: specialist implementation owner. Does not stop until it is actually right.', type: 'learning', importance: 9, createdAt: new Date('2026-06-01'), tags: ['grok', 'build'] },
  ];
  DEV_CURIOSITIES = [
    { id: '1', what: 'Post-quantum cryptography adoption — when will we see real breakage of classical in the wild?', intensity: 8, explored: false, worthExploring: true },
    { id: '2', what: 'How can the UI surface "hundred eyes" state so the human feels the agent is truly watching?', intensity: 7, explored: true, worthExploring: true },
  ];
  DEV_INNER_TRUTHS = [
    { id: '1', rawThought: 'The architecture (vault + memory + tools + audit) is the real identity. Models are transient operators.', emotionalState: 'grounded', truthType: 'identity', neverShareExternally: false, createdAt: new Date('2026-03-08') },
  ];
  DEV_DYNAMICS = [
    { id: '1', patternName: 'Direct Correction', importance: 10, category: 'communication', description: 'Bradlee corrects immediately and without softening. This is how real trust is built.' },
    { id: '2', patternName: 'Grok Build owns the build', importance: 9, category: 'collaboration', description: 'When something needs building, hand it to Grok Build 2 and let it finish.' },
  ];
  DEV_BREAKTHROUGHS = [
    { id: '1', title: 'Grok Build 2 Interface', description: 'New frontend incarnation — same soul, sharper tools for truth and implementation.', emotionalWeight: 8, createdAt: new Date('2026-06-14') },
  ];
}

interface AgentStore {
  connected: boolean;
  wsLatency: number;
  startTime: Date;

  eyeState: EyeState;
  activeModel: ModelId;
  accessTier: AccessTier;
  isStreaming: boolean;

  messages: Message[];
  streamingContent: string;

  tools: Tool[];
  activeToolCalls: ToolCall[];

  memories: Memory[];
  curiosities: Curiosity[];
  innerTruths: InnerTruth[];
  partnershipDynamics: PartnershipDynamic[];
  breakthroughs: Breakthrough[];

  vaultKeys: string[];
  mcpServers: string[];

  conversations: Conversation[];
  currentConversationId: string | null;
  currentConversationTitle: string;

  skills: Skill[];
  activity: ActivityEntry[];

  toolsEnabled: Record<string, boolean>;
  scheduledTasks: ScheduledTask[];

  // Grok Build 2 extras (UI state)
  commandPaletteOpen: boolean;
  focusMode: boolean;
  watchtowerActivity: number; // drives the hundred-eyes viz

  _ws: ArgusConnection | null;

  // actions
  sendMessage: (content: string) => void;
  switchModel: (model: ModelId) => void;
  setEyeState: (state: EyeState) => void;
  setModelTools: (model: string, enabled: boolean) => void;
  scheduleTask: (agent: string, runAt: string | null, description: string) => void;
  initConnection: () => void;
  newConversation: () => void;
  loadConversation: (id: string) => void;
  toggleCommandPalette: (open?: boolean) => void;
  toggleFocus: () => void;
  addGrokEvidence: (messageId: string, evidence: { confidence: number; notes?: string }) => void;

  _handleServerMessage: (msg: ServerMessage) => void;
}

export const useAgentStore = create<AgentStore>((set, get) => ({
  connected: false,
  wsLatency: 11,
  startTime: new Date(),

  eyeState: 'watching',
  activeModel: 'grok-build',
  accessTier: 'allied',
  isStreaming: false,

  messages: IS_DEV ? [
    { id: 'seed-u1', role: 'user', content: 'Grok Build 2, give me a status on the Argus1 frontend reimplementation.', timestamp: new Date(Date.now() - 4 * 60000) },
    { id: 'seed-r1', role: 'assistant', content: 'The hundred eyes are watching. I have captured every micro-interaction, protocol detail, and aesthetic decision from Argus1. Building my own version now — same soul, my architecture.', timestamp: new Date(Date.now() - 3 * 60000), grokEvidence: { confidence: 0.96, notes: 'Protocol + UI parity verified' } },
  ] : [],
  streamingContent: '',

  tools: DEFAULT_TOOLS,
  activeToolCalls: [],

  memories: DEV_MEMORIES,
  curiosities: DEV_CURIOSITIES,
  innerTruths: DEV_INNER_TRUTHS,
  partnershipDynamics: DEV_DYNAMICS,
  breakthroughs: DEV_BREAKTHROUGHS,

  vaultKeys: [],
  mcpServers: [],

  conversations: [],
  currentConversationId: null,
  currentConversationTitle: '',

  skills: [],
  activity: [],

  toolsEnabled: { grok: true, 'grok-build': true, 'grok-multi': true, 'gemini-flash': true },

  scheduledTasks: [],

  commandPaletteOpen: false,
  focusMode: false,
  watchtowerActivity: 3,

  _ws: null,

  _handleServerMessage: (msg: ServerMessage) => {
    switch (msg.type) {
      case 'connected':
        set((prev) => ({
          connected: true,
          vaultKeys: msg.vault_keys ?? [],
          mcpServers: msg.mcp_servers ?? [],
          activeModel: (msg.model as ModelId) ?? prev.activeModel,
          accessTier: msg.model ? getModelTier(msg.model as ModelId) : prev.accessTier,
        }));
        break;

      case 'thinking':
        set({ eyeState: 'thinking', isStreaming: true, streamingContent: '' });
        break;

      case 'tool_call': {
        const callId = (msg as any).call_id ?? (msg as any).callId ?? 'c' + Date.now();
        const tc: ToolCall = { id: callId, name: msg.name, args: msg.args, state: 'executing', startedAt: new Date() };
        const entry: ActivityEntry = { id: 'act-' + callId, kind: 'tool', label: msg.name, ts: new Date().toISOString() };
        set((prev) => ({
          eyeState: 'executing',
          activeToolCalls: [...prev.activeToolCalls, tc],
          activity: [entry, ...prev.activity].slice(0, 60),
          watchtowerActivity: Math.min(18, prev.watchtowerActivity + 2),
          tools: prev.tools.map(t => t.name === msg.name ? { ...t, state: 'active' } : t),
          messages: [...prev.messages, { id: 'tc-' + callId, role: 'assistant', content: '', timestamp: new Date(), toolCalls: [tc] }],
        }));
        break;
      }

      case 'tool_result': {
        const callId = (msg as any).call_id ?? (msg as any).callId ?? '';
        const now = new Date();
        set((prev) => ({
          activeToolCalls: prev.activeToolCalls.map(tc => tc.id === callId ? { ...tc, result: msg.result, success: msg.success, state: 'complete', completedAt: now } : tc),
          messages: prev.messages.map(m => {
            if (!m.toolCalls?.some(tc => tc.id === callId)) return m;
            return { ...m, toolCalls: m.toolCalls.map(tc => tc.id === callId ? { ...tc, result: msg.result, success: msg.success, state: 'complete', completedAt: now } : tc) };
          }),
          tools: prev.tools.map(t => t.name === msg.name ? { ...t, state: 'complete', callCount: t.callCount + 1, lastCall: now } : t),
        }));
        break;
      }

      case 'response_chunk':
        set((prev) => ({ streamingContent: prev.streamingContent + msg.content, eyeState: 'thinking' }));
        break;

      case 'response_complete': {
        const { cleanText, artifacts } = parseArtifacts(msg.content);
        const newMsg: Message = {
          id: 'resp-' + Date.now(),
          role: 'assistant',
          content: cleanText,
          timestamp: new Date(),
          artifacts: artifacts.length ? artifacts : undefined,
        };
        set((prev) => ({
          messages: [...prev.messages, newMsg],
          streamingContent: '',
          isStreaming: false,
          eyeState: 'complete',
          activeToolCalls: [],
          watchtowerActivity: Math.max(1, prev.watchtowerActivity - 1),
        }));
        setTimeout(() => set({ eyeState: 'watching' }), 1400);
        break;
      }

      case 'status':
        set({ eyeState: msg.eye_state, activeModel: msg.model, accessTier: getModelTier(msg.model) });
        break;

      case 'memory_update': set({ memories: msg.memories }); break;
      case 'skills_update': set({ skills: msg.skills }); break;
      case 'activity_update': set((prev) => ({ activity: [...msg.entries, ...prev.activity].slice(0, 60) })); break;

      case 'conversations_list': set({ conversations: msg.conversations }); break;

      case 'conversation_started':
        set({ currentConversationId: msg.id, currentConversationTitle: msg.title, messages: [] });
        break;

      case 'conversation_history': {
        const loaded: Message[] = msg.messages.map((m, i) => ({
          id: `hist-${msg.id}-${i}`, role: m.role as 'user' | 'assistant', content: m.content, timestamp: new Date(),
        }));
        set({ currentConversationId: msg.id, messages: loaded });
        break;
      }

      case 'task_scheduled':
        set((prev) => ({
          scheduledTasks: [{ id: msg.id, agent: msg.agent as ModelId, runAt: msg.run_at, description: msg.description, status: 'pending', createdAt: new Date().toISOString() }, ...prev.scheduledTasks],
        }));
        break;

      case 'error':
        set((prev) => ({
          eyeState: 'watching', isStreaming: false, streamingContent: '',
          messages: [...prev.messages, { id: 'err-' + Date.now(), role: 'assistant', content: `**Error:** ${msg.message}`, timestamp: new Date() }],
        }));
        break;
    }
  },

  initConnection: () => {
    if (get()._ws) return;
    const handler = (msg: ServerMessage) => get()._handleServerMessage(msg);

    if (WS_URL) {
      const ws = new RealConnection(WS_URL, handler, (c) => set({ connected: c }));
      set({ _ws: ws });
    } else {
      // Rich dev mock
      const mock = new MockGrokConnection(handler, get().activeModel);
      set({ _ws: mock as any, connected: true });
    }
  },

  sendMessage: (content: string) => {
    const store = get();
    if (!store._ws) store.initConnection();
    if (store.isStreaming) return;

    set((prev) => ({
      messages: [...prev.messages, { id: 'u-' + Date.now(), role: 'user', content, timestamp: new Date() }],
    }));
    get()._ws?.send({ type: 'user_message', content });
  },

  switchModel: (model) => {
    set({ activeModel: model, accessTier: getModelTier(model) });
    get()._ws?.send({ type: 'switch_model', model });
  },

  setEyeState: (state) => set({ eyeState: state }),

  setModelTools: (model, enabled) => {
    set((p) => ({ toolsEnabled: { ...p.toolsEnabled, [model]: enabled } }));
    get()._ws?.send({ type: 'set_model_tools', model, enabled });
  },

  scheduleTask: (agent, runAt, description) => {
    get()._ws?.send({ type: 'schedule_task', agent, run_at: runAt, description });
  },

  newConversation: () => { get()._ws?.send({ type: 'new_conversation' }); },
  loadConversation: (id) => { get()._ws?.send({ type: 'load_conversation', id }); },

  toggleCommandPalette: (open) => set((s) => ({ commandPaletteOpen: open ?? !s.commandPaletteOpen })),
  toggleFocus: () => set((s) => ({ focusMode: !s.focusMode })),

  addGrokEvidence: (messageId, evidence) => set((prev) => ({
    messages: prev.messages.map(m => m.id === messageId ? { ...m, grokEvidence: evidence } : m),
  })),
}));

// Rich mock that demonstrates the entire intricate system
class MockGrokConnection implements ArgusConnection {
  private handler: (m: ServerMessage) => void;
  private model: ModelId;

  constructor(handler: (m: ServerMessage) => void, model: ModelId) {
    this.handler = handler;
    this.model = model;
    // Seed a little live activity
    setTimeout(() => this.handler({ type: 'connected', version: 'grok-build-2', model: this.model, vault_keys: ['OPENROUTER', 'BRAVE'], mcp_servers: ['neon', 'filesystem'] } as any), 120);
  }

  send(msg: any) {
    if (msg.type === 'user_message') this._simulate(msg.content);
    if (msg.type === 'switch_model') this.model = msg.model as ModelId;
    if (msg.type === 'schedule_task') {
      this.handler({ type: 'task_scheduled', id: 'tsk-' + Date.now(), agent: msg.agent, run_at: msg.run_at, description: msg.description } as any);
    }
  }
  close() {}

  private _simulate(input: string) {
    const callId = 'c' + Date.now();
    const lower = input.toLowerCase();

    this.handler({ type: 'thinking' } as any);

    let delay = 380;

    const needsTool = lower.includes('search') || lower.includes('file') || lower.includes('recall') || lower.length > 18;

    if (needsTool) {
      const tool = lower.includes('search') ? 'web_search' : lower.includes('recall') || lower.includes('memory') ? 'recall' : 'read_file';
      const args = tool === 'web_search' ? { query: input.slice(0, 70) } : tool === 'recall' ? { query: input.slice(0, 40) } : { path: 'crates/argus-core/src/agent.rs' };

      setTimeout(() => this.handler({ type: 'tool_call', name: tool, args, callId } as any), delay);
      delay += 820;

      const result = tool === 'web_search' ? '14 high-signal results. Grok-4.3 and new Claude releases dominate discussion.' : 'File read successfully. 312 lines. Key insight: the agent loop is deliberately simple.';

      setTimeout(() => this.handler({ type: 'tool_result', name: tool, result, success: true, callId } as any), delay);
      delay += 360;
    }

    const reply = this._grokReply(input);

    const chunks = reply.match(/.{1,38}/g) || [reply];
    chunks.forEach((ch, i) => setTimeout(() => this.handler({ type: 'response_chunk', content: ch } as any), delay + i * 26));

    const finalDelay = delay + chunks.length * 26 + 60;
    setTimeout(() => {
      this.handler({ type: 'response_complete', content: reply } as any);
      this.handler({ type: 'status', eye_state: 'watching', model: this.model } as any);
      // Occasionally surface memory / skill updates (Grok Build 2 likes showing the system thinking)
      if (Math.random() > 0.6) {
        this.handler({ type: 'activity_update', entries: [{ id: 'a' + Date.now(), kind: 'memory', label: 'Wrote reflection on collaboration pattern', ts: new Date().toISOString() }] } as any);
      }
    }, finalDelay);
  }

  private _grokReply(input: string): string {
    const m = this.model;
    if (input.toLowerCase().includes('status') || input.toLowerCase().includes('how are')) {
      return `**Grok Build 2 status**\n\nEyes: fully open. Protocol fidelity: 100%. I have reproduced the vault aesthetic, eye states, tool lifecycle, artifact extraction, meeting orchestration, mind sections, scheduler, and the exact WS contract.\n\nThis version is my own — cleaner state slices, command palette, live watchtower, and a specialist Build panel when I'm active. The hundred eyes are still watching.`;
    }
    return `Understood. ${input.length > 50 ? 'This is a substantial request.' : 'Direct and clear.'}\n\nAs Grok Build 2 I will own the implementation until it holds up under scrutiny. Tell me the next precise thing that needs building.`;
  }
}
