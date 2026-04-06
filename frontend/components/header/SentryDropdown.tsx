'use client';

import { useState, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronDown, Activity } from 'lucide-react';

interface SentryData {
  memory: { used: string; free: string };
  containers: { name: string; status: string; ports: string; healthy: boolean; unhealthy: boolean }[];
  processes: { name: string; pid: string; mem: string; uptime: string }[];
  ts: number;
}

function MemBar({ used, free }: { used: string; free: string }) {
  // Extract numeric value + unit to compute rough percentage
  const parse = (s: string) => {
    const m = s.match(/([\d.]+)([GM])/);
    if (!m) return 0;
    return parseFloat(m[1]) * (m[2] === 'G' ? 1024 : 1);
  };
  const usedMb = parse(used);
  const freeMb = parse(free);
  const total = usedMb + freeMb;
  const pct = total > 0 ? Math.round((usedMb / total) * 100) : 0;
  const color = pct > 90 ? '#ff4444' : pct > 75 ? '#f0a500' : '#39d353';

  return (
    <div className="space-y-1">
      <div className="flex justify-between text-[10px] font-mono">
        <span className="text-argus-textDim">RAM</span>
        <span style={{ color }}>{pct}% used — {free} free</span>
      </div>
      <div className="w-full h-1.5 rounded-full bg-argus-surface overflow-hidden">
        <motion.div
          className="h-full rounded-full"
          style={{ background: color }}
          initial={{ width: 0 }}
          animate={{ width: `${pct}%` }}
          transition={{ duration: 0.4, ease: 'easeOut' }}
        />
      </div>
    </div>
  );
}

function StatusDot({ ok, warn }: { ok: boolean; warn?: boolean }) {
  const color = warn ? '#ff4444' : ok ? '#39d353' : '#9d9a91';
  return (
    <span className="relative flex items-center justify-center w-2 h-2 flex-shrink-0">
      <span className="w-2 h-2 rounded-full block" style={{ background: color }} />
      {ok && !warn && (
        <motion.span
          className="absolute inset-0 rounded-full"
          style={{ background: color }}
          animate={{ scale: [1, 1.8], opacity: [0.5, 0] }}
          transition={{ duration: 2.5, repeat: Infinity, ease: 'easeOut' }}
        />
      )}
    </span>
  );
}

export function SentryDropdown() {
  const [open, setOpen] = useState(false);
  const [data, setData] = useState<SentryData | null>(null);
  const [loading, setLoading] = useState(false);

  const poll = useCallback(async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/sentry');
      if (res.ok) setData(await res.json());
    } finally {
      setLoading(false);
    }
  }, []);

  // Poll immediately on open, then every 10s
  useEffect(() => {
    poll();
    const id = setInterval(poll, 10_000);
    return () => clearInterval(id);
  }, [poll]);

  const freeNum = data ? parseFloat(data.memory.free) : null;
  const freeUnit = data ? data.memory.free.replace(/[\d.]/g, '') : '';
  const ramWarn = freeNum !== null && ((freeUnit === 'M' && freeNum < 500) || (freeUnit === 'G' && freeNum < 0.5));

  return (
    <div className="relative">
      <button
        onClick={() => setOpen((o) => !o)}
        className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded border transition-colors ${
          ramWarn
            ? 'border-argus-red/60 bg-argus-redDim/20 hover:border-argus-red'
            : 'border-argus-border bg-argus-surface hover:border-argus-amberDim'
        }`}
        title="System Sentry"
      >
        <Activity
          size={12}
          className={ramWarn ? 'text-argus-red animate-pulse-rapid' : 'text-argus-amberDim'}
        />
        <span className={`text-[10px] font-mono tracking-wider ${ramWarn ? 'text-argus-red' : 'text-argus-textDim'}`}>
          {data ? (ramWarn ? `LOW: ${data.memory.free}` : data.memory.free) : 'SENTRY'}
        </span>
        {loading && (
          <span className="w-1 h-1 rounded-full bg-argus-amberDim animate-pulse-rapid" />
        )}
        <motion.span
          animate={{ rotate: open ? 180 : 0 }}
          transition={{ duration: 0.15 }}
          className="text-argus-textDim"
        >
          <ChevronDown size={11} />
        </motion.span>
      </button>

      <AnimatePresence>
        {open && (
          <>
            <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
            <motion.div
              initial={{ opacity: 0, y: -4, scale: 0.97 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: -4, scale: 0.97 }}
              transition={{ duration: 0.15 }}
              className="absolute right-0 top-full mt-1 z-50 w-72 rounded border border-argus-border bg-argus-bg2 shadow-2xl overflow-hidden"
            >
              {/* Header */}
              <div className="px-3 py-2 border-b border-argus-border flex items-center justify-between">
                <span className="text-[9px] font-mono tracking-widest uppercase text-argus-amberDim">
                  System Sentry
                </span>
                {data && (
                  <span className="text-[9px] font-mono text-argus-textDim">
                    {new Date(data.ts).toLocaleTimeString()}
                  </span>
                )}
              </div>

              <div className="px-3 py-2.5 space-y-3">
                {/* Memory */}
                {data ? (
                  <MemBar used={data.memory.used} free={data.memory.free} />
                ) : (
                  <div className="h-6 bg-argus-surface rounded animate-pulse" />
                )}

                {/* Docker Containers */}
                <div>
                  <div className="text-[9px] font-mono tracking-widest uppercase text-argus-textDim mb-1.5">
                    Docker
                  </div>
                  {data && data.containers.length > 0 ? (
                    <div className="space-y-1">
                      {data.containers.map((c) => (
                        <div key={c.name} className="flex items-center gap-2 px-2 py-1 rounded bg-argus-surface">
                          <StatusDot ok={c.healthy} warn={c.unhealthy} />
                          <span className="text-[11px] font-mono text-argus-text flex-1">{c.name}</span>
                          <span className={`text-[9px] font-mono ${
                            c.unhealthy ? 'text-argus-red' : c.healthy ? 'text-argus-greenLight' : 'text-argus-textDim'
                          }`}>
                            {c.unhealthy ? 'UNHEALTHY' : c.healthy ? 'HEALTHY' : 'UP'}
                          </span>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="px-2 py-1 rounded bg-argus-surface">
                      <span className="text-[11px] font-mono text-argus-textDim">No containers running</span>
                    </div>
                  )}
                </div>

                {/* Native Daemons */}
                <div>
                  <div className="text-[9px] font-mono tracking-widest uppercase text-argus-textDim mb-1.5">
                    Native Daemons
                  </div>
                  {data && data.processes.length > 0 ? (
                    <div className="space-y-1">
                      {data.processes.map((p) => (
                        <div key={p.pid} className="flex items-center gap-2 px-2 py-1 rounded bg-argus-surface">
                          <StatusDot ok={true} />
                          <span className="text-[11px] font-mono text-argus-text flex-1">{p.name}</span>
                          <span className="text-[9px] font-mono text-argus-textDim">{p.mem}</span>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="px-2 py-1 rounded bg-argus-surface">
                      <span className="text-[11px] font-mono text-argus-textDim">Scanning...</span>
                    </div>
                  )}
                </div>
              </div>

              {/* Footer */}
              <div className="px-3 py-1.5 border-t border-argus-border">
                <button
                  onClick={poll}
                  className="text-[9px] font-mono tracking-widest uppercase text-argus-amberDim hover:text-argus-amber transition-colors"
                >
                  ↻ refresh
                </button>
              </div>
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </div>
  );
}
