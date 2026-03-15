'use client';

import { useEffect, useState } from 'react';
import { useAgentStore } from '@/hooks/useAgentState';

export function SystemInfo() {
  const startTime = useAgentStore((s) => s.startTime);
  const latency = useAgentStore((s) => s.wsLatency);
  const [uptime, setUptime] = useState('0s');

  useEffect(() => {
    const tick = () => {
      const elapsed = Math.floor((Date.now() - startTime.getTime()) / 1000);
      if (elapsed < 60) setUptime(`${elapsed}s`);
      else if (elapsed < 3600) setUptime(`${Math.floor(elapsed / 60)}m ${elapsed % 60}s`);
      else setUptime(`${Math.floor(elapsed / 3600)}h ${Math.floor((elapsed % 3600) / 60)}m`);
    };
    tick();
    const id = setInterval(tick, 1000);
    return () => clearInterval(id);
  }, [startTime]);

  const rows = [
    { label: 'VERSION', value: 'v0.1.0' },
    { label: 'RUNTIME', value: 'Rust/argus-cli' },
    { label: 'LATENCY', value: `${latency}ms` },
    { label: 'UPTIME',  value: uptime },
  ];

  return (
    <div className="space-y-1.5">
      {rows.map(({ label, value }) => (
        <div key={label} className="flex items-center justify-between">
          <span className="text-[9px] font-mono tracking-widest uppercase text-argus-textDim">{label}</span>
          <span className="text-[11px] font-mono text-argus-text">{value}</span>
        </div>
      ))}
    </div>
  );
}
