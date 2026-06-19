'use client';

import { useState, useEffect, useRef, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Plus, Send, RotateCcw } from 'lucide-react';
import { RealConnection } from '@/hooks/useWebSocket';
import { ServerMessage, Message, ToolCall, Artifact, ModelId, EyeState } from '@/lib/types';
import { WS_URL } from '@/lib/constants';
import { parseArtifacts } from '@/lib/artifacts';
import { UserMessage } from '@/components/conversation/UserMessage';
import { ArgusMessage } from '@/components/conversation/ArgusMessage';
import { ToolCallBlock } from '@/components/conversation/ToolCallBlock';
import { ArtifactPanel } from '@/components/artifacts/ArtifactPanel';
import { PaneModelSelector } from './PaneModelSelector';
import { StarfieldBackground } from '@/components/shared/StarfieldBackground';

const EYE: Record<EyeState, string> = {
  watching: '◉', thinking: '◎', executing: '⊙', complete: '✦',
};

interface Props {
  paneIndex: number;   // 1-based label for display
  initialModel?: ModelId;
  onClose: () => void;
  /** If set, auto-sends this message as soon as the WebSocket connects */
  openingBrief?: string;
}

export function ChatPane({ paneIndex, initialModel = 'grok-build', onClose, openingBrief }: Props) {
  const [connected, setConnected] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [streamingContent, setStreamingContent] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);
  const [eyeState, setEyeState] = useState<EyeState>('watching');
  const [model, setModel] = useState<ModelId>(initialModel);
  const [activeToolCalls, setActiveToolCalls] = useState<ToolCall[]>([]);
  const [artifactState, setArtifactState] = useState<{ artifacts: Artifact[]; index: number } | null>(null);
  const [inputValue, setInputValue] = useState('');
  const [title, setTitle] = useState('');

  const wsRef = useRef<RealConnection | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // ── Handle incoming server messages ─────────────────────────────────────
  const handleMessage = useCallback((msg: ServerMessage) => {
    switch (msg.type) {
      case 'connected':
        setConnected(true);
        break;

      case 'thinking':
        setEyeState('thinking');
        setIsStreaming(true);
        setStreamingContent('');
        break;

      case 'tool_call': {
        const callId = (msg as any).call_id ?? (msg as any).callId ?? String(Date.now());
        const tc: ToolCall = {
          id: callId, name: msg.name, args: msg.args,
          state: 'executing', startedAt: new Date(),
        };
        setEyeState('executing');
        setActiveToolCalls((prev) => [...prev, tc]);
        setMessages((prev) => [
          ...prev,
          { id: 'tc-' + callId, role: 'assistant', content: '', timestamp: new Date(), toolCalls: [tc] },
        ]);
        break;
      }

      case 'tool_result': {
        const callId = (msg as any).call_id ?? (msg as any).callId ?? '';
        const now = new Date();
        setActiveToolCalls((prev) =>
          prev.map((tc) => tc.id === callId
            ? { ...tc, result: msg.result, success: msg.success, state: 'complete', completedAt: now }
            : tc)
        );
        setMessages((prev) =>
          prev.map((m) => {
            if (!m.toolCalls?.some((tc) => tc.id === callId)) return m;
            return {
              ...m,
              toolCalls: m.toolCalls.map((tc) =>
                tc.id === callId
                  ? { ...tc, result: msg.result, success: msg.success, state: 'complete', completedAt: now }
                  : tc
              ),
            };
          })
        );
        break;
      }

      case 'response_chunk':
        setStreamingContent((prev) => prev + msg.content);
        setEyeState('thinking');
        break;

      case 'response_complete': {
        const { cleanText, artifacts } = parseArtifacts(msg.content);
        setMessages((prev) => [
          ...prev,
          {
            id: 'resp-' + Date.now(),
            role: 'assistant',
            content: cleanText,
            timestamp: new Date(),
            artifacts: artifacts.length > 0 ? artifacts : undefined,
          },
        ]);
        setStreamingContent('');
        setIsStreaming(false);
        setEyeState('complete');
        setActiveToolCalls([]);
        setTimeout(() => setEyeState('watching'), 1500);
        break;
      }

      case 'status':
        setEyeState(msg.eye_state as EyeState);
        break;

      case 'conversation_started':
        setMessages([]);
        setTitle(msg.title || '');
        break;

      case 'error':
        setMessages((prev) => [
          ...prev,
          { id: 'err-' + Date.now(), role: 'assistant', content: `**Error:** ${msg.message}`, timestamp: new Date() },
        ]);
        setIsStreaming(false);
        setEyeState('watching');
        break;
    }
  }, []);

  // ── Connect on mount ─────────────────────────────────────────────────────
  useEffect(() => {
    if (!WS_URL) return;
    const conn = new RealConnection(WS_URL, handleMessage, setConnected);
    wsRef.current = conn;
    return () => { conn.close(); wsRef.current = null; };
  }, [handleMessage]);

  // ── Auto-scroll ──────────────────────────────────────────────────────────
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingContent]);

  // ── Auto-send opening brief when connected ───────────────────────────────
  const briefSentRef = useRef(false);
  useEffect(() => {
    if (connected && openingBrief && !briefSentRef.current && messages.length === 0) {
      briefSentRef.current = true;
      setTimeout(() => {
        setMessages((prev) => [
          ...prev,
          { id: 'user-brief-' + Date.now(), role: 'user', content: openingBrief, timestamp: new Date() },
        ]);
        wsRef.current?.send({ type: 'user_message', content: openingBrief });
        setTitle(openingBrief.slice(0, 50));
      }, 800);
    }
  }, [connected, openingBrief, messages.length]);

  // ── Auto-resize textarea ─────────────────────────────────────────────────
  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 120) + 'px';
  }, [inputValue]);

  // ── Actions ──────────────────────────────────────────────────────────────
  function send() {
    const text = inputValue.trim();
    if (!text || isStreaming || !wsRef.current) return;
    setInputValue('');
    setMessages((prev) => [
      ...prev,
      { id: 'user-' + Date.now(), role: 'user', content: text, timestamp: new Date() },
    ]);
    wsRef.current.send({ type: 'user_message', content: text });
    if (!title) setTitle(text.slice(0, 50));
  }

  function switchModel(id: ModelId) {
    setModel(id);
    wsRef.current?.send({ type: 'switch_model', model: id });
  }

  function newConversation() {
    wsRef.current?.send({ type: 'new_conversation' });
    setMessages([]);
    setTitle('');
    setStreamingContent('');
    setIsStreaming(false);
    setActiveToolCalls([]);
  }

  function onKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); send(); }
  }

  const placeholder = isStreaming
    ? `${EYE[eyeState]} ${eyeState === 'thinking' ? 'Thinking...' : 'Executing...'}`
    : `◉ Pane ${paneIndex} — ${connected ? 'connected' : 'connecting...'}`;

  // ── Render ───────────────────────────────────────────────────────────────
  return (
    <div
      className="flex flex-col h-full"
      style={{ borderLeft: '1px solid #1e1e32', background: '#0a0a0f', minWidth: 0, flex: 1 }}
    >
      {/* Pane header */}
      <div
        className="flex-shrink-0 px-3 py-2 flex items-center gap-2 border-b"
        style={{ borderColor: '#1e1e32', background: '#0d0d16' }}
      >
        {/* Pane number badge */}
        <span
          className="text-[8px] font-mono px-1.5 py-px rounded flex-shrink-0"
          style={{ background: 'rgba(201,168,76,0.1)', border: '1px solid rgba(201,168,76,0.3)', color: '#c9a84c', letterSpacing: '0.1em' }}
        >
          {paneIndex}
        </span>

        {/* Connection dot */}
        <span
          className="w-1.5 h-1.5 rounded-full flex-shrink-0"
          style={{ background: connected ? '#4a7c59' : '#3a3a5a' }}
          title={connected ? 'Connected' : 'Disconnected'}
        />

        {/* Title */}
        {title && (
          <span className="text-[9px] font-mono truncate flex-1 min-w-0" style={{ color: '#3a3a5a' }} title={title}>
            {title}
          </span>
        )}
        {!title && <span className="flex-1" />}

        {/* Model selector */}
        <PaneModelSelector model={model} onSwitch={switchModel} />

        {/* New conversation */}
        <button
          onClick={newConversation}
          disabled={isStreaming}
          className="flex items-center justify-center cursor-pointer flex-shrink-0 transition-all"
          style={{
            width: 24, height: 24, borderRadius: '7px',
            background: 'rgba(255,255,255,0.04)',
            border: '1px solid #2a2a42',
            color: '#6a6a8a',
          }}
          onMouseEnter={(e) => {
            const el = e.currentTarget as HTMLButtonElement;
            el.style.color = '#ffb000';
            el.style.borderColor = 'rgba(255,176,0,0.4)';
            el.style.background = 'rgba(255,176,0,0.08)';
          }}
          onMouseLeave={(e) => {
            const el = e.currentTarget as HTMLButtonElement;
            el.style.color = '#6a6a8a';
            el.style.borderColor = '#2a2a42';
            el.style.background = 'rgba(255,255,255,0.04)';
          }}
          title="New conversation"
        >
          <RotateCcw size={10} />
        </button>

        {/* Close pane */}
        <button
          onClick={onClose}
          className="flex items-center justify-center cursor-pointer flex-shrink-0 transition-all"
          style={{
            width: 24, height: 24, borderRadius: '7px',
            background: 'rgba(255,255,255,0.04)',
            border: '1px solid #2a2a42',
            color: '#6a6a8a',
          }}
          onMouseEnter={(e) => {
            const el = e.currentTarget as HTMLButtonElement;
            el.style.color = '#c9a84c';
            el.style.borderColor = 'rgba(201,168,76,0.4)';
            el.style.background = 'rgba(201,168,76,0.07)';
          }}
          onMouseLeave={(e) => {
            const el = e.currentTarget as HTMLButtonElement;
            el.style.color = '#6a6a8a';
            el.style.borderColor = '#2a2a42';
            el.style.background = 'rgba(255,255,255,0.04)';
          }}
          title="Close pane"
        >
          <X size={11} />
        </button>
      </div>

      {/* Body — artifact or chat */}
      <div className="flex flex-1 overflow-hidden">
        <AnimatePresence mode="wait">
          {artifactState ? (
            <motion.div
              key="artifact"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="flex-1 overflow-hidden"
            >
              <ArtifactPanel
                artifacts={artifactState.artifacts}
                initialIndex={artifactState.index}
                onClose={() => setArtifactState(null)}
              />
            </motion.div>
          ) : (
            <motion.div
              key="chat"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="flex flex-col flex-1 overflow-hidden relative"
            >
              <StarfieldBackground eyeState={eyeState} motionScale={0.85} />

              <div className="flex-1 overflow-y-auto px-4 py-4 space-y-4 relative z-10">
                {messages.map((msg) => {
                  if (msg.role === 'user') return <UserMessage key={msg.id} message={msg} />;
                  return (
                    <div key={msg.id}>
                      {msg.toolCalls?.map((tc) => <ToolCallBlock key={tc.id} toolCall={tc} />)}
                      {(msg.content || msg.artifacts?.length) ? (
                        <ArgusMessage
                          message={msg}
                          onOpenArtifact={(artifacts, index) => setArtifactState({ artifacts, index })}
                        />
                      ) : null}
                    </div>
                  );
                })}

                {isStreaming && streamingContent && (
                  <div className="animate-fade-in">
                    <div className="argus-prose text-argus-text text-sm leading-relaxed whitespace-pre-wrap">
                      {streamingContent}
                      <span className="inline-block w-1.5 h-4 bg-argus-amber ml-0.5 animate-pulse-rapid align-bottom" />
                    </div>
                  </div>
                )}

                {/* Empty state */}
                {messages.length === 0 && !isStreaming && (
                  <div className="flex flex-col items-center justify-center h-full py-16 text-center">
                    <span className="text-3xl mb-3" style={{ opacity: 0.3 }}>◎</span>
                    <p className="text-[11px] font-mono" style={{ color: '#3a3a5a' }}>Pane {paneIndex} ready</p>
                    <p className="text-[9px] font-mono mt-1" style={{ color: '#2a2a3a' }}>
                      {connected ? 'Connected · ask anything' : 'Connecting to Argus...'}
                    </p>
                  </div>
                )}

                <div ref={bottomRef} />
              </div>

              {/* Input */}
              <PaneInput
                value={inputValue}
                onChange={setInputValue}
                onSubmit={send}
                onKeyDown={onKeyDown}
                disabled={isStreaming}
                placeholder={placeholder}
                textareaRef={textareaRef}
              />
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}

// ── Elegant pane input ────────────────────────────────────────────────────────
interface PaneInputProps {
  value: string;
  onChange: (v: string) => void;
  onSubmit: () => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  disabled: boolean;
  placeholder: string;
  textareaRef: React.RefObject<HTMLTextAreaElement | null>;
}

function PaneInput({ value, onChange, onSubmit, onKeyDown, disabled, placeholder, textareaRef }: PaneInputProps) {
  const [focused, setFocused] = useState(false);
  const canSend = !!value.trim() && !disabled;

  return (
    <div
      className="flex-shrink-0 px-3 py-3"
      style={{ borderTop: '1px solid #1e1e32', background: '#0d0d14' }}
    >
      <div
        style={{
          borderRadius: '12px',
          border: disabled
            ? '1px solid rgba(201,168,76,0.3)'
            : focused
            ? '1px solid rgba(201,168,76,0.45)'
            : '1px solid #2a2a42',
          background: focused ? 'rgba(201,168,76,0.025)' : '#0a0a12',
          boxShadow: disabled
            ? '0 0 0 2px rgba(201,168,76,0.07)'
            : focused
            ? '0 0 0 2px rgba(201,168,76,0.05)'
            : 'none',
          transition: 'border-color 0.18s, box-shadow 0.18s, background 0.18s',
          padding: '8px 10px 8px 13px',
          display: 'flex',
          alignItems: 'flex-end',
          gap: '8px',
        }}
      >
        <textarea
          ref={textareaRef}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={onKeyDown}
          onFocus={() => setFocused(true)}
          onBlur={() => setFocused(false)}
          disabled={disabled}
          placeholder={placeholder}
          rows={1}
          className="flex-1 bg-transparent resize-none outline-none leading-relaxed"
          style={{
            fontFamily: "'Instrument Sans', sans-serif",
            fontSize: '13px',
            color: '#e2e2ea',
            maxHeight: '120px',
            paddingBottom: '1px',
          }}
        />
        <button
          onClick={onSubmit}
          disabled={!canSend}
          className="flex-shrink-0 flex items-center justify-center cursor-pointer transition-all"
          style={{
            width: 28,
            height: 28,
            borderRadius: '9px',
            background: canSend ? 'rgba(201,168,76,0.2)' : 'rgba(255,255,255,0.03)',
            border: canSend ? '1px solid rgba(201,168,76,0.45)' : '1px solid #2a2a42',
            color: canSend ? '#c9a84c' : '#3a3a5a',
            cursor: canSend ? 'pointer' : 'not-allowed',
          }}
          onMouseEnter={(e) => {
            if (!canSend) return;
            (e.currentTarget as HTMLButtonElement).style.background = 'rgba(201,168,76,0.32)';
          }}
          onMouseLeave={(e) => {
            if (!canSend) return;
            (e.currentTarget as HTMLButtonElement).style.background = 'rgba(201,168,76,0.2)';
          }}
        >
          <Send size={11} />
        </button>
      </div>
    </div>
  );
}
