'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronRight } from 'lucide-react';
import { ToolCall } from '@/lib/types';
import { EYE_SYMBOLS, EYE_LABELS } from '@/lib/constants';

interface Props {
  toolCall: ToolCall;
}

const STATE_EYE: Record<string, 'thinking' | 'executing' | 'complete' | 'watching'> = {
  pending:   'thinking',
  executing: 'executing',
  complete:  'complete',
  error:     'watching',
};

const BORDER_COLOR = {
  pending:   '#1a1a2e',
  executing: '#4a8fc4',
  complete:  '#4a7c59',
  error:     '#8b1a1a',
};

const STATE_LABEL = {
  pending:   'PENDING',
  executing: 'EXECUTING',
  complete:  'COMPLETE',
  error:     'ERROR',
};

export function ToolCallBlock({ toolCall }: Props) {
  const [expanded, setExpanded] = useState(false);
  const eyeState = STATE_EYE[toolCall.state];
  const borderColor = BORDER_COLOR[toolCall.state];
  const symbol = EYE_SYMBOLS[eyeState];

  const argEntries = Object.entries(toolCall.args ?? {});

  return (
    <motion.div
      initial={{ opacity: 0, x: -8 }}
      animate={{ opacity: 1, x: 0 }}
      className="my-2 rounded overflow-hidden"
      style={{
        borderLeft: `2px solid ${borderColor}`,
        background: 'linear-gradient(135deg, #0d0d14, #111118)',
      }}
    >
      {/* Header */}
      <div className="flex items-center gap-2 px-3 py-2">
        <motion.span
          className="font-mono text-base leading-none"
          style={{ color: borderColor, display: 'inline-block' }}
          animate={
            toolCall.state === 'executing'
              ? { opacity: [1, 0.3, 1] }
              : toolCall.state === 'complete'
              ? { scale: [1.3, 1] }
              : {}
          }
          transition={
            toolCall.state === 'executing'
              ? { duration: 0.8, repeat: Infinity }
              : { duration: 0.3 }
          }
        >
          {symbol}
        </motion.span>
        <span className="text-[10px] font-mono tracking-widest uppercase" style={{ color: borderColor }}>
          {STATE_LABEL[toolCall.state]}
        </span>
        <span className="text-[11px] font-mono text-argus-text ml-1 font-medium">
          {toolCall.name}
        </span>
      </div>

      {/* Args */}
      {argEntries.length > 0 && (
        <div className="px-3 pb-2 space-y-0.5">
          {argEntries.map(([k, v]) => (
            <div key={k} className="flex gap-2">
              <span className="text-[10px] font-mono text-argus-textDim">{k}:</span>
              <span className="text-[10px] font-mono text-argus-text truncate">
                {typeof v === 'string' ? `"${v}"` : JSON.stringify(v)}
              </span>
            </div>
          ))}
        </div>
      )}

      {/* Result row */}
      {toolCall.result && (
        <div className="border-t border-argus-border/40 px-3 py-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="font-mono text-sm" style={{ color: toolCall.success ? '#4a7c59' : '#8b1a1a' }}>
                {toolCall.success ? '✦' : '✕'}
              </span>
              <span className="text-[10px] font-mono text-argus-textDim truncate max-w-[220px]">
                {toolCall.result.length > 60 ? toolCall.result.slice(0, 60) + '…' : toolCall.result}
              </span>
            </div>
            {toolCall.result.length > 60 && (
              <button
                onClick={() => setExpanded((e) => !e)}
                className="flex items-center gap-0.5 text-[10px] font-mono text-argus-amberDim hover:text-argus-amber transition-colors flex-shrink-0"
              >
                <motion.span animate={{ rotate: expanded ? 90 : 0 }} transition={{ duration: 0.15 }}>
                  <ChevronRight size={11} />
                </motion.span>
                {expanded ? 'collapse' : 'expand'}
              </button>
            )}
          </div>

          <AnimatePresence>
            {expanded && (
              <motion.div
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: 'auto', opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                transition={{ duration: 0.2 }}
                style={{ overflow: 'hidden' }}
              >
                <div
                  className="mt-2 p-2 rounded text-[11px] font-mono text-argus-textDim leading-relaxed max-h-40 overflow-y-auto"
                  style={{ background: '#0a0a0f' }}
                >
                  {toolCall.result}
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      )}
    </motion.div>
  );
}
