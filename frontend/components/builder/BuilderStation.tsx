'use client';

import { useAgentStore } from '@/hooks/useAgentState';
import { MODEL_CONFIG } from '@/lib/models';
import { BUILDER_THEME, PRIMARY_CODER } from '@/lib/builder';

export function BuilderStation() {
  const activeModel = useAgentStore((s) => s.activeModel);
  const summonBuilder = useAgentStore((s) => s.summonBuilder);

  const cfg = MODEL_CONFIG[PRIMARY_CODER];
  const isActive = activeModel === PRIMARY_CODER;

  return (
    <div className="px-3 pb-2">
      <button
        onClick={() => summonBuilder()}
        className="w-full text-left rounded-lg overflow-hidden transition-all duration-200"
        style={{
          background: isActive
            ? `linear-gradient(135deg, ${BUILDER_THEME.surface} 0%, rgba(74,222,128,0.02) 100%)`
            : 'rgba(255,255,255,0.025)',
          border: isActive ? `1px solid ${BUILDER_THEME.border}` : '1px solid rgba(255,255,255,0.07)',
          boxShadow: isActive ? `0 0 24px ${BUILDER_THEME.glow}, inset 0 1px 0 rgba(255,255,255,0.05)` : 'none',
        }}
      >
        <div
          className="px-2.5 py-2 flex items-center justify-between"
          style={{ borderBottom: isActive ? `1px solid ${BUILDER_THEME.border}` : '1px solid rgba(255,255,255,0.05)' }}
        >
          <div className="flex items-center gap-1.5">
            <span className="text-sm">{cfg.icon}</span>
            <span className="text-[9px] font-mono tracking-[2px] uppercase" style={{ color: cfg.color }}>
              Builder Station
            </span>
          </div>
          <span
            className="text-[7px] font-mono px-1.5 py-px rounded uppercase tracking-wider"
            style={{
              background: isActive ? 'rgba(74,222,128,0.15)' : 'rgba(255,255,255,0.04)',
              color: isActive ? cfg.color : '#4a4a5a',
              border: `1px solid ${isActive ? BUILDER_THEME.border : 'rgba(255,255,255,0.06)'}`,
            }}
          >
            {isActive ? 'active' : 'summon'}
          </span>
        </div>

        <div className="px-2.5 py-2">
          <div className="text-[10px] font-mono font-medium" style={{ color: isActive ? cfg.color : '#8a8a9a' }}>
            {cfg.name}
          </div>
          <div className="text-[8px] font-mono mt-0.5" style={{ color: '#4a4a5a' }}>
            {cfg.role} · 12 tool rounds · full stack access
          </div>
          {!isActive && (
            <div className="text-[8px] font-mono mt-1.5" style={{ color: '#3a3a48' }}>
              ⌘B to summon · click to select
            </div>
          )}
        </div>
      </button>

    </div>
  );
}