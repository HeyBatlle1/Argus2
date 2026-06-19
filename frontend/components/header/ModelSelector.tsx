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

  const builderModel = MODELS_IN_ORDER.filter((m) => MODEL_CONFIG[m].isPrimaryCoder);
  const royalModels = MODELS_IN_ORDER.filter((m) => MODEL_CONFIG[m].tier === 'royal');
  const alliedModels = MODELS_IN_ORDER.filter(
    (m) => MODEL_CONFIG[m].tier === 'allied' && !MODEL_CONFIG[m].isPrimaryCoder,
  );

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
              className="absolute right-0 top-full mt-1 z-50 w-72 rounded border border-argus-border bg-argus-bg2 shadow-2xl overflow-hidden"
            >
              {/* Builder — primary coder */}
              <div className="px-3 pt-2.5 pb-1" style={{ background: 'rgba(74,222,128,0.04)' }}>
                <div className="flex items-center gap-1.5 mb-1.5">
                  <span className="text-xs">⚡</span>
                  <span className="text-[9px] font-mono tracking-widest uppercase" style={{ color: '#4ade80' }}>Builder Station</span>
                  <span className="text-[8px] font-mono text-argus-textDim ml-auto">12 rounds</span>
                </div>
                {builderModel.map((id) => (
                  <ModelOption key={id} id={id} active={id === activeModel} onSelect={select} showToolToggle />
                ))}
              </div>

              <div className="border-t border-argus-border" />

              {/* Royal tier */}
              <div className="px-3 pt-2.5 pb-1">
                <div className="flex items-center gap-1.5 mb-1.5">
                  <span className="text-xs">👑</span>
                  <span className="text-[9px] font-mono tracking-widest uppercase text-argus-amberDim">Royal Tier</span>
                  <span className="text-[8px] font-mono text-argus-textDim ml-auto">always on</span>
                </div>
                {royalModels.map((id) => (
                  <ModelOption
                    key={id}
                    id={id}
                    active={id === activeModel}
                    onSelect={select}
                  />
                ))}
              </div>

              <div className="border-t border-argus-border" />

              {/* Allied tier */}
              <div className="px-3 pt-2.5 pb-2.5">
                <div className="flex items-center gap-1.5 mb-1.5">
                  <span className="text-xs">🛡</span>
                  <span className="text-[9px] font-mono tracking-widest uppercase text-argus-greenLight">Allied Tier</span>
                  <span className="text-[8px] font-mono text-argus-textDim ml-auto">toggle tools</span>
                </div>
                {alliedModels.map((id) => (
                  <ModelOption
                    key={id}
                    id={id}
                    active={id === activeModel}
                    onSelect={select}
                    showToolToggle
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

function ToolToggle({ modelId }: { modelId: ModelId }) {
  const toolsEnabled = useAgentStore((s) => s.toolsEnabled);
  const setModelTools = useAgentStore((s) => s.setModelTools);
  const enabled = toolsEnabled[modelId] ?? true;

  return (
    <button
      onClick={(e) => {
        e.stopPropagation();
        setModelTools(modelId, !enabled);
      }}
      title={enabled ? 'Tools on — click to disable' : 'Tools off — click to enable'}
      className="flex items-center gap-1.5 ml-auto shrink-0"
      style={{ padding: '2px 0' }}
    >
      <span className="text-[8px] font-mono" style={{ color: enabled ? '#39d353' : '#3a3a5a' }}>
        {enabled ? 'ON' : 'OFF'}
      </span>
      {/* Track */}
      <div
        className="relative transition-colors"
        style={{
          width: 28,
          height: 14,
          borderRadius: 7,
          background: enabled ? 'rgba(57,211,83,0.25)' : 'rgba(255,255,255,0.06)',
          border: enabled ? '1px solid rgba(57,211,83,0.5)' : '1px solid #2a2a42',
          transition: 'background 0.2s, border-color 0.2s',
        }}
      >
        {/* Thumb */}
        <motion.div
          animate={{ x: enabled ? 14 : 0 }}
          transition={{ type: 'spring', stiffness: 500, damping: 30 }}
          style={{
            position: 'absolute',
            top: 2,
            left: 2,
            width: 8,
            height: 8,
            borderRadius: '50%',
            background: enabled ? '#39d353' : '#3a3a5a',
          }}
        />
      </div>
    </button>
  );
}

function ModelOption({
  id,
  active,
  onSelect,
  showToolToggle,
}: {
  id: ModelId;
  active: boolean;
  onSelect: (id: ModelId) => void;
  showToolToggle?: boolean;
}) {
  const m = MODEL_CONFIG[id];
  return (
    <div
      className={`flex items-center px-2 py-1.5 rounded transition-colors mb-0.5 ${
        active
          ? 'bg-argus-amber/10 border border-argus-amberDim/40'
          : 'hover:bg-white/[0.04] border border-transparent'
      }`}
    >
      {/* Left: click to select */}
      <button
        onClick={() => onSelect(id)}
        className="flex items-center gap-2 flex-1 min-w-0 text-left"
      >
        <span className="text-sm">{m.icon}</span>
        <span className={`text-[12px] font-mono ${active ? 'text-argus-amber' : 'text-argus-text'}`}>
          {m.name}
        </span>
        {active && (
          <span
            className="text-[9px] font-mono tracking-widest"
            style={{ color: m.isPrimaryCoder ? '#4ade80' : undefined }}
          >
            {m.isPrimaryCoder ? 'BUILDER' : 'ACTIVE'}
          </span>
        )}
      </button>

      {/* Right: tool toggle (allied only) */}
      {showToolToggle && <ToolToggle modelId={id} />}
    </div>
  );
}
