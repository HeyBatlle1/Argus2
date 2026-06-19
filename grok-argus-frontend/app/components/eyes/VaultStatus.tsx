'use client';
import { Lock, Unlock, Key } from 'lucide-react';

export function VaultStatus({ locked, keys }: { locked: boolean; keys: string[] }) {
  return (
    <div>
      <div className="flex items-center gap-2 mb-1.5 text-[11px]">
        {locked ? <Lock size={11} className="text-[#ff4444]" /> : <Unlock size={11} className="text-[#39d353]" />}
        <span style={{ color: locked ? '#8b1a1a' : '#4a7c59' }}>{locked ? 'LOCKED' : 'UNLOCKED'}</span>
        <span className="ml-auto text-[#b8b5ac] text-[9px]">{keys.length} keys</span>
      </div>
      {keys.map(k => <div key={k} className="flex items-center gap-1.5 text-[10px] text-[#b8b5ac]"><Key size={9} className="text-[#c48a20]" />{k}</div>)}
    </div>
  );
}
