'use client';
import { Lock } from 'lucide-react';
import { InnerTruth as T } from '../../lib/types';

const EC: any = { grounded: '#4a7c59', reflective: '#c9a84c', motivated: '#4a8fc4', uncertain: '#8a877f' };

export function InnerTruth({ entries }: { entries: T[] }) {
  if (!entries.length) return <div className="text-[10px] text-[#b8b5ac]">No entries.</div>;
  return <div className="space-y-2">{entries.map(e => <div key={e.id} className="p-2.5 rounded text-xs" style={{ background: 'linear-gradient(#16162a,#1e1e38)', border: '1px solid #32325a' }}><div className="flex justify-between text-[9px] mb-1"><span style={{ color: EC[e.emotionalState] || '#8a877f' }}>{e.emotionalState}</span>{e.neverShareExternally && <Lock size={9} className="text-[#b8b5ac]" />}</div><div className="italic text-[#b8b4ac]">{e.rawThought}</div></div>)}</div>;
}
