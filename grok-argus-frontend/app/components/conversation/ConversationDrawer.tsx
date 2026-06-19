'use client';
import { useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Plus, Search } from 'lucide-react';
import { useAgentStore } from '../../hooks/useAgentState';

export function ConversationDrawer({ open, onClose }: { open: boolean; onClose: () => void }) {
  const convs = useAgentStore(s => s.conversations);
  const current = useAgentStore(s => s.currentConversationId);
  const load = useAgentStore(s => s.loadConversation);
  const nw = useAgentStore(s => s.newConversation);
  const [q, setQ] = useState('');
  const inp = useRef<HTMLInputElement>(null);

  useEffect(() => { if (open) setTimeout(() => inp.current?.focus(), 120); else setQ(''); }, [open]);

  const filtered = q ? convs.filter(c => c.title.toLowerCase().includes(q.toLowerCase())) : convs;

  return (
    <AnimatePresence>
      {open && <>
        <motion.div key="bd" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="fixed inset-0 z-40" style={{ background: 'rgba(0,0,0,0.55)' }} onClick={onClose} />
        <motion.div key="dr" initial={{ x: -300, opacity: 0 }} animate={{ x: 0, opacity: 1 }} exit={{ x: -300, opacity: 0 }} transition={{ ease: [0.32, 0.72, 0, 1] }} className="fixed left-0 top-0 bottom-0 z-50 w-[290px] flex flex-col" style={{ background: '#0d0d18', borderRight: '1px solid #1e1e32' }}>
          <div className="flex items-center justify-between px-4 pt-16 pb-2 border-b" style={{ borderColor: '#1e1e32' }}>
            <span className="text-[9px] font-mono tracking-[0.14em] uppercase text-[#f5b800]">CONVERSATIONS</span>
            <div className="flex gap-1">
              <button onClick={() => { nw(); onClose(); }} className="px-2 py-0.5 rounded text-[9px] font-mono flex items-center gap-1" style={{ background: 'rgba(255,176,0,0.1)', border: '1px solid rgba(255,176,0,0.35)', color: '#ffb000' }}><Plus size={10} />NEW</button>
              <button onClick={onClose} className="w-6 h-6 flex items-center justify-center rounded" style={{ background: 'rgba(255,255,255,0.04)', border: '1px solid #2a2a42' }}><X size={12} /></button>
            </div>
          </div>
          <div className="p-2 border-b" style={{ borderColor: '#1a1a2e' }}>
            <div className="flex items-center gap-2 px-2 py-1 rounded text-xs" style={{ background: '#111120', border: '1px solid #1e1e32' }}>
              <Search size={11} className="text-[#5a5a7a]" />
              <input ref={inp} value={q} onChange={e => setQ(e.target.value)} placeholder="Search…" className="bg-transparent flex-1 outline-none text-[#c8c8d8]" />
            </div>
          </div>
          <div className="flex-1 overflow-auto p-2 space-y-1 text-sm">
            {filtered.length === 0 && <div className="text-center text-[#3a3a5a] py-8 text-xs font-mono">No conversations</div>}
            {filtered.map(c => (
              <button key={c.id} onClick={() => { load(c.id); onClose(); }} className="w-full text-left px-3 py-2 rounded" style={{ background: c.id === current ? 'rgba(255,176,0,0.1)' : undefined, border: c.id === current ? '1px solid rgba(255,176,0,0.3)' : '1px solid transparent' }}>
                <div className="text-xs truncate">{c.title}</div>
                <div className="text-[9px] text-[#5a5a7a] mt-0.5 font-mono">{c.messageCount} turns • {c.model || 'mixed'}</div>
              </button>
            ))}
          </div>
        </motion.div>
      </>}
    </AnimatePresence>
  );
}
