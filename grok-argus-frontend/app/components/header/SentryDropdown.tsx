'use client';
import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Activity, ChevronDown } from 'lucide-react';

export function SentryDropdown() {
  const [open, setOpen] = useState(false);
  // Static demo data — in real would poll /api/sentry
  const data = { memory: { used: '9.2G', free: '4.8G' }, containers: [{ name: 'argus-daemon', healthy: true }, { name: 'argus-workspace', healthy: true }], ts: Date.now() };
  const warn = false;

  return (
    <div className="relative">
      <button onClick={() => setOpen(!open)} className="flex items-center gap-1.5 px-2 py-1 rounded border text-[10px] font-mono" style={{ borderColor: warn ? '#8b1a1a' : '#32325a', color: warn ? '#ff4444' : '#b8b5ac' }}>
        <Activity size={12} /> SENTRY <ChevronDown size={10} />
      </button>
      <AnimatePresence>
        {open && (
          <>
            <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
            <motion.div initial={{ opacity: 0, y: -4 }} animate={{ opacity: 1, y: 0 }} className="absolute right-0 mt-1 z-50 w-72 rounded border p-3 text-[10px] font-mono" style={{ background: '#12121f', borderColor: '#32325a' }}>
              <div className="text-[#f5b800] mb-2 tracking-widest">SYSTEM SENTRY</div>
              <div>RAM: {data.memory.used} used — {data.memory.free} free</div>
              <div className="mt-2">Docker:</div>
              {data.containers.map(c => <div key={c.name} className="flex justify-between"><span>{c.name}</span><span style={{ color: c.healthy ? '#39d353' : '#ff4444' }}>{c.healthy ? 'HEALTHY' : 'DOWN'}</span></div>)}
              <div className="text-[#5a5a7a] mt-2 text-[9px]">Live in prod via daemon /sentry</div>
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </div>
  );
}
