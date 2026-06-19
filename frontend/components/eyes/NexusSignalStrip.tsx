'use client';

import { useAgentStore } from '@/hooks/useAgentState';
import { EyeStateIndicator } from '@/components/shared/EyeStateIndicator';
import { MODEL_CONFIG } from '@/lib/models';
import { EYE_COLORS } from '@/lib/constants';

function timeAgo(iso: string): string {
  const sec = Math.floor((Date.now() - new Date(iso).getTime()) / 1000);
  if (sec < 5) return 'now';
  if (sec < 60) return `${sec}s`;
  if (sec < 3600) return `${Math.floor(sec / 60)}m`;
  return `${Math.floor(sec / 3600)}h`;
}

export function NexusSignalStrip() {
  const eyeState     = useAgentStore((s) => s.eyeState);
  const connected    = useAgentStore((s) => s.connected);
  const wsLatency    = useAgentStore((s) => s.wsLatency);
  const activeModel  = useAgentStore((s) => s.activeModel);
  const accessTier   = useAgentStore((s) => s.accessTier);
  const memories     = useAgentStore((s) => s.memories);
  const activity     = useAgentStore((s) => s.activity);
  const activeToolCalls = useAgentStore((s) => s.activeToolCalls);

  const modelCfg = MODEL_CONFIG[activeModel];
  const last = activity[0];
  const stateColor = EYE_COLORS[eyeState];

  return (
    <div
      className="mx-3 mb-3 rounded-lg overflow-hidden"
      style={{
        background: 'linear-gradient(135deg, rgba(255,255,255,0.03) 0%, rgba(103,246,255,0.04) 100%)',
        border: '1px solid rgba(103,246,255,0.12)',
        boxShadow: `inset 0 1px 0 rgba(255,255,255,0.04), 0 0 20px ${stateColor}18`,
      }}
    >
      {/* Row 1 — presence */}
      <div className="flex items-center justify-between px-2.5 py-1.5 border-b" style={{ borderColor: 'rgba(255,255,255,0.05)' }}>
        <div className="flex items-center gap-1.5">
          <EyeStateIndicator state={eyeState} size="xs" showLabel />
        </div>
        <div className="flex items-center gap-2 text-[8px] font-mono" style={{ color: '#5a5a68' }}>
          <span
            className="w-1.5 h-1.5 rounded-full flex-shrink-0"
            style={{
              background: connected ? '#39d353' : '#3a3a48',
              boxShadow: connected ? '0 0 6px rgba(57,211,83,0.5)' : 'none',
            }}
          />
          {connected ? `${wsLatency}ms` : 'offline'}
        </div>
      </div>

      {/* Row 2 — who is here */}
      <div className="px-2.5 py-1.5 flex items-center gap-1.5 min-w-0">
        <span
          className="w-1.5 h-1.5 rounded-full flex-shrink-0"
          style={{ background: modelCfg.color, boxShadow: `0 0 6px ${modelCfg.color}66` }}
        />
        <span className="text-[9px] font-mono truncate" style={{ color: modelCfg.color }}>
          {modelCfg.name.toUpperCase()}
        </span>
        <span className="text-[8px] font-mono uppercase" style={{ color: '#3a3a48' }}>
          · {accessTier}
        </span>
        <span className="text-[8px] font-mono ml-auto flex-shrink-0" style={{ color: '#4a4a5a' }}>
          {memories.length} mem
        </span>
      </div>

      {/* Row 3 — last signal */}
      <div className="px-2.5 py-1.5 text-[8px] font-mono leading-snug" style={{ color: '#4a4a5a', background: 'rgba(0,0,0,0.2)' }}>
        {activeToolCalls.length > 0 ? (
          <span style={{ color: '#67f6ff' }}>
            running {activeToolCalls[activeToolCalls.length - 1].name}…
          </span>
        ) : last ? (
          <>
            <span style={{ color: '#6a6a7a' }}>last </span>
            <span style={{ color: '#8a8a9a' }}>
              {last.kind === 'tool' ? last.label : last.label}
            </span>
            <span style={{ color: '#3a3a48' }}> · {timeAgo(last.ts)}</span>
          </>
        ) : (
          <span style={{ color: '#3a3a48', fontStyle: 'italic' }}>awaiting signal</span>
        )}
      </div>
    </div>
  );
}