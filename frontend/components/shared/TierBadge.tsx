'use client';

import { AccessTier } from '@/lib/types';

interface Props {
  tier: AccessTier;
  className?: string;
}

const TIER_CONFIG = {
  royal: { label: 'ROYAL', icon: '👑', bg: 'rgba(201,168,76,0.12)', color: '#c9a84c', border: 'rgba(201,168,76,0.3)' },
  allied: { label: 'ALLIED', icon: '🛡', bg: 'rgba(74,124,89,0.12)', color: '#4a7c59', border: 'rgba(74,124,89,0.3)' },
  guest:  { label: 'GUEST',  icon: '👁', bg: 'rgba(138,135,127,0.1)', color: '#8a877f', border: 'rgba(138,135,127,0.2)' },
};

export function TierBadge({ tier, className = '' }: Props) {
  const cfg = TIER_CONFIG[tier];
  return (
    <span
      className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-mono tracking-wider ${className}`}
      style={{ background: cfg.bg, color: cfg.color, border: `1px solid ${cfg.border}` }}
    >
      <span>{cfg.icon}</span>
      <span>{cfg.label}</span>
    </span>
  );
}
