'use client';
import { useState, useRef, useEffect } from 'react';
import { X, Send, RotateCcw } from 'lucide-react';
import { RealConnection } from '../../hooks/useWebSocket';
import { ServerMessage, Message, ToolCall, Artifact, ModelId, EyeState } from '../../lib/types';
import { WS_URL } from '../../lib/constants';
import { parseArtifacts } from '../../lib/artifacts';
import { UserMessage } from '../conversation/UserMessage';
import { ArgusMessage } from '../conversation/ArgusMessage';
import { ToolCallBlock } from '../conversation/ToolCallBlock';
import { PaneModelSelector } from './PaneModelSelector';
import { ArtifactPanel } from '../artifacts/ArtifactPanel';

const E = { watching: '◉', thinking: '◎', executing: '⊙', complete: '✦' };

export function ChatPane({ paneIndex, initialModel = 'grok-build', onClose, openingBrief }: { paneIndex: number; initialModel?: ModelId; onClose: () => void; openingBrief?: string }) {
  const [connected, setConnected] = useState(false);
  const [msgs, setMsgs] = useState<Message[]>([]);
  const [stream, setStream] = useState('');
  const [isStream, setIsStream] = useState(false);
  const [eye, setEye] = useState<EyeState>('watching');
  const [model, setModel] = useState<ModelId>(initialModel);
  const [activeTC, setActiveTC] = useState<ToolCall[]>([]);
  const [artifact, setArtifact] = useState<any>(null);
  const [input, setInput] = useState('');
  const [title, setTitle] = useState('');

  const wsRef = useRef<RealConnection | null>(null);
  const bottom = useRef<HTMLDivElement>(null);
  const ta = useRef<HTMLTextAreaElement>(null);

  const handle = (msg: ServerMessage) => {
    switch (msg.type) {
      case 'connected': setConnected(true); break;
      case 'thinking': setEye('thinking'); setIsStream(true); setStream(''); break;
      case 'tool_call': {
        const id = (msg as any).callId || (msg as any).call_id || Date.now().toString();
        const tc = { id, name: msg.name, args: msg.args, state: 'executing' as const, startedAt: new Date() };
        setEye('executing');
        setActiveTC(p => [...p, tc]);
        setMsgs(p => [...p, { id: 'tc' + id, role: 'assistant', content: '', timestamp: new Date(), toolCalls: [tc] }]);
        break;
      }
      case 'tool_result': {
        const id = (msg as any).callId || (msg as any).call_id || '';
        const now = new Date();
        setActiveTC(p => p.map(tc => tc.id === id ? { ...tc, result: msg.result, success: msg.success, state: 'complete', completedAt: now } : tc));
        setMsgs(p => p.map(m => !m.toolCalls?.some(tc => tc.id === id) ? m : { ...m, toolCalls: m.toolCalls!.map(tc => tc.id === id ? { ...tc, result: msg.result, success: msg.success, state: 'complete', completedAt: now } : tc) }));
        break;
      }
      case 'response_chunk': setStream(s => s + msg.content); setEye('thinking'); break;
      case 'response_complete': {
        const { cleanText, artifacts } = parseArtifacts(msg.content);
        setMsgs(p => [...p, { id: 'r' + Date.now(), role: 'assistant', content: cleanText, timestamp: new Date(), artifacts: artifacts.length ? artifacts : undefined }]);
        setStream(''); setIsStream(false); setEye('complete'); setActiveTC([]);
        setTimeout(() => setEye('watching'), 1300);
        break;
      }
      case 'status': setEye(msg.eye_state as EyeState); break;
      case 'error': setMsgs(p => [...p, { id: 'e' + Date.now(), role: 'assistant', content: `**Error:** ${msg.message}`, timestamp: new Date() }]); setIsStream(false); setEye('watching'); break;
    }
  };

  useEffect(() => {
    if (!WS_URL) return;
    const c = new RealConnection(WS_URL, handle, setConnected);
    wsRef.current = c;
    return () => { c.close(); wsRef.current = null; };
  }, []);

  useEffect(() => { bottom.current?.scrollIntoView({ behavior: 'smooth' }); }, [msgs, stream]);

  // Auto brief
  const briefRef = useRef(false);
  useEffect(() => {
    if (connected && openingBrief && !briefRef.current && msgs.length === 0) {
      briefRef.current = true;
      setTimeout(() => {
        setMsgs(p => [...p, { id: 'ub' + Date.now(), role: 'user', content: openingBrief, timestamp: new Date() }]);
        wsRef.current?.send({ type: 'user_message', content: openingBrief });
        setTitle(openingBrief.slice(0, 48));
      }, 700);
    }
  }, [connected, openingBrief, msgs.length]);

  function send() {
    const t = input.trim(); if (!t || isStream || !wsRef.current) return;
    setInput('');
    setMsgs(p => [...p, { id: 'u' + Date.now(), role: 'user', content: t, timestamp: new Date() }]);
    wsRef.current.send({ type: 'user_message', content: t });
    if (!title) setTitle(t.slice(0, 46));
  }
  function kd(e: React.KeyboardEvent) { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); send(); } }
  function swM(id: ModelId) { setModel(id); wsRef.current?.send({ type: 'switch_model', model: id }); }

  const ph = isStream ? `${E[eye]} ${eye === 'thinking' ? 'Thinking…' : 'Working…'}` : `◉ Pane ${paneIndex}`;

  return (
    <div className="flex flex-col h-full" style={{ borderLeft: '1px solid #1e1e32', background: '#0a0a0f', flex: 1, minWidth: 0 }}>
      <div className="px-3 py-1.5 flex items-center gap-2 border-b flex-shrink-0 text-[10px]" style={{ background: '#0d0d16', borderColor: '#1e1e32' }}>
        <span className="px-1.5 py-px rounded font-mono" style={{ background: 'rgba(201,168,76,0.1)', border: '1px solid rgba(201,168,76,0.3)', color: '#c9a84c' }}>{paneIndex}</span>
        <span className="w-1.5 h-1.5 rounded-full" style={{ background: connected ? '#4a7c59' : '#3a3a5a' }} />
        {title && <span className="text-[#3a3a5a] font-mono truncate flex-1">{title}</span>}
        <PaneModelSelector model={model} onSwitch={swM} />
        <button onClick={() => { wsRef.current?.send({ type: 'new_conversation' }); setMsgs([]); setTitle(''); }} className="w-6 h-6 flex items-center justify-center rounded" style={{ background: 'rgba(255,255,255,0.04)', border: '1px solid #2a2a42' }}><RotateCcw size={10} /></button>
        <button onClick={onClose} className="w-6 h-6 flex items-center justify-center rounded" style={{ background: 'rgba(255,255,255,0.04)', border: '1px solid #2a2a42' }}><X size={11} /></button>
      </div>

      <div className="flex-1 overflow-hidden flex flex-col">
        <div className="flex-1 overflow-y-auto px-4 py-4 space-y-3">
          {msgs.map(m => m.role === 'user' ? <UserMessage key={m.id} message={m} /> : <div key={m.id}>{m.toolCalls?.map(tc => <ToolCallBlock key={tc.id} toolCall={tc} />)}{(m.content || m.artifacts) && <ArgusMessage message={m} onOpenArtifact={(aa, ii) => setArtifact({ artifacts: aa, index: ii })} />}</div>)}
          {isStream && stream && <div className="argus-prose text-sm whitespace-pre-wrap">{stream}<span className="inline-block w-1 h-3 bg-[#f5b800] ml-0.5 animate-pulse align-bottom" /></div>}
          {msgs.length === 0 && !isStream && <div className="h-full flex items-center justify-center text-center text-[#3a3a5a] text-[11px] font-mono">Pane {paneIndex} — {connected ? 'ready' : 'connecting…'}</div>}
          <div ref={bottom} />
        </div>
        <div className="px-3 py-2.5 border-t flex-shrink-0" style={{ background: '#0d0d14', borderColor: '#1e1e32' }}>
          <div className="flex gap-2 items-end rounded-xl border px-3 py-1.5" style={{ borderColor: isStream ? 'rgba(201,168,76,0.35)' : '#2a2a42', background: '#0a0a12' }}>
            <textarea ref={ta} value={input} onChange={e => setInput(e.target.value)} onKeyDown={kd} disabled={isStream} placeholder={ph} rows={1} className="flex-1 bg-transparent resize-none text-sm outline-none" style={{ maxHeight: 96 }} />
            <button onClick={send} disabled={!input.trim() || isStream} className="w-7 h-7 rounded-lg flex items-center justify-center" style={{ background: input.trim() && !isStream ? 'rgba(201,168,76,0.2)' : 'rgba(255,255,255,0.04)', border: '1px solid #2a2a42', color: input.trim() && !isStream ? '#c9a84c' : '#3a3a5a' }}><Send size={12} /></button>
          </div>
        </div>
      </div>

      {artifact && <div className="absolute inset-0 z-40 bg-black/70" onClick={() => setArtifact(null)}><div className="absolute right-0 top-0 bottom-0 w-[62%]" onClick={e => e.stopPropagation()}><div onClick={() => setArtifact(null)} className="absolute top-2 right-2 z-50 cursor-pointer text-[#b8b5ac]"><X /></div><div style={{ height: '100%' }}><ArtifactPanel artifacts={artifact.artifacts} initialIndex={artifact.index} onClose={() => setArtifact(null)} /></div></div></div>}
    </div>
  );
}
