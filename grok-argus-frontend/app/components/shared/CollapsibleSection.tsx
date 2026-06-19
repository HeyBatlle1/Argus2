'use client';
import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronDown } from 'lucide-react';

interface Props { title: string; children: React.ReactNode; defaultOpen?: boolean; count?: number; }

export function CollapsibleSection({ title, children, defaultOpen = true, count }: Props) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className="border-b" style={{ borderColor: '#32325a' }}>
      <button onClick={() => setOpen(o => !o)} className="w-full flex justify-between px-3 py-2 text-left hover:bg-[#1e1e38] transition" style={{ background: '#16162a' }}>
        <span className="flex items-center gap-2 text-[10px] font-mono tracking-[0.12em] uppercase text-[#b8b5ac]">{title}{count !== undefined && <span className="text-[9px] bg-white/5 px-1 rounded text-[#b8b5ac]">{count}</span>}</span>
        <motion.span animate={{ rotate: open ? 0 : -90 }}><ChevronDown size={12} /></motion.span>
      </button>
      <AnimatePresence initial={false}>
        {open && <motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: 'auto', opacity: 1 }} exit={{ height: 0, opacity: 0 }} transition={{ duration: 0.18 }} style={{ overflow: 'hidden' }}><div className="px-3 pb-3 pt-1">{children}</div></motion.div>}
      </AnimatePresence>
    </div>
  );
}
