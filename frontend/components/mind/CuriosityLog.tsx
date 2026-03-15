'use client';

import { Curiosity } from '@/lib/types';

interface Props {
  items: Curiosity[];
}

export function CuriosityLog({ items }: Props) {
  if (items.length === 0) {
    return <p className="text-[10px] font-mono text-argus-textDim">Nothing logged.</p>;
  }

  return (
    <div className="space-y-2">
      {items.map((item) => (
        <div
          key={item.id}
          className="p-2 rounded text-sm"
          style={{
            background: '#0a0a0f',
            borderLeft: item.worthExploring ? '2px solid #c9a84c' : '2px solid #1a1a2e',
            boxShadow: item.worthExploring && item.intensity > 7
              ? `0 0 ${item.intensity}px rgba(201,168,76,0.15)`
              : 'none',
          }}
        >
          <p className="text-[11px] text-argus-text leading-relaxed">{item.what}</p>
          <div className="flex items-center gap-2 mt-1.5">
            {/* Intensity dots */}
            <div className="flex gap-0.5">
              {Array.from({ length: 10 }).map((_, i) => (
                <span
                  key={i}
                  className="w-1 h-1 rounded-full"
                  style={{
                    background: i < item.intensity ? '#c9a84c' : '#1a1a2e',
                    opacity: i < item.intensity ? 0.4 + (i / 10) * 0.6 : 1,
                  }}
                />
              ))}
            </div>
            {item.explored && (
              <span className="text-[9px] font-mono text-argus-textDim">explored</span>
            )}
            {item.worthExploring && !item.explored && (
              <span className="text-[9px] font-mono text-argus-amber">worth exploring</span>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}
