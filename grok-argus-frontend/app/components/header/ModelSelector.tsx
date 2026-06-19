'use client';
import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronDown } from 'lucide-react';
import { useAgentStore } from '../../hooks/useAgentState';
import { MODEL_CONFIG, MODELS_IN_ORDER } from '../../lib/models';
import { TierBadge } from '../shared/TierBadge';
import { ModelId } from '../../lib/types';

export function ModelSelector() {
  const [open, setOpen] = useState(false);
  const active = useAgentStore(s => s.activeModel);
  const sw = useAgentStore(s => s.switchModel);
  const cfg = MODEL_CONFIG[active];

  return (
    <div className="relative">
      <button onClick={() => setOpen(!open)} className="flex items-center gap-1.5 px-2.5 py-1 rounded border text-[11px]" style={{ borderColor: '#32325a', background: '#16162a' }}>
        <span>{cfg.icon}</span><span style={{ color: cfg.color }}>{cfg.name}</span><TierBadge tier={cfg.tier} /><ChevronDown size={11} className="text-[#5a5a7a]" />
      </button>
      <AnimatePresence>
        {open && (
          <>
            <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
            <motion.div initial={{ opacity: 0, y: -3 }} animate={{ opacity: 1, y: 0 }} className="absolute right-0 mt-1 z-50 w-64 rounded border overflow-hidden" style={{ background: '#12121f', borderColor: '#32325a' }}>
              {MODELS_IN_ORDER.map(id => {
                const m = MODEL_CONFIG[id]; const act = id === active;
                return (
                  <div key={id} onClick={() => { sw(id); setOpen(false); }} className="flex items-center px-3 py-1.5 cursor-pointer" style={{ background: act ? 'rgba(201,168,76,0.08)' : undefined }}>
                    <span className="mr-2">{m.icon}</span><span className="font-mono text-[12px]" style={{ color: act ? '#f5b800' : '#e8e5dc' }}>{m.name}</span>
                    <TierBadge tier={m.tier} className="ml-auto" />
                  </div>
                );
              })}
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </div>
  );
}
