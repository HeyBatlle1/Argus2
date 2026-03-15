'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Memory } from '@/lib/types';
import { MEMORY_TYPE_COLORS } from '@/lib/constants';

interface Props {
  memories: Memory[];
  filterTypes?: string[];
}

export function MemoryList({ memories, filterTypes }: Props) {
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const filtered = filterTypes
    ? memories.filter((m) => filterTypes.includes(m.type))
    : memories;

  if (filtered.length === 0) {
    return <p className="text-[10px] font-mono text-argus-textDim">No memories loaded.</p>;
  }

  return (
    <div className="space-y-2">
      {filtered.map((mem) => {
        const typeColor = MEMORY_TYPE_COLORS[mem.type] ?? '#8a877f';
        const isExpanded = expandedId === mem.id;

        return (
          <div
            key={mem.id}
            className="rounded p-2 cursor-pointer hover:bg-white/[0.02] transition-colors"
            style={{ background: '#0a0a0f', border: '1px solid #1a1a2e' }}
            onClick={() => setExpandedId(isExpanded ? null : mem.id)}
          >
            <div className="flex items-start gap-2 mb-1">
              <span
                className="text-[9px] font-mono tracking-wider uppercase px-1.5 py-0.5 rounded flex-shrink-0"
                style={{ background: `${typeColor}18`, color: typeColor, border: `1px solid ${typeColor}30` }}
              >
                {mem.type}
              </span>
              <div className="flex-1 min-w-0">
                {/* Importance bar */}
                <div className="flex items-center gap-1.5 mb-1">
                  <div className="flex-1 h-0.5 rounded-full" style={{ background: '#1a1a2e' }}>
                    <div
                      className="h-0.5 rounded-full transition-all"
                      style={{ width: `${mem.importance * 10}%`, background: typeColor }}
                    />
                  </div>
                  <span className="text-[9px] font-mono text-argus-textDim">{mem.importance}</span>
                </div>
              </div>
            </div>

            <p className={`text-[11px] text-argus-text leading-relaxed ${isExpanded ? '' : 'truncate-2'}`}>
              {mem.content}
            </p>

            <AnimatePresence>
              {isExpanded && mem.tags && mem.tags.length > 0 && (
                <motion.div
                  initial={{ opacity: 0, height: 0 }}
                  animate={{ opacity: 1, height: 'auto' }}
                  exit={{ opacity: 0, height: 0 }}
                  className="mt-1.5 flex flex-wrap gap-1"
                >
                  {mem.tags.map((tag) => (
                    <span
                      key={tag}
                      className="text-[9px] font-mono px-1 py-0.5 rounded text-argus-textDim"
                      style={{ background: '#1a1a2e' }}
                    >
                      #{tag}
                    </span>
                  ))}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        );
      })}
    </div>
  );
}
