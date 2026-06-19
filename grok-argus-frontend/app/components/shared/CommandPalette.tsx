'use client';
import React, { useState } from 'react';
import { useAgentStore } from '../../hooks/useAgentState';
import { MODEL_CONFIG, MODELS_IN_ORDER } from '../../lib/models';
import { ModelId } from '../../lib/types';

interface Props { open: boolean; onClose: () => void; }

export function CommandPalette({ open, onClose }: Props) {
  const [q, setQ] = useState('');
  const switchModel = useAgentStore((s) => s.switchModel);
  const newConv = useAgentStore((s) => s.newConversation);
  const schedule = useAgentStore((s) => s.scheduleTask);
  const send = useAgentStore((s) => s.sendMessage);

  if (!open) return null;

  const actions = [
    { id: 'new', label: 'New conversation', kbd: 'N', run: () => { newConv(); onClose(); } },
    { id: 'meeting', label: 'Start monthly meeting (all models)', kbd: 'M', run: () => { /* parent handles via key */ onClose(); window.dispatchEvent(new KeyboardEvent('keydown', { key: 'm' })); } },
    ...MODELS_IN_ORDER.map((id) => ({
      id: 'model-' + id,
      label: `Switch to ${MODEL_CONFIG[id].name}`,
      kbd: '',
      run: () => { switchModel(id as ModelId); onClose(); },
    })),
    { id: 'truth', label: 'Truth-check last response (Grok Build)', run: () => { onClose(); /* demo */ alert('Grok Build 2: Evidence scan complete. No contradictions found in this transcript.'); } },
    { id: 'deploy', label: 'Deploy a scheduled task now', run: () => { schedule('grok-build', null, 'Deep verification pass on current implementation'); onClose(); } },
  ].filter(a => a.label.toLowerCase().includes(q.toLowerCase()));

  return (
    <div className="cmdk" onClick={onClose}>
      <div className="cmdk-inner" onClick={e => e.stopPropagation()}>
        <input
          autoFocus
          className="cmdk-input"
          placeholder="Command or jump to model…  (esc to close)"
          value={q}
          onChange={(e) => setQ(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && actions[0]) actions[0].run();
            if (e.key === 'Escape') onClose();
          }}
        />
        <div className="cmdk-list">
          {actions.map((a, idx) => (
            <div key={a.id} className={`cmdk-item ${idx === 0 ? 'selected' : ''}`} onClick={a.run}>
              {a.label}
              {'kbd' in a && a.kbd && <span className="cmdk-kbd">{a.kbd}</span>}
            </div>
          ))}
          {actions.length === 0 && <div className="px-5 py-3 text-[#5a5a7a] text-sm font-mono">No matches</div>}
        </div>
        <div className="px-4 py-2 text-[10px] text-[#3a3a5a] border-t border-[#1e1e32] font-mono">Grok Build 2 • Argus1 • Everything is connected</div>
      </div>
    </div>
  );
}
