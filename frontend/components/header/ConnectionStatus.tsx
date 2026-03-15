'use client';

import { motion } from 'framer-motion';
import { useAgentStore } from '@/hooks/useAgentState';

type Status = 'connected' | 'disconnected' | 'reconnecting';

export function ConnectionStatus() {
  const connected = useAgentStore((s) => s.connected);
  const latency = useAgentStore((s) => s.wsLatency);

  // For now: connected if mock WS is initialized
  const status: Status = connected ? 'connected' : 'disconnected';

  const cfg = {
    connected:    { color: '#4a7c59', label: 'CONNECTED',    animate: false },
    disconnected: { color: '#8b1a1a', label: 'OFFLINE',      animate: false },
    reconnecting: { color: '#c9a84c', label: 'RECONNECTING', animate: true  },
  }[status];

  return (
    <div className="flex items-center gap-2">
      <div className="relative flex items-center justify-center w-2 h-2">
        <span
          className="w-2 h-2 rounded-full block"
          style={{ background: cfg.color }}
        />
        {cfg.animate && (
          <motion.span
            className="absolute inset-0 rounded-full"
            style={{ background: cfg.color }}
            animate={{ scale: [1, 2], opacity: [0.6, 0] }}
            transition={{ duration: 1, repeat: Infinity }}
          />
        )}
        {status === 'connected' && (
          <motion.span
            className="absolute inset-0 rounded-full"
            style={{ background: cfg.color }}
            animate={{ scale: [1, 1.8], opacity: [0.4, 0] }}
            transition={{ duration: 3, repeat: Infinity, ease: 'easeOut' }}
          />
        )}
      </div>
      <span className="text-[10px] font-mono tracking-widest" style={{ color: cfg.color }}>
        {cfg.label}
      </span>
      {status === 'connected' && (
        <span className="text-[10px] font-mono text-argus-textDim">{latency}ms</span>
      )}
    </div>
  );
}
