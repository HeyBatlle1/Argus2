'use client';
import { useEffect, useState } from 'react';
import { useAgentStore } from '../../hooks/useAgentState';
import { EyeState } from '../../lib/types';
import { EYE_COLORS } from '../../lib/constants';

export function Watchtower() {
  const eye = useAgentStore(s => s.eyeState);
  const act = useAgentStore(s => s.watchtowerActivity);
  const [eyes, setEyes] = useState<number[]>(Array.from({ length: 27 }, () => Math.random()));

  // Pulse the eyes when activity or eye state changes
  useEffect(() => {
    setEyes(prev => prev.map((_, idx) => (idx % 3 === 0 ? 0.4 + Math.random() * 0.6 : prev[idx])));
  }, [eye, act]);

  const col = EYE_COLORS[eye];

  return (
    <div className="fixed bottom-3 left-3 z-[65] pointer-events-none select-none" title="The Hundred Eyes — live perception aggregate (Grok Build 2)">
      <div className="flex gap-px p-1 rounded" style={{ background: 'rgba(13,13,22,0.7)', border: '1px solid #1e1e32' }}>
        {eyes.map((v, i) => (
          <div key={i} className="w-[5px] h-2.5 rounded-sm transition-all" style={{ background: col, opacity: Math.max(0.15, v * (0.65 + (act / 22))) }} />
        ))}
      </div>
      <div className="text-[7px] text-center mt-px tracking-[1.5px] text-[#3a3a5a] font-mono">WATCHTOWER</div>
    </div>
  );
}
