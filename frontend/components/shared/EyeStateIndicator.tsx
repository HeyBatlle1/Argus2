'use client';

import { motion, AnimatePresence } from 'framer-motion';
import { EyeState } from '@/lib/types';
import { EYE_SYMBOLS, EYE_COLORS, EYE_LABELS } from '@/lib/constants';

interface Props {
  state: EyeState;
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  showLabel?: boolean;
  className?: string;
}

const SIZE_CLASS = {
  xs: 'text-sm',
  sm: 'text-base',
  md: 'text-xl',
  lg: 'text-3xl',
  xl: 'text-5xl',
};

const LABEL_SIZE = {
  xs: 'text-[9px]',
  sm: 'text-[10px]',
  md: 'text-xs',
  lg: 'text-sm',
  xl: 'text-base',
};

export function EyeStateIndicator({ state, size = 'md', showLabel = false, className = '' }: Props) {
  const symbol = EYE_SYMBOLS[state];
  const color = EYE_COLORS[state];
  const label = EYE_LABELS[state];

  const variants = {
    watching: {
      scale: [1, 1.04, 1],
      opacity: [1, 0.7, 1],
      transition: { duration: 3, repeat: Infinity, ease: 'easeInOut' },
    },
    thinking: {
      rotate: [0, 360],
      transition: { duration: 3, repeat: Infinity, ease: 'linear' },
    },
    executing: {
      scale: [1, 1.15, 1],
      opacity: [1, 0.4, 1],
      transition: { duration: 0.8, repeat: Infinity, ease: 'easeInOut' },
    },
    complete: {
      scale: [1.4, 1],
      opacity: [0, 1],
      transition: { duration: 0.4, ease: 'easeOut' },
    },
  };

  return (
    <span className={`inline-flex items-center gap-1.5 font-mono ${className}`}>
      <AnimatePresence mode="wait">
        <motion.span
          key={state}
          style={{ color, display: 'inline-block' }}
          className={`${SIZE_CLASS[size]} leading-none`}
          animate={variants[state]}
        >
          {symbol}
        </motion.span>
      </AnimatePresence>
      {showLabel && (
        <span
          className={`${LABEL_SIZE[size]} tracking-widest uppercase font-mono`}
          style={{ color }}
        >
          {label}
        </span>
      )}
    </span>
  );
}
