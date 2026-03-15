'use client';

import { useState, useRef, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Send } from 'lucide-react';
import { useAgentStore } from '@/hooks/useAgentState';
import { EYE_SYMBOLS } from '@/lib/constants';

export function InputArea() {
  const [value, setValue] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const sendMessage = useAgentStore((s) => s.sendMessage);
  const isStreaming = useAgentStore((s) => s.isStreaming);
  const eyeState = useAgentStore((s) => s.eyeState);
  const initConnection = useAgentStore((s) => s.initConnection);

  // Initialize connection on mount (real WS or dev mock)
  useEffect(() => { initConnection(); }, [initConnection]);

  // Auto-resize textarea
  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 120) + 'px';
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
    : '◉ Argus is watching...';

  return (
    <div
      className="flex-shrink-0 border-t border-argus-border px-4 py-3"
      style={{ background: '#0d0d14' }}
    >
      <div
        className={`flex items-end gap-3 rounded border px-3 py-2 transition-colors ${
          isStreaming
            ? 'border-argus-amberDim/30'
            : 'border-argus-border focus-within:border-argus-amberDim'
        }`}
        style={{ background: '#0a0a0f' }}
      >
        <textarea
          ref={textareaRef}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={onKeyDown}
          disabled={isStreaming}
          placeholder={placeholder}
          rows={1}
          className="flex-1 bg-transparent text-sm text-argus-text placeholder-argus-textDim/50 resize-none outline-none font-sans leading-relaxed py-0.5"
          style={{ maxHeight: '120px', fontFamily: "'Instrument Sans', sans-serif" }}
        />
        <motion.button
          onClick={submit}
          disabled={!value.trim() || isStreaming}
          className="flex-shrink-0 w-7 h-7 rounded flex items-center justify-center transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          style={{
            background: value.trim() && !isStreaming ? 'rgba(201,168,76,0.15)' : 'transparent',
            color: value.trim() && !isStreaming ? '#c9a84c' : '#8a877f',
          }}
          animate={
            isStreaming
              ? { boxShadow: ['0 0 4px rgba(201,168,76,0.2)', '0 0 10px rgba(201,168,76,0.5)', '0 0 4px rgba(201,168,76,0.2)'] }
              : {}
          }
          transition={isStreaming ? { duration: 1.2, repeat: Infinity } : {}}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
        >
          <Send size={14} />
        </motion.button>
      </div>
      <div className="flex justify-between items-center mt-1.5 px-1">
        <span className="text-[9px] font-mono text-argus-textDim/40">Enter to send · Shift+Enter for newline</span>
        <span className="text-[9px] font-mono text-argus-textDim/40">{value.length > 0 ? `${value.length}` : ''}</span>
      </div>
    </div>
  );
}
