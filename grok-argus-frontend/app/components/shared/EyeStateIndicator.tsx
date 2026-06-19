'use client';
import { motion, AnimatePresence } from 'framer-motion';
import { EyeState } from '../../lib/types';
import { EYE_SYMBOLS, EYE_COLORS, EYE_LABELS } from '../../lib/constants';

interface Props { state: EyeState; size?: 'sm' | 'md'; showLabel?: boolean; }

export function EyeStateIndicator({ state, size = 'md', showLabel }: Props) {
  const s = EYE_SYMBOLS[state]; const c = EYE_COLORS[state]; const l = EYE_LABELS[state];
  const variants: any = {
    watching: { scale: [1, 1.04, 1], opacity: [1, 0.65, 1], transition: { duration: 2.8, repeat: Infinity } },
    thinking: { rotate: [0, 360], transition: { duration: 2.4, repeat: Infinity, ease: 'linear' } },
    executing: { scale: [1, 1.18, 1], opacity: [1, 0.35, 1], transition: { duration: 0.7, repeat: Infinity } },
    complete: { scale: [1.45, 1], opacity: [0, 1], transition: { duration: 0.32 } },
  };
  const fs = size === 'sm' ? 'text-base' : 'text-xl';
  return (
    <span className="inline-flex items-center gap-1.5 font-mono">
      <AnimatePresence mode="wait">
        <motion.span key={state} style={{ color: c }} className={fs} animate={variants[state]}>{s}</motion.span>
      </AnimatePresence>
      {showLabel && <span className="text-[10px] tracking-[0.12em] uppercase" style={{ color: c }}>{l}</span>}
    </span>
  );
}
