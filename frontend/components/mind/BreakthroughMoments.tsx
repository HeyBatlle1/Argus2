'use client';

import { motion } from 'framer-motion';
import { Breakthrough } from '@/lib/types';

interface Props {
  breakthroughs: Breakthrough[];
}

export function BreakthroughMoments({ breakthroughs }: Props) {
  if (breakthroughs.length === 0) {
    return <p className="text-[10px] font-mono text-argus-textDim">No milestones yet.</p>;
  }

  return (
    <div className="space-y-2">
      {breakthroughs.map((b) => (
        <motion.div
          key={b.id}
          className="p-2.5 rounded"
          style={{
            background: '#0a0a0f',
            border: '1px solid #1a1a2e',
            boxShadow: `0 0 ${b.emotionalWeight * 1.5}px rgba(201,168,76,${b.emotionalWeight / 80})`,
          }}
          whileHover={{
            boxShadow: `0 0 ${b.emotionalWeight * 3}px rgba(201,168,76,${b.emotionalWeight / 40})`,
          }}
        >
          <div className="flex items-center justify-between mb-1">
            <span className="text-[11px] font-mono text-argus-amber font-medium">✦ {b.title}</span>
            <span className="text-[9px] font-mono text-argus-textDim flex-shrink-0 ml-2">
              {b.createdAt.toLocaleDateString()}
            </span>
          </div>
          <p className="text-[10px] text-argus-textDim leading-relaxed">{b.description}</p>
          {/* Emotional weight */}
          <div className="flex items-center gap-1.5 mt-1.5">
            <div className="flex-1 h-0.5 rounded-full" style={{ background: '#1a1a2e' }}>
              <div
                className="h-0.5 rounded-full"
                style={{ width: `${b.emotionalWeight * 10}%`, background: 'linear-gradient(to right, #8a6f2e, #c9a84c)' }}
              />
            </div>
          </div>
        </motion.div>
      ))}
    </div>
  );
}
