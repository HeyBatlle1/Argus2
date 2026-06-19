'use client';


import { motion, AnimatePresence } from 'framer-motion';
import dynamic from 'next/dynamic';
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
import { SkillsList } from './SkillsList';
import { ActivityFeed } from './ActivityFeed';
import { SemanticField } from './SemanticField';
import { ScheduleChronicle } from './ScheduleChronicle';

// ExecutionFlow uses @xyflow/react which needs DOM — import with ssr:false
const ExecutionFlow = dynamic(
  () => import('./ExecutionFlow').then((m) => ({ default: m.ExecutionFlow })),
  { ssr: false, loading: () => (
    <div className="flex items-center justify-center h-full text-[#3a3a48] font-mono text-[10px]">
      loading flow…
    </div>
  )}
);

type View = 'mind' | 'field' | 'flow' | 'schedule';

interface Props {
  forceCollapsed?: boolean;
}

const TAB_LABELS: { id: View; label: string; hint: string }[] = [
  { id: 'mind',     label: 'MIND',   hint: 'memory · skills' },
  { id: 'field',    label: 'GRAPH',  hint: 'knowledge map' },
  { id: 'flow',     label: 'FLOW',   hint: 'tool graph' },
  { id: 'schedule', label: 'SCHED',  hint: 'tasks' },
];

export function MindPanel({ forceCollapsed = false }: Props) {
  const collapsed = useAgentStore((s) => s.mindCollapsed);
  const setCollapsed = useAgentStore((s) => s.setMindCollapsed);
  const view = useAgentStore((s) => s.mindView) as View;
  const setView = useAgentStore((s) => s.setMindView);
  const isCollapsed = forceCollapsed || collapsed;
  const tier = useAccessTier();

  const memories    = useAgentStore((s) => s.memories);
  const curiosities = useAgentStore((s) => s.curiosities);
  const innerTruths = useAgentStore((s) => s.innerTruths);
  const dynamics    = useAgentStore((s) => s.partnershipDynamics);
  const breakthroughs = useAgentStore((s) => s.breakthroughs);
  const skills      = useAgentStore((s) => s.skills);
  const activity    = useAgentStore((s) => s.activity);
  const activeToolCalls  = useAgentStore((s) => s.activeToolCalls);
  const scheduledTasks   = useAgentStore((s) => s.scheduledTasks);

  return (
    <motion.div
      animate={{ width: isCollapsed ? 32 : 300 }}
      transition={{ duration: 0.2, ease: 'easeInOut' }}
      className="relative h-full flex-shrink-0 overflow-hidden"
      style={{ background: '#070710' }}
    >
      {/* Collapsed strip */}
      <AnimatePresence>
        {isCollapsed && (
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
        {!isCollapsed && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 flex flex-col overflow-hidden"
          >
            {/* Panel header */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-argus-border flex-shrink-0" style={{ background: 'rgba(255,255,255,0.02)' }}>
              <div className="flex items-center gap-2">
                <span className="text-[9px] font-mono tracking-widest uppercase text-argus-amber">
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

            {/* Tab bar */}
            <div className="flex border-b border-argus-border flex-shrink-0" style={{ background: 'rgba(255,255,255,0.015)' }}>
              {TAB_LABELS.map(({ id, label }) => {
                const isActive = view === id;
                const badge =
                  id === 'flow' && activeToolCalls.length > 0 ? activeToolCalls.length :
                  id === 'schedule' && scheduledTasks.filter(t => t.status === 'pending').length > 0
                    ? scheduledTasks.filter(t => t.status === 'pending').length
                    : undefined;
                return (
                  <button
                    key={id}
                    onClick={() => setView(id)}
                    className="flex-1 py-1.5 text-[9px] font-mono tracking-[1.5px] relative transition-colors"
                    style={{ color: isActive ? '#f5b800' : '#3a3a48' }}
                  >
                    {label}
                    {badge !== undefined && (
                      <span className="ml-1 text-[8px] rounded-full px-1 py-px font-mono" style={{ background: 'rgba(103,246,255,0.15)', color: '#67f6ff' }}>
                        {badge}
                      </span>
                    )}
                    {isActive && (
                      <motion.div
                        layoutId="mind-tab-indicator"
                        className="absolute bottom-0 left-0 right-0 h-px"
                        style={{ background: '#f5b800' }}
                      />
                    )}
                  </button>
                );
              })}
            </div>

            {/* Tab content */}
            <div className="flex-1 overflow-hidden min-h-0">
              {view === 'field'    && <SemanticField />}
              {view === 'flow'     && <ExecutionFlow />}
              {view === 'schedule' && <ScheduleChronicle />}
              {view === 'mind'  && (
                <div className="h-full overflow-y-auto">
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
                      <CollapsibleSection title="Activity" defaultOpen={true} count={activity.length > 0 ? activity.length : undefined}>
                        <ActivityFeed entries={activity} />
                      </CollapsibleSection>

                      <CollapsibleSection title="Skills" defaultOpen={false} count={skills.length > 0 ? skills.length : undefined}>
                        <SkillsList skills={skills} />
                      </CollapsibleSection>

                      <CollapsibleSection title="Session Context" defaultOpen={false} count={memories.length}>
                        <MemoryList memories={memories} />
                      </CollapsibleSection>

                      <CollapsibleSection title="Curiosity Log" defaultOpen={false} count={curiosities.length}>
                        <CuriosityLog items={curiosities} />
                      </CollapsibleSection>

                      <CollapsibleSection title="Partnership Dynamics" defaultOpen={false} count={dynamics.length}>
                        <PartnershipDynamics dynamics={dynamics} />
                      </CollapsibleSection>

                      <CollapsibleSection title="Inner Truth" defaultOpen={false} count={innerTruths.length}>
                        <InnerTruth entries={innerTruths} />
                      </CollapsibleSection>

                      <CollapsibleSection title="Breakthrough Moments" defaultOpen={false} count={breakthroughs.length}>
                        <BreakthroughMoments breakthroughs={breakthroughs} />
                      </CollapsibleSection>
                    </>
                  )}
                </div>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
