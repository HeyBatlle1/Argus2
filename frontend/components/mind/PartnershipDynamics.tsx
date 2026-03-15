'use client';

import { PartnershipDynamic } from '@/lib/types';

interface Props {
  dynamics: PartnershipDynamic[];
}

const CATEGORY_COLORS: Record<string, string> = {
  communication: '#4a8fc4',
  collaboration:  '#c9a84c',
  values:         '#4a7c59',
  process:        '#8b5cf6',
};

export function PartnershipDynamics({ dynamics }: Props) {
  if (dynamics.length === 0) {
    return <p className="text-[10px] font-mono text-argus-textDim">No patterns logged.</p>;
  }

  return (
    <div className="space-y-2">
      {dynamics.map((d) => {
        const catColor = CATEGORY_COLORS[d.category] ?? '#8a877f';
        return (
          <div
            key={d.id}
            className="p-2 rounded"
            style={{ background: '#0a0a0f', border: '1px solid #1a1a2e' }}
          >
            <div className="flex items-start justify-between gap-2 mb-1">
              <span className="text-[11px] font-mono text-argus-text font-medium">{d.patternName}</span>
              <span
                className="text-[9px] font-mono tracking-wide px-1 py-0.5 rounded flex-shrink-0"
                style={{ background: `${catColor}15`, color: catColor }}
              >
                {d.category}
              </span>
            </div>
            {/* Importance bar */}
            <div className="flex items-center gap-1.5 mb-1.5">
              <div className="flex-1 h-0.5 rounded-full" style={{ background: '#1a1a2e' }}>
                <div
                  className="h-0.5 rounded-full"
                  style={{ width: `${d.importance * 10}%`, background: catColor }}
                />
              </div>
              <span className="text-[9px] font-mono text-argus-textDim">{d.importance}</span>
            </div>
            <p className="text-[10px] text-argus-textDim leading-relaxed">{d.description}</p>
          </div>
        );
      })}
    </div>
  );
}
