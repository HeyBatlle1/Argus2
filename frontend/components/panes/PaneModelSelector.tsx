'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronDown } from 'lucide-react';
import { MODEL_CONFIG, MODELS_IN_ORDER } from '@/lib/models';
import { ModelId } from '@/lib/types';

interface Props {
  model: ModelId;
  onSwitch: (model: ModelId) => void;
}

export function PaneModelSelector({ model, onSwitch }: Props) {
  const [open, setOpen] = useState(false);
  const cfg = MODEL_CONFIG[model];

  function select(id: ModelId) {
    onSwitch(id);
    setOpen(false);
  }

  return (
    <div className="relative">
      <button
        onClick={() => setOpen((o) => !o)}
        className="flex items-center gap-1.5 px-2 py-1 rounded transition-colors"
        style={{
          border: '1px solid #1e1e32',
          background: 'transparent',
          color: '#c8c8d8',
        }}
        onMouseEnter={(e) => (e.currentTarget.style.borderColor = 'rgba(201,168,76,0.4)')}
        onMouseLeave={(e) => (e.currentTarget.style.borderColor = '#1e1e32')}
      >
        <span className="text-xs">{cfg.icon}</span>
        <span className="text-[10px] font-mono tracking-wide" style={{ color: cfg.color }}>{cfg.name}</span>
        <motion.span animate={{ rotate: open ? 180 : 0 }} transition={{ duration: 0.15 }} style={{ color: '#5a5a7a' }}>
          <ChevronDown size={10} />
        </motion.span>
      </button>

      <AnimatePresence>
        {open && (
          <>
            <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
            <motion.div
              initial={{ opacity: 0, y: -4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -4 }}
              transition={{ duration: 0.12 }}
              className="absolute left-0 top-full mt-1 z-50 rounded overflow-hidden"
              style={{ background: '#0d0d18', border: '1px solid #1e1e32', width: 180, boxShadow: '0 8px 24px rgba(0,0,0,0.6)' }}
            >
              {MODELS_IN_ORDER.map((id) => {
                const m = MODEL_CONFIG[id];
                const active = id === model;
                return (
                  <button
                    key={id}
                    onClick={() => select(id)}
                    className="w-full flex items-center gap-2 px-3 py-2 text-left transition-colors"
                    style={{
                      background: active ? 'rgba(201,168,76,0.08)' : 'transparent',
                      borderLeft: active ? '2px solid #c9a84c' : '2px solid transparent',
                    }}
                    onMouseEnter={(e) => { if (!active) (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,255,255,0.04)'; }}
                    onMouseLeave={(e) => { if (!active) (e.currentTarget as HTMLButtonElement).style.background = 'transparent'; }}
                  >
                    <span className="text-xs">{m.icon}</span>
                    <span className="text-[11px] font-mono" style={{ color: active ? '#c9a84c' : '#8a8a9a' }}>{m.name}</span>
                  </button>
                );
              })}
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </div>
  );
}
