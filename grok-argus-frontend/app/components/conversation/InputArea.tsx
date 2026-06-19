'use client';
import { useState, useRef, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Send } from 'lucide-react';
import { useAgentStore } from '../../hooks/useAgentState';
import { EYE_SYMBOLS } from '../../lib/constants';

export function InputArea() {
  const [val, setVal] = useState('');
  const [focused, setFocused] = useState(false);
  const ref = useRef<HTMLTextAreaElement>(null);
  const send = useAgentStore(s => s.sendMessage);
  const streaming = useAgentStore(s => s.isStreaming);
  const eye = useAgentStore(s => s.eyeState);

  useEffect(() => {
    const el = ref.current; if (!el) return;
    el.style.height = 'auto'; el.style.height = Math.min(el.scrollHeight, 132) + 'px';
  }, [val]);

  function submit() {
    const t = val.trim(); if (!t || streaming) return;
    setVal(''); send(t);
  }
  function kd(e: React.KeyboardEvent) { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); submit(); } }

  const ph = streaming ? `${EYE_SYMBOLS[eye]} ${eye === 'thinking' ? 'Thinking…' : eye === 'executing' ? 'Executing…' : 'Working…'}` : '◉  Ask anything — Grok Build 2 is listening.';

  return (
    <div className="flex-shrink-0 px-4 py-4" style={{ background: '#0d0d14', borderTop: '1px solid #1e1e32' }}>
      <div style={{ borderRadius: 14, border: streaming ? '1px solid rgba(201,168,76,0.35)' : focused ? '1px solid rgba(201,168,76,0.55)' : '1px solid #2a2a42', background: focused ? 'rgba(201,168,76,0.025)' : '#0a0a12', padding: '10px 13px 9px 16px', display: 'flex', alignItems: 'flex-end', gap: 10 }}>
        <textarea ref={ref} value={val} onChange={e => setVal(e.target.value)} onKeyDown={kd} onFocus={() => setFocused(true)} onBlur={() => setFocused(false)} disabled={streaming} placeholder={ph} rows={1} className="flex-1 bg-transparent resize-none outline-none text-[14px] leading-relaxed" style={{ color: '#e8e8f0', maxHeight: 132 }} />
        <motion.button onClick={submit} disabled={!val.trim() || streaming} whileTap={{ scale: 0.92 }} className="w-8 h-8 rounded-[10px] flex items-center justify-center flex-shrink-0" style={{ background: val.trim() && !streaming ? 'rgba(201,168,76,0.22)' : 'rgba(255,255,255,0.04)', border: val.trim() && !streaming ? '1px solid rgba(201,168,76,0.5)' : '1px solid #2a2a42', color: val.trim() && !streaming ? '#c9a84c' : '#3a3a5a' }}>
          <Send size={13} />
        </motion.button>
      </div>
      <div className="flex justify-between px-1 mt-1.5 text-[9px] font-mono" style={{ color: '#2a2a42' }}>
        <span>Enter • Shift+Enter newline</span>
        <span style={{ color: val.length > 180 ? '#f5b800' : '#2a2a42' }}>{val.length || ''}</span>
      </div>
    </div>
  );
}
