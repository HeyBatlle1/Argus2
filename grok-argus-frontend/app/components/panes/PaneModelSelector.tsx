'use client';
import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronDown } from 'lucide-react';
import { MODEL_CONFIG, MODELS_IN_ORDER } from '../../lib/models';
import { ModelId } from '../../lib/types';

export function PaneModelSelector({ model, onSwitch }: { model: ModelId; onSwitch: (m: ModelId) => void }) {
  const [o, setO] = useState(false);
  const m = MODEL_CONFIG[model];
  return (
    <div className="relative">
      <button onClick={() => setO(!o)} className="flex items-center gap-1 px-1.5 py-0.5 text-[10px] rounded" style={{ border: '1px solid #1e1e32', color: m.color }}>
        {m.icon} {m.name} <ChevronDown size={9} />
      </button>
      <AnimatePresence>
        {o && <>
          <div className="fixed inset-0 z-40" onClick={() => setO(false)} />
          <motion.div initial={{ opacity: 0, y: -2 }} animate={{ opacity: 1, y: 0 }} className="absolute z-50 mt-1 w-44 rounded overflow-hidden text-[10px] font-mono" style={{ background: '#0d0d18', border: '1px solid #1e1e32' }}>
            {MODELS_IN_ORDER.map(id => <button key={id} onClick={() => { onSwitch(id); setO(false); }} className="w-full text-left px-3 py-1.5 flex gap-1.5" style={{ background: id === model ? 'rgba(201,168,76,0.08)' : undefined }}>{MODEL_CONFIG[id].icon} {MODEL_CONFIG[id].name}</button>)}
          </motion.div>
        </>}
      </AnimatePresence>
    </div>
  );
}
