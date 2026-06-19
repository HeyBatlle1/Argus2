'use client';
import { AccessTier } from '../../lib/types';

const CFG: any = {
  royal: { icon: '👑', label: 'ROYAL', color: '#c9a84c', bg: 'rgba(201,168,76,0.13)', b: 'rgba(201,168,76,0.3)' },
  allied: { icon: '🛡', label: 'ALLIED', color: '#4a7c59', bg: 'rgba(74,124,89,0.12)', b: 'rgba(74,124,89,0.3)' },
  guest: { icon: '👁', label: 'GUEST', color: '#8a877f', bg: 'rgba(138,135,127,0.1)', b: 'rgba(138,135,127,0.2)' },
};

export function TierBadge({ tier, className = '' }: { tier: AccessTier; className?: string }) {
  const c = CFG[tier];
  return <span className={`inline-flex items-center gap-1 px-1.5 py-px rounded text-[9px] font-mono tracking-wider ${className}`} style={{ background: c.bg, color: c.color, border: `1px solid ${c.b}` }}><span>{c.icon}</span><span>{c.label}</span></span>;
}
