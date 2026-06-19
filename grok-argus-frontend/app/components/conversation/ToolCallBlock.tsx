'use client';
import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronRight } from 'lucide-react';
import { ToolCall } from '../../lib/types';
import { EYE_SYMBOLS } from '../../lib/constants';

const B = { pending: '#5a5a8a', executing: '#4a8fc4', complete: '#4a7c59', error: '#8b1a1a' };

export function ToolCallBlock({ toolCall }: { toolCall: ToolCall }) {
  const [exp, setExp] = useState(false);
  const bc = B[toolCall.state];
  const sym = EYE_SYMBOLS[toolCall.state === 'executing' ? 'executing' : toolCall.state === 'complete' ? 'complete' : 'watching'];
  return (
    <motion.div initial={{ opacity: 0, x: -6 }} animate={{ opacity: 1, x: 0 }} className="my-1.5 rounded overflow-hidden" style={{ borderLeft: `2px solid ${bc}`, background: 'linear-gradient(135deg,#12121f,#16162a)' }}>
      <div className="flex items-center gap-2 px-3 py-1.5 text-[10px] font-mono">
        <motion.span style={{ color: bc }} animate={toolCall.state === 'executing' ? { opacity: [1, .3, 1] } : {}} transition={{ duration: .75, repeat: Infinity }}>{sym}</motion.span>
        <span style={{ color: bc, letterSpacing: '.08em' }}>{toolCall.state.toUpperCase()}</span>
        <span className="text-[#e8e5dc] ml-1">{toolCall.name}</span>
      </div>
      {Object.keys(toolCall.args || {}).length > 0 && <div className="px-3 pb-1.5 text-[10px] font-mono text-[#b8b5ac]">{Object.entries(toolCall.args).map(([k, v]) => <div key={k}>{k}: <span className="text-[#e8e5dc]">{JSON.stringify(v)}</span></div>)}</div>}
      {toolCall.result && (
        <div className="border-t px-3 py-1.5 text-[10px]" style={{ borderColor: 'rgba(255,255,255,0.08)' }}>
          <div className="flex justify-between">
            <span style={{ color: toolCall.success ? '#39d353' : '#ff4444' }}>{toolCall.success ? '✦' : '✕'} {toolCall.result.slice(0, 58)}{toolCall.result.length > 58 ? '…' : ''}</span>
            {toolCall.result.length > 58 && <button onClick={() => setExp(!exp)} className="text-[#f5b800] flex items-center gap-px text-[9px]"><ChevronRight size={10} className={exp ? 'rotate-90' : ''} />{exp ? 'less' : 'more'}</button>}
          </div>
          <AnimatePresence>{exp && <motion.div initial={{ height: 0 }} animate={{ height: 'auto' }} className="mt-1 p-1.5 bg-[#16162a] rounded text-[#b8b5ac] max-h-32 overflow-auto font-mono text-[10px]">{toolCall.result}</motion.div>}</AnimatePresence>
        </div>
      )}
    </motion.div>
  );
}
