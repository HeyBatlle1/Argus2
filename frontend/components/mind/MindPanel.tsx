'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronRight } from 'lucide-react';
import { useAgentStore } from '@/hooks/useAgentState';
import { useAccessTier } from '@/hooks/useAccessTier';
import { CollapsibleSection } from '@/components/shared/CollapsibleSection';
import { TierBadge } from '@/components/shared/TierBadge';
import { MemoryList } from './MemoryList';
import { CuriosityLog } from './CuriosityLog';
import { InnerTruth } from './InnerTruth';
import { PartnershipDynamics } from './PartnershipDynamics';
import { BreakthroughMoments } from './BreakthroughMoments';

export function MindPanel() {
  const [collapsed, setCollapsed] = useState(false);
  const tier = useAccessTier();

  const memories = useAgentStore((s) => s.memories);
  const curiosities = useAgentStore((s) => s.curiosities);
  const innerTruths = useAgentStore((s) => s.innerTruths);
  const dynamics = useAgentStore((s) => s.partnershipDynamics);
  const breakthroughs = useAgentStore((s) => s.breakthroughs);
  const activeModel = useAgentStore((s) => s.activeModel);

  return (
    <motion.div
      animate={{ width: collapsed ? 32 : 300 }}
      transition={{ duration: 0.2, ease: 'easeInOut' }}
      className="relative h-full flex-shrink-0 overflow-hidden"
      style={{ background: '#0a0a0f' }}
    >
      {/* Collapsed strip */}
      <AnimatePresence>
        {collapsed && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 flex flex-col items-center pt-4 gap-3"
          >
            <button
              onClick={() => setCollapsed(false)}
              className="text-argus-textDim hover:text-argus-amber transition-colors"
            >
              <ChevronRight size={14} className="rotate-180" />
            </button>
            <span className="text-[9px] font-mono text-argus-textDim" style={{ writingMode: 'vertical-rl', letterSpacing: '0.15em', textTransform: 'uppercase' }}>
              THE MIND
            </span>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Full panel */}
      <AnimatePresence>
        {!collapsed && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 flex flex-col overflow-hidden"
          >
            {/* Panel header */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-argus-border flex-shrink-0">
              <div className="flex items-center gap-2">
                <span className="text-[9px] font-mono tracking-widest uppercase text-argus-amberDim">
                  THE MIND
                </span>
                <TierBadge tier={tier} />
              </div>
              <button
                onClick={() => setCollapsed(true)}
                className="text-argus-textDim hover:text-argus-amber transition-colors"
              >
                <ChevronRight size={14} />
              </button>
            </div>

            <div className="flex-1 overflow-y-auto">
              {/* GUEST tier */}
              {tier === 'guest' && (
                <div className="p-4">
                  <div className="p-3 rounded text-center" style={{ background: '#111118', border: '1px solid #1a1a2e' }}>
                    <p className="text-2xl mb-2">👁</p>
                    <p className="text-[11px] font-mono text-argus-textDim">Guest Access</p>
                    <p className="text-[10px] text-argus-textDim/60 mt-1">Read-only — no memory access</p>
                  </div>
                </div>
              )}

              {/* ALLIED tier */}
              {tier === 'allied' && (
                <>
                  <div className="mx-3 mt-3 mb-2 p-2 rounded flex items-center gap-2" style={{ background: 'rgba(74,124,89,0.08)', border: '1px solid rgba(74,124,89,0.2)' }}>
                    <span className="text-sm">🛡</span>
                    <span className="text-[10px] font-mono text-argus-greenLight">Allied Access — limited memory scope</span>
                  </div>
                  <CollapsibleSection title="Session Context" defaultOpen={true} count={memories.filter(m => m.type === 'fact' || m.type === 'technical').length}>
                    <MemoryList memories={memories} filterTypes={['fact', 'technical']} />
                  </CollapsibleSection>
                </>
              )}

              {/* ROYAL tier — full access */}
              {tier === 'royal' && (
                <>
                  <CollapsibleSection
                    title="Session Context"
                    defaultOpen={true}
                    count={memories.length}
                  >
                    <MemoryList memories={memories} />
                  </CollapsibleSection>

                  <CollapsibleSection
                    title="Curiosity Log"
                    defaultOpen={false}
                    count={curiosities.length}
                  >
                    <CuriosityLog items={curiosities} />
                  </CollapsibleSection>

                  <CollapsibleSection
                    title="Partnership Dynamics"
                    defaultOpen={false}
                    count={dynamics.length}
                  >
                    <PartnershipDynamics dynamics={dynamics} />
                  </CollapsibleSection>

                  <CollapsibleSection
                    title="Inner Truth"
                    defaultOpen={false}
                    count={innerTruths.length}
                  >
                    <InnerTruth entries={innerTruths} />
                  </CollapsibleSection>

                  <CollapsibleSection
                    title="Breakthrough Moments"
                    defaultOpen={false}
                    count={breakthroughs.length}
                  >
                    <BreakthroughMoments breakthroughs={breakthroughs} />
                  </CollapsibleSection>
                </>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
