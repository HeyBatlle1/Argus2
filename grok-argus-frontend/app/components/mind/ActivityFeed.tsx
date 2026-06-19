'use client';
import { ActivityEntry } from '../../lib/types';

const K: any = { tool: { c: '#4a9c6b', t: 'TOOL' }, memory: { c: '#6a7acc', t: 'MEM' }, discord: { c: '#5865f2', t: 'DISC' }, audit: { c: '#5a5a7a', t: 'AUD' } };

export function ActivityFeed({ entries }: { entries: ActivityEntry[] }) {
  if (!entries.length) return <div className="text-[10px] text-[#3a3a5a] py-3 text-center font-mono">No activity yet</div>;
  return <div className="space-y-px text-[10px]">{entries.slice(0, 12).map((e, i) => { const k = K[e.kind] || K.audit; return <div key={e.id} className="flex gap-2 py-1 px-1 border-b border-[#0d0d18] last:border-0"><span className="font-mono px-1 text-[8px] self-start mt-px" style={{ background: k.c + '22', color: k.c }}>{k.t}</span><div className="flex-1 min-w-0 text-[#8a8a9a] truncate">{e.label}</div></div>; })}</div>;
}
