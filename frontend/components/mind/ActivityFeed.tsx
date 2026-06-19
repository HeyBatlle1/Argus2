'use client';

import { ActivityEntry } from '@/lib/types';

const KIND_LABELS: Record<ActivityEntry['kind'], { color: string; tag: string }> = {
  tool:    { color: '#4a9c6b', tag: 'TOOL' },
  memory:  { color: '#6a7acc', tag: 'MEM' },
  discord: { color: '#5865f2', tag: 'DISC' },
  audit:   { color: '#5a5a7a', tag: 'AUDIT' },
};

function timeAgo(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const s = Math.floor(diff / 1000);
  if (s < 60) return `${s}s ago`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ago`;
  return `${Math.floor(m / 60)}h ago`;
}

interface Props {
  entries: ActivityEntry[];
}

export function ActivityFeed({ entries }: Props) {
  if (entries.length === 0) {
    return (
      <div className="text-center py-4">
        <p className="text-[10px] font-mono" style={{ color: '#3a3a5a' }}>
          No activity yet this session
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-0">
      {entries.map((e, i) => {
        const cfg = KIND_LABELS[e.kind] ?? KIND_LABELS.audit;
        return (
          <div
            key={e.id}
            className="flex items-start gap-2 py-1.5"
            style={{ borderBottom: i < entries.length - 1 ? '1px solid #0d0d18' : undefined }}
          >
            <span
              className="text-[7px] font-mono px-1 py-px rounded flex-shrink-0 mt-0.5"
              style={{ background: `${cfg.color}18`, color: cfg.color, letterSpacing: '0.05em' }}
            >
              {cfg.tag}
            </span>
            <div className="flex-1 min-w-0">
              <span className="text-[10px] font-mono truncate block" style={{ color: '#8a8a9a' }}>
                {e.label}
              </span>
              {e.detail && (
                <span className="text-[9px] font-mono truncate block" style={{ color: '#3a3a5a' }}>
                  {e.detail}
                </span>
              )}
            </div>
            <span className="text-[8px] font-mono flex-shrink-0" style={{ color: '#2a2a3a' }}>
              {timeAgo(e.ts)}
            </span>
          </div>
        );
      })}
    </div>
  );
}
