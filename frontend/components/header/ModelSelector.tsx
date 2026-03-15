'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronDown } from 'lucide-react';
import { useAgentStore } from '@/hooks/useAgentState';
import { MODEL_CONFIG, MODELS_IN_ORDER } from '@/lib/models';
import { TierBadge } from '@/components/shared/TierBadge';
import { ModelId } from '@/lib/types';

export function ModelSelector() {
  const [open, setOpen] = useState(false);
  const activeModel = useAgentStore((s) => s.activeModel);
  const switchModel = useAgentStore((s) => s.switchModel);
  const cfg = MODEL_CONFIG[activeModel];

  const royalModels = MODELS_IN_ORDER.filter((m) => MODEL_CONFIG[m].tier === 'royal');
  const alliedModels = MODELS_IN_ORDER.filter((m) => MODEL_CONFIG[m].tier === 'allied');

  function select(id: ModelId) {
    switchModel(id);
    setOpen(false);
  }

  return (
    <div className="relative">
      <button
        onClick={() => setOpen((o) => !o)}
        className="flex items-center gap-2 px-3 py-1.5 rounded border border-argus-border bg-argus-surface hover:border-argus-amberDim transition-colors"
      >
        <span className="text-sm">{cfg.icon}</span>
        <span className="text-[11px] font-mono text-argus-text tracking-wide">{cfg.name}</span>
        <TierBadge tier={cfg.tier} />
        <motion.span
          animate={{ rotate: open ? 180 : 0 }}
          transition={{ duration: 0.15 }}
          className="text-argus-textDim"
        >
          <ChevronDown size={12} />
        </motion.span>
      </button>

      <AnimatePresence>
        {open && (
          <>
            {/* Backdrop */}
            <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />

            <motion.div
              initial={{ opacity: 0, y: -4, scale: 0.97 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: -4, scale: 0.97 }}
              transition={{ duration: 0.15 }}
              className="absolute right-0 top-full mt-1 z-50 w-64 rounded border border-argus-border bg-argus-bg2 shadow-2xl overflow-hidden"
            >
              {/* Royal tier */}
              <div className="px-3 pt-2.5 pb-1">
                <div className="flex items-center gap-1.5 mb-1.5">
                  <span className="text-xs">👑</span>
                  <span className="text-[9px] font-mono tracking-widest uppercase text-argus-amberDim">Royal Tier</span>
                </div>
                {royalModels.map((id) => {
                  const m = MODEL_CONFIG[id];
                  return (
                    <ModelOption
                      key={id}
                      id={id}
                      active={id === activeModel}
                      onSelect={select}
                    />
                  );
                })}
              </div>

              <div className="border-t border-argus-border" />

              {/* Allied tier */}
              <div className="px-3 pt-2.5 pb-2.5">
                <div className="flex items-center gap-1.5 mb-1.5">
                  <span className="text-xs">🛡</span>
                  <span className="text-[9px] font-mono tracking-widest uppercase text-argus-greenLight">Allied Tier</span>
                </div>
                {alliedModels.map((id) => (
                  <ModelOption
                    key={id}
                    id={id}
                    active={id === activeModel}
                    onSelect={select}
                  />
                ))}
              </div>
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </div>
  );
}

function ModelOption({ id, active, onSelect }: { id: ModelId; active: boolean; onSelect: (id: ModelId) => void }) {
  const m = MODEL_CONFIG[id];
  return (
    <button
      onClick={() => onSelect(id)}
      className={`w-full flex items-center justify-between px-2 py-1.5 rounded text-left transition-colors mb-0.5 ${
        active
          ? 'bg-argus-amber/10 border border-argus-amberDim/40'
          : 'hover:bg-white/[0.04] border border-transparent'
      }`}
    >
      <div className="flex items-center gap-2">
        <span className="text-sm">{m.icon}</span>
        <span className={`text-[12px] font-mono ${active ? 'text-argus-amber' : 'text-argus-text'}`}>
          {m.name}
        </span>
      </div>
      {active && (
        <span className="text-[9px] font-mono text-argus-amber tracking-widest">ACTIVE</span>
      )}
    </button>
  );
}
