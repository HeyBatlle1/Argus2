'use client';
import { motion, AnimatePresence } from 'framer-motion';
import { useAgentStore } from '../../hooks/useAgentState';
import { EYE_SYMBOLS, EYE_COLORS } from '../../lib/constants';

export function ArgusEye() {
  const eyeState = useAgentStore((s) => s.eyeState);
  const symbol = EYE_SYMBOLS[eyeState];
  const color = EYE_COLORS[eyeState];

  const glow = {
    watching: `0 0 8px rgba(74,124,89,0.5), 0 0 22px rgba(74,124,89,0.25)`,
    thinking: `0 0 9px rgba(201,168,76,0.7), 0 0 26px rgba(201,168,76,0.35)`,
    executing: `0 0 9px rgba(74,143,196,0.75), 0 0 26px rgba(74,143,196,0.45)`,
    complete: `0 0 14px rgba(255,255,255,0.85), 0 0 34px rgba(255,255,255,0.35)`,
  }[eyeState];

  const anim: any = {
    watching: { scale: [1, 1.06, 1], transition: { duration: 2.8, repeat: Infinity } },
    thinking: { rotate: [0, 360], transition: { duration: 2.6, repeat: Infinity, ease: 'linear' as const } },
    executing: { scale: [1, 1.22, 1], opacity: [1, 0.45, 1], transition: { duration: 0.75, repeat: Infinity } },
    complete: { scale: [1.55, 1], opacity: [0, 1], transition: { duration: 0.38 } },
  }[eyeState];

  return (
    <div className="relative flex items-center justify-center w-9 h-9">
      <motion.div className="absolute inset-0 rounded-full" animate={{ boxShadow: glow }} transition={{ duration: 0.35 }} />
      <AnimatePresence mode="wait">
        <motion.span key={eyeState} className="text-2xl font-mono leading-none relative z-10" style={{ color, display: 'inline-block' }} animate={anim}>
          {symbol}
        </motion.span>
      </AnimatePresence>
    </div>
  );
}
