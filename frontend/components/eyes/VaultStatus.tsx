'use client';

import { Lock, Unlock, Key } from 'lucide-react';

interface Props {
  locked: boolean;
  keys: string[];
}

export function VaultStatus({ locked, keys }: Props) {
  return (
    <div>
      <div className="flex items-center gap-2 mb-2">
        {locked ? (
          <Lock size={12} className="text-argus-red" />
        ) : (
          <Unlock size={12} className="text-argus-greenLight" />
        )}
        <span
          className="text-[11px] font-mono"
          style={{ color: locked ? '#8b1a1a' : '#4a7c59' }}
        >
          {locked ? 'LOCKED' : 'UNLOCKED'}
        </span>
        <span className="text-[9px] font-mono text-argus-textDim ml-auto">
          {keys.length} keys
        </span>
      </div>
      <div className="space-y-1">
        {keys.map((k) => (
          <div key={k} className="flex items-center gap-1.5">
            <Key size={9} className="text-argus-amberDim flex-shrink-0" />
            <span className="text-[10px] font-mono text-argus-textDim truncate">{k}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
