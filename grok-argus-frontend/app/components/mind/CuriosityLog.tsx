'use client';
import { Curiosity } from '../../lib/types';

export function CuriosityLog({ items }: { items: Curiosity[] }) {
  if (!items.length) return <div className="text-[10px] text-[#b8b5ac]">Nothing logged.</div>;
  return <div className="space-y-1.5">{items.map(it => <div key={it.id} className="p-2 rounded text-xs" style={{ background: '#16162a', borderLeft: it.worthExploring ? '2px solid #f5b800' : '2px solid #32325a' }}>{it.what}<div className="mt-1 flex gap-0.5">{Array.from({ length: 8 }).map((_, i) => <span key={i} className="w-0.5 h-0.5 rounded" style={{ background: i < it.intensity ? '#f5b800' : '#32325a' }} />)}</div></div>)}</div>;
}
