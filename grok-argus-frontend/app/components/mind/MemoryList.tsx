'use client';
import { useState } from 'react';
import { Memory } from '../../lib/types';
import { MEMORY_TYPE_COLORS } from '../../lib/constants';

export function MemoryList({ memories, filterTypes }: { memories: Memory[]; filterTypes?: string[] }) {
  const [exp, setExp] = useState<string | null>(null);
  const list = filterTypes ? memories.filter(m => filterTypes.includes(m.type)) : memories;
  if (!list.length) return <div className="text-[10px] text-[#b8b5ac]">No memories.</div>;
  return <div className="space-y-1.5">{list.map(m => { const col = MEMORY_TYPE_COLORS[m.type] || '#8a877f'; const open = exp === m.id; return <div key={m.id} onClick={() => setExp(open ? null : m.id)} className="p-2 rounded cursor-pointer text-[11px]" style={{ background: '#16162a', border: '1px solid #32325a' }}><div className="flex gap-1.5 mb-0.5"><span className="font-mono text-[9px] px-1" style={{ background: col + '22', color: col, border: `1px solid ${col}40` }}>{m.type}</span><div className="flex-1 h-0.5 mt-1.5 rounded" style={{ background: '#32325a' }}><div className="h-0.5 rounded" style={{ width: `${m.importance * 10}%`, background: col }} /></div></div><div className={open ? '' : 'truncate-2'}>{m.content}</div></div>; })}</div>;
}
