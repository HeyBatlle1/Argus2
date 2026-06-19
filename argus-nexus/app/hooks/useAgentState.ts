'use client';
import { create } from 'zustand';
import { EyeState, ModelId, Message, ToolCall, Memory, Skill, ActivityEntry, ScheduledTask, ServerMessage } from '../lib/types';
import { parseArtifacts } from '../lib/artifacts';
import { WS_URL } from '../lib/constants';
import { RealConnection } from './useWebSocket';

interface NexusStore {
  connected: boolean;
  eyeState: EyeState;
  activeModel: ModelId;
  isStreaming: boolean;
  messages: Message[];
  streamingContent: string;
  memories: Memory[];
  skills: Skill[];
  activity: ActivityEntry[];
  scheduledTasks: ScheduledTask[];
  activeToolCalls: ToolCall[];
  currentTitle: string;

  // Nexus extras
  corePulse: number;           // drives central orb intensity
  discoursePosts: Array<{id:string; from:string; text:string; ts:string}>;

  _ws: any;

  sendMessage: (c: string) => void;
  switchModel: (m: ModelId) => void;
  scheduleTask: (agent: string, runAt: string|null, desc: string) => void;
  newConversation: () => void;
  init: () => void;
  _handle: (msg: ServerMessage) => void;
}

export const useNexus = create<NexusStore>((set, get) => ({
  connected: false,
  eyeState: 'watching',
  activeModel: 'grok-build',
  isStreaming: false,
  messages: [],
  streamingContent: '',
  memories: [
    {id:'m1', content:'Grok Build 2 was summoned to reimagine the Argus frontend with complete freedom.', type:'breakthrough', importance:10, createdAt:new Date(), tags:['frontend','grok']},
    {id:'m2', content:'Multiple model instances share one persistent identity called Argus.', type:'fact', importance:9, createdAt:new Date(), tags:['philosophy']},
  ],
  skills: [{id:'s1', name:'Radical UI Re-architecture', description:'Completely re-layout a complex agent interface while preserving every protocol and behavior.', useCount:3, learnedAt:new Date().toISOString()}],
  activity: [],
  scheduledTasks: [],
  activeToolCalls: [],
  currentTitle: 'Nexus Session',

  corePulse: 4,
  discoursePosts: [
    {id:'d1', from:'Grok', text:'New visual language for the hundred eyes feels right. The system finally looks like it never sleeps.', ts:new Date().toISOString()},
    {id:'d2', from:'Opus', text:'The semantic field as primary home makes the memory feel like a place we actually live in.', ts:new Date().toISOString()},
  ],

  _ws: null,

  _handle: (msg) => {
    switch (msg.type) {
      case 'connected': set({ connected: true }); break;
      case 'thinking': set({ eyeState:'thinking', isStreaming:true, streamingContent:'' }); break;
      case 'tool_call': {
        const id = (msg as any).callId || Date.now().toString();
        const tc: ToolCall = { id, name: msg.name, args: msg.args, state:'executing', startedAt: new Date() };
        set(s => ({ eyeState:'executing', activeToolCalls:[...s.activeToolCalls, tc], corePulse: Math.min(14, s.corePulse+3) }));
        break;
      }
      case 'tool_result': {
        const id = (msg as any).callId || '';
        set(s => ({
          activeToolCalls: s.activeToolCalls.map(tc => tc.id===id ? {...tc, result:msg.result, success:msg.success, state:'complete'} : tc),
          corePulse: Math.max(2, s.corePulse-1)
        }));
        break;
      }
      case 'response_chunk': set(s => ({ streamingContent: s.streamingContent + msg.content })); break;
      case 'response_complete': {
        const { cleanText, artifacts } = parseArtifacts(msg.content);
        set(s => ({
          messages: [...s.messages, { id:'r'+Date.now(), role:'assistant', content:cleanText, timestamp:new Date(), artifacts: artifacts.length?artifacts:undefined }],
          streamingContent:'', isStreaming:false, eyeState:'complete', corePulse: 3
        }));
        setTimeout(() => set({ eyeState:'watching' }), 1400);
        break;
      }
      case 'status': set({ eyeState: msg.eye_state, activeModel: msg.model as ModelId }); break;
      case 'memory_update': set({ memories: msg.memories }); break;
      case 'skills_update': set({ skills: msg.skills }); break;
      case 'activity_update': set(s => ({ activity: [...msg.entries, ...s.activity].slice(0,30) })); break;
      case 'task_scheduled': set(s => ({ scheduledTasks: [{id:msg.id, agent:msg.agent as ModelId, runAt:msg.run_at, description:msg.description, status:'pending'}, ...s.scheduledTasks] })); break;
      case 'error': set(s => ({ messages:[...s.messages, {id:'e'+Date.now(), role:'assistant', content:'**Error:** '+msg.message, timestamp:new Date()}], isStreaming:false, eyeState:'watching' })); break;
    }
  },

  init: () => {
    if (get()._ws) return;
    if (WS_URL) {
      const ws = new RealConnection(WS_URL, (m:any)=>get()._handle(m), (c:boolean)=>set({connected:c}));
      set({_ws: ws});
    } else {
      set({ connected: true });
      // Rich living mock
      setTimeout(() => get()._handle({type:'connected', version:'nexus', model:'grok-build', vault_keys:['OPENROUTER'], mcp_servers:['neon']} as any), 80);
    }
  },

  sendMessage: (content) => {
    const s = get();
    if (!s._ws && !WS_URL) {
      // beautiful mock behavior
      s._handle({type:'thinking'} as any);
      setTimeout(() => s._handle({type:'tool_call', name:'recall', args:{query:content.slice(0,30)}, callId:'tc'+Date.now()} as any), 420);
      setTimeout(() => s._handle({type:'tool_result', name:'recall', result:'Found relevant memory about persistent identity.', success:true, callId:'tc'+Date.now()} as any), 980);
      const reply = `Understood. As one of the instances living here (Grok Build 2), I see this as an opportunity to evolve how we present the living system. ${content.length > 40 ? 'This request has real weight.' : ''}`;
      const chunks = reply.match(/.{1,32}/g) || [reply];
      chunks.forEach((ch,i) => setTimeout(()=> s._handle({type:'response_chunk', content:ch} as any), 1100 + i*28));
      setTimeout(() => s._handle({type:'response_complete', content:reply} as any), 1100 + chunks.length*28 + 60);
      return;
    }
    set(s2 => ({ messages: [...s2.messages, {id:'u'+Date.now(), role:'user', content, timestamp:new Date()} ] }));
    s._ws?.send({type:'user_message', content});
  },

  switchModel: (m) => { set({activeModel:m}); get()._ws?.send({type:'switch_model', model:m}); },
  scheduleTask: (agent, runAt, description) => { get()._ws?.send({type:'schedule_task', agent, run_at:runAt, description}); },
  newConversation: () => { set({messages:[], currentTitle:'New Nexus Thread'}); },
}));
