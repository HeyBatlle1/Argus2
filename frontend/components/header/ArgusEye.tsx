'use client';

import { motion, AnimatePresence } from 'framer-motion';
import { useAgentStore } from '@/hooks/useAgentState';
import { EYE_SYMBOLS, EYE_COLORS } from '@/lib/constants';

export function ArgusEye() {
  const eyeState = useAgentStore((s) => s.eyeState);
  const symbol = EYE_SYMBOLS[eyeState];
  const color = EYE_COLORS[eyeState];

  const glowMap = {
    watching:  `0 0 8px rgba(74,124,89,0.5), 0 0 20px rgba(74,124,89,0.2)`,
    thinking:  `0 0 8px rgba(201,168,76,0.6), 0 0 20px rgba(201,168,76,0.3)`,
    executing: `0 0 8px rgba(74,143,196,0.7), 0 0 24px rgba(74,143,196,0.4)`,
    complete:  `0 0 12px rgba(255,255,255,0.8), 0 0 30px rgba(255,255,255,0.3)`,
  };

  const animations = {
    watching:  { scale: [1, 1.05, 1], transition: { duration: 3, repeat: Infinity, ease: 'easeInOut' } },
    thinking:  { rotate: [0, 360], transition: { duration: 3, repeat: Infinity, ease: 'linear' } },
    executing: { scale: [1, 1.2, 1], opacity: [1, 0.5, 1], transition: { duration: 0.8, repeat: Infinity, ease: 'easeInOut' } },
    complete:  { scale: [1.5, 1], opacity: [0, 1], transition: { duration: 0.4, ease: 'easeOut' } },
  };

  return (
    <div className="relative flex items-center justify-center w-9 h-9">
      {/* Halo */}
      <motion.div
        className="absolute inset-0 rounded-full"
        animate={{ boxShadow: glowMap[eyeState] }}
        transition={{ duration: 0.4 }}
      />
      {/* Eye symbol */}
      <AnimatePresence mode="wait">
        <motion.span
          key={eyeState}
          className="text-2xl font-mono leading-none relative z-10"
          style={{ color, display: 'inline-block' }}
          animate={animations[eyeState]}
        >
          {symbol}
        </motion.span>
      </AnimatePresence>
    </div>
  );
}
