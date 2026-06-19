'use client';
import { motion } from 'framer-motion';
import { Tool } from '../../lib/types';

const STATE_COLOR: Record<Tool['state'], string> = { idle: '#32325a', active: '#4a8fc4', complete: '#4a7c59', error: '#8b1a1a' };

export function ToolStatus({ tool }: { tool: Tool }) {
  const color = STATE_COLOR[tool.state];
  return (
    <div className="flex justify-between py-1 text-[11px] font-mono group">
      <div className="flex items-center gap-2">
        <motion.span className="w-1.5 h-1.5 rounded-full" style={{ background: color }} animate={tool.state === 'active' ? { opacity: [1, 0.3, 1] } : {}} transition={{ duration: 0.7, repeat: Infinity }} />
        <span className="text-[#b8b5ac] group-hover:text-[#e8e5dc] transition-colors">{tool.label}</span>
      </div>
      {tool.callCount > 0 && <span className="text-[#b8b5ac] bg-white/5 px-1 rounded text-[9px]">{tool.callCount}</span>}
    </div>
  );
}
