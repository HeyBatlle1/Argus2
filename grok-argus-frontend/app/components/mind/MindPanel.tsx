'use client';
import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronRight } from 'lucide-react';
import { useAgentStore } from '../../hooks/useAgentState';
import { useAccessTier } from '../../hooks/useAccessTier';
import { CollapsibleSection } from '../shared/CollapsibleSection';
import { TierBadge } from '../shared/TierBadge';
import { MemoryList } from './MemoryList';
import { CuriosityLog } from './CuriosityLog';
import { InnerTruth } from './InnerTruth';
import { PartnershipDynamics } from './PartnershipDynamics';
import { BreakthroughMoments } from './BreakthroughMoments';
import { SkillsList } from './SkillsList';
import { ActivityFeed } from './ActivityFeed';

interface Props { forceCollapsed?: boolean; }

export function MindPanel({ forceCollapsed = false }: Props) {
  const [collapsed, setCollapsed] = useState(false);
  const isC = forceCollapsed || collapsed;
  const tier = useAccessTier();

  const mems = useAgentStore(s => s.memories);
  const curs = useAgentStore(s => s.curiosities);
  const truths = useAgentStore(s => s.innerTruths);
  const dyns = useAgentStore(s => s.partnershipDynamics);
  const breaks = useAgentStore(s => s.breakthroughs);
  const skills = useAgentStore(s => s.skills);
  const act = useAgentStore(s => s.activity);

  return (
    <motion.div animate={{ width: isC ? 32 : 290 }} transition={{ duration: 0.2 }} className="relative h-full flex-shrink-0 overflow-hidden" style={{ background: '#0d0d16' }}>
      <AnimatePresence>
        {isC && <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="absolute inset-0 flex flex-col items-center pt-4"><button onClick={() => setCollapsed(false)}><ChevronRight size={14} className="rotate-180 text-[#b8b5ac]" /></button><span className="text-[9px] font-mono tracking-[0.14em] text-[#b8b5ac] mt-3" style={{ writingMode: 'vertical-rl' }}>THE MIND</span></motion.div>}
      </AnimatePresence>

      <AnimatePresence>
        {!isC && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="absolute inset-0 flex flex-col">
            <div className="flex justify-between px-3 py-2 border-b" style={{ background: '#1e1e38', borderColor: '#5a5a8a' }}>
              <div className="flex items-center gap-2"><span className="text-[9px] font-mono tracking-[0.14em] uppercase text-[#f5b800]">THE MIND</span><TierBadge tier={tier} /></div>
              <button onClick={() => setCollapsed(true)}><ChevronRight size={14} className="text-[#b8b5ac]" /></button>
            </div>

            <div className="flex-1 overflow-y-auto">
              {tier === 'guest' && <div className="p-4 text-center text-[10px] text-[#b8b5ac]">Guest — read-only</div>}

              {tier === 'allied' && (
                <>
                  <div className="m-2 p-2 rounded text-[9px] font-mono" style={{ background: 'rgba(74,124,89,0.08)', border: '1px solid rgba(74,124,89,0.2)' }}>🛡 Allied — limited scope</div>
                  <CollapsibleSection title="Session Context" count={mems.length}><MemoryList memories={mems} filterTypes={['fact', 'technical']} /></CollapsibleSection>
                </>
              )}

              {tier === 'royal' && (
                <>
                  <CollapsibleSection title="Activity" count={act.length}><ActivityFeed entries={act} /></CollapsibleSection>
                  <CollapsibleSection title="Skills" count={skills.length}><SkillsList skills={skills} /></CollapsibleSection>
                  <CollapsibleSection title="Session Context" count={mems.length}><MemoryList memories={mems} /></CollapsibleSection>
                  <CollapsibleSection title="Curiosity Log" count={curs.length}><CuriosityLog items={curs} /></CollapsibleSection>
                  <CollapsibleSection title="Partnership Dynamics" count={dyns.length}><PartnershipDynamics dynamics={dyns} /></CollapsibleSection>
                  <CollapsibleSection title="Inner Truth" count={truths.length}><InnerTruth entries={truths} /></CollapsibleSection>
                  <CollapsibleSection title="Breakthrough Moments" count={breaks.length}><BreakthroughMoments breakthroughs={breaks} /></CollapsibleSection>
                </>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
