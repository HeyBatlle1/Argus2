'use client';

import { useState, useRef, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Send } from 'lucide-react';
import { useAgentStore } from '@/hooks/useAgentState';
import { EYE_SYMBOLS } from '@/lib/constants';
import { isBuilderModel, BUILDER_THEME } from '@/lib/builder';
import { BuilderQuickBar } from '@/components/builder/BuilderQuickBar';

export function InputArea() {
  const [value, setValue] = useState('');
  const [focused, setFocused] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const sendMessage = useAgentStore((s) => s.sendMessage);
  const isStreaming = useAgentStore((s) => s.isStreaming);
  const eyeState = useAgentStore((s) => s.eyeState);
  const activeModel = useAgentStore((s) => s.activeModel);
  const initConnection = useAgentStore((s) => s.initConnection);

  const builderMode = isBuilderModel(activeModel);
  const accent = builderMode ? BUILDER_THEME.accent : '#c9a84c';

  useEffect(() => { initConnection(); }, [initConnection]);

  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 140) + 'px';
  }, [value]);

  function submit() {
    const text = value.trim();
    if (!text || isStreaming) return;
    setValue('');
    sendMessage(text);
  }

  function onKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submit();
    }
  }

  const placeholder = isStreaming
    ? `${EYE_SYMBOLS[eyeState]} ${eyeState === 'thinking' ? 'Thinking...' : eyeState === 'executing' ? 'Executing...' : 'Processing...'}`
    : builderMode
    ? '⚡  Grok Build — describe what to ship...'
    : '◉  Ask Argus anything...';

  const canSend = !!value.trim() && !isStreaming;

  return (
    <div
      className="flex-shrink-0 px-4 py-4 chat-input-footer"
      style={{
        background: 'rgba(10, 10, 18, 0.78)',
        backdropFilter: 'blur(12px)',
        borderTop: builderMode ? `1px solid ${BUILDER_THEME.border}` : '1px solid rgba(30, 30, 50, 0.6)',
      }}
    >
      {builderMode && <BuilderQuickBar />}

      <div
        style={{
          borderRadius: '14px',
          border: isStreaming
            ? `1px solid ${accent}55`
            : focused
            ? `1px solid ${accent}88`
            : '1px solid #2a2a42',
          background: focused ? `${accent}08` : '#0a0a12',
          boxShadow: isStreaming
            ? `0 0 0 3px ${accent}14, inset 0 1px 0 rgba(255,255,255,0.03)`
            : focused
            ? `0 0 0 3px ${accent}10, inset 0 1px 0 rgba(255,255,255,0.03)`
            : 'inset 0 1px 0 rgba(255,255,255,0.02)',
          transition: 'border-color 0.2s, box-shadow 0.2s, background 0.2s',
          padding: '10px 12px 10px 16px',
          display: 'flex',
          alignItems: 'flex-end',
          gap: '10px',
        }}
      >
        <textarea
          ref={textareaRef}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={onKeyDown}
          onFocus={() => setFocused(true)}
          onBlur={() => setFocused(false)}
          disabled={isStreaming}
          placeholder={placeholder}
          rows={1}
          className="flex-1 bg-transparent resize-none outline-none leading-relaxed"
          style={{
            fontFamily: builderMode ? "'JetBrains Mono', monospace" : "'Instrument Sans', sans-serif",
            fontSize: '14px',
            color: '#e8e8f0',
            maxHeight: '140px',
            paddingBottom: '2px',
          }}
        />

        <motion.button
          onClick={submit}
          disabled={!canSend}
          className="flex-shrink-0 flex items-center justify-center cursor-pointer"
          style={{
            width: 32,
            height: 32,
            borderRadius: '10px',
            background: canSend ? `${accent}33` : 'rgba(255,255,255,0.04)',
            border: canSend ? `1px solid ${accent}88` : '1px solid #2a2a42',
            color: canSend ? accent : '#3a3a5a',
            cursor: canSend ? 'pointer' : 'not-allowed',
            transition: 'all 0.18s',
          }}
          animate={
            isStreaming
              ? { boxShadow: [`0 0 4px ${accent}33`, `0 0 12px ${accent}66`, `0 0 4px ${accent}33`] }
              : {}
          }
          transition={isStreaming ? { duration: 1.4, repeat: Infinity } : {}}
          whileHover={canSend ? { scale: 1.06 } : {}}
          whileTap={canSend ? { scale: 0.93 } : {}}
        >
          <Send size={13} />
        </motion.button>
      </div>

      <div className="flex justify-between items-center mt-2 px-1">
        <span className="text-[9px] font-mono" style={{ color: '#2a2a42' }}>
          {builderMode ? '⌘B summon builder · ' : ''}Enter send · Shift+Enter newline
        </span>
        <span className="text-[9px] font-mono" style={{ color: value.length > 200 ? accent : '#2a2a42' }}>
          {value.length > 0 ? value.length : ''}
        </span>
      </div>
    </div>
  );
}