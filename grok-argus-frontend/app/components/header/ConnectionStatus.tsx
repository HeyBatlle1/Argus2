'use client';
import { motion } from 'framer-motion';
import { useAgentStore } from '../../hooks/useAgentState';

export function ConnectionStatus() {
  const connected = useAgentStore((s) => s.connected);
  const latency = useAgentStore((s) => s.wsLatency);

  const status = connected ? 'connected' : 'disconnected';
  const cfg = {
    connected: { color: '#39d353', label: 'CONNECTED' },
    disconnected: { color: '#ff4444', label: 'OFFLINE' },
  }[status];

  return (
    <div className="flex items-center gap-2 text-[11px] font-mono">
      <div className="relative w-2 h-2">
        <span className="block w-2 h-2 rounded-full" style={{ background: cfg.color }} />
        {connected && <motion.span className="absolute inset-0 rounded-full" style={{ background: cfg.color }} animate={{ scale: [1, 1.85], opacity: [0.45, 0] }} transition={{ duration: 2.6, repeat: Infinity }} />}
      </div>
      <span style={{ color: cfg.color, fontWeight: 700, letterSpacing: '0.06em' }}>{cfg.label}</span>
      {connected && <span className="text-[#b8b5ac] text-[10px]">{latency}ms</span>}
    </div>
  );
}
