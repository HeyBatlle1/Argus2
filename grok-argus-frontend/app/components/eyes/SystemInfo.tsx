'use client';
import { useEffect, useState } from 'react';
import { useAgentStore } from '../../hooks/useAgentState';

export function SystemInfo() {
  const start = useAgentStore((s) => s.startTime);
  const lat = useAgentStore((s) => s.wsLatency);
  const [up, setUp] = useState('0s');

  useEffect(() => {
    const t = setInterval(() => {
      const s = Math.floor((Date.now() - start.getTime()) / 1000);
      setUp(s < 60 ? `${s}s` : s < 3600 ? `${Math.floor(s / 60)}m` : `${Math.floor(s / 3600)}h`);
    }, 1000);
    return () => clearInterval(t);
  }, [start]);

  const rows = [
    ['VERSION', 'GB2 / v0.2'],
    ['RUNTIME', 'Rust + Next'],
    ['LATENCY', `${lat}ms`],
    ['UPTIME', up],
  ];
  return <div className="space-y-1 text-[10px]">{rows.map(([l, v]) => <div key={l} className="flex justify-between"><span className="text-[#b8b5ac] font-mono tracking-widest">{l}</span><span className="font-mono">{v}</span></div>)}</div>;
}
