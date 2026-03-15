'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronDown } from 'lucide-react';

interface Props {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
  count?: number;
  className?: string;
}

export function CollapsibleSection({ title, children, defaultOpen = true, count, className = '' }: Props) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className={`border-b border-argus-border ${className}`}>
      <button
        onClick={() => setOpen((o) => !o)}
        className="w-full flex items-center justify-between px-3 py-2 text-left hover:bg-white/[0.02] transition-colors group"
      >
        <span className="flex items-center gap-2">
          <span className="text-[10px] font-mono tracking-widest uppercase text-argus-textDim group-hover:text-argus-amber transition-colors">
            {title}
          </span>
          {count !== undefined && (
            <span className="text-[9px] font-mono text-argus-textDim bg-white/5 px-1 rounded">
              {count}
            </span>
          )}
        </span>
        <motion.span
          animate={{ rotate: open ? 0 : -90 }}
          transition={{ duration: 0.15 }}
          className="text-argus-textDim"
        >
          <ChevronDown size={12} />
        </motion.span>
      </button>

      <AnimatePresence initial={false}>
        {open && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2, ease: 'easeInOut' }}
            style={{ overflow: 'hidden' }}
          >
            <div className="px-3 pb-3 pt-1">
              {children}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
