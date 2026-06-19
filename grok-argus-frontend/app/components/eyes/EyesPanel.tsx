'use client';
import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronLeft } from 'lucide-react';
import { useAgentStore } from '../../hooks/useAgentState';
import { CollapsibleSection } from '../shared/CollapsibleSection';
import { EyeStateIndicator } from '../shared/EyeStateIndicator';
import { ToolStatus } from './ToolStatus';
import { VaultStatus } from './VaultStatus';
import { SystemInfo } from './SystemInfo';
import { EYE_LABELS } from '../../lib/constants';
import { MODEL_CONFIG } from '../../lib/models';

interface Props { forceCollapsed?: boolean; }

export function EyesPanel({ forceCollapsed = false }: Props) {
  const [collapsed, setCollapsed] = useState(false);
  const isCollapsed = forceCollapsed || collapsed;

  const eyeState = useAgentStore((s) => s.eyeState);
  const activeModel = useAgentStore((s) => s.activeModel);
  const tools = useAgentStore((s) => s.tools);
  const startTime = useAgentStore((s) => s.startTime);
  const vaultKeys = useAgentStore((s) => s.vaultKeys);
  const mcpServers = useAgentStore((s) => s.mcpServers);
  const connected = useAgentStore((s) => s.connected);

  const cfg = MODEL_CONFIG[activeModel];

  return (
    <motion.div animate={{ width: isCollapsed ? 32 : 238 }} transition={{ duration: 0.2 }} className="relative h-full border-r flex-shrink-0 overflow-hidden" style={{ background: '#0d0d16', borderColor: '#32325a' }}>
      <AnimatePresence>
        {isCollapsed && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="absolute inset-0 flex flex-col items-center pt-4 gap-3">
            <button onClick={() => setCollapsed(false)} className="text-[#b8b5ac] hover:text-[#f5b800]"><ChevronLeft size={14} className="rotate-180" /></button>
            <EyeStateIndicator state={eyeState} size="sm" />
          </motion.div>
        )}
      </AnimatePresence>

      <AnimatePresence>
        {!isCollapsed && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="absolute inset-0 flex flex-col">
            <div className="flex items-center justify-between px-3 py-2 border-b flex-shrink-0" style={{ background: '#1e1e38', borderColor: '#5a5a8a' }}>
              <span className="text-[9px] font-mono tracking-[0.14em] uppercase text-[#f5b800]">THE EYES</span>
              <button onClick={() => setCollapsed(true)} className="text-[#b8b5ac] hover:text-[#f5b800]"><ChevronLeft size={14} /></button>
            </div>

            <div className="flex-1 overflow-y-auto min-h-0">
              <CollapsibleSection title="Agent Status" defaultOpen>
                <div className="space-y-2">
                  <EyeStateIndicator state={eyeState} size="sm" showLabel />
                  <div className="flex justify-between text-[10px]"><span className="text-[#b8b5ac] font-mono tracking-widest">MODEL</span><span className="font-mono text-[#f5b800]">{cfg.name}</span></div>
                  <div className="flex justify-between text-[10px]"><span className="text-[#b8b5ac] font-mono tracking-widest">TIER</span><span className="font-mono" style={{ color: cfg.color }}>{cfg.icon} {cfg.tierLabel}</span></div>
                  {cfg.role && <div className="text-[9px] text-[#6a6a8a] font-mono italic">{cfg.role}</div>}
                </div>
              </CollapsibleSection>

              <CollapsibleSection title="Active Tools" count={tools.length} defaultOpen>
                <div className="divide-y divide-[#32325a]/30">
                  {tools.map(t => <ToolStatus key={t.name} tool={t} />)}
                </div>
              </CollapsibleSection>

              <CollapsibleSection title="Vault" defaultOpen>
                <VaultStatus locked={!connected} keys={vaultKeys} />
              </CollapsibleSection>

              <CollapsibleSection title="MCP Servers" count={mcpServers.length}>
                {mcpServers.length === 0 ? <p className="text-[10px] text-[#b8b5ac]">No servers connected.</p> : mcpServers.map(n => <div key={n} className="text-[10px] font-mono text-[#b8b5ac] flex items-center gap-1.5"><span className="w-1.5 h-1.5 bg-[#39d353] rounded-full" />{n}</div>)}
              </CollapsibleSection>

              <CollapsibleSection title="System">
                <SystemInfo />
              </CollapsibleSection>
            </div>

            {/* The Cell — Ferris behind bars (exact spirit, slightly refined) */}
            <div className="mx-2 mb-2 mt-2 flex-shrink-0" style={{ border: '2px solid #c9a84c', borderRadius: 6, background: '#06060e', overflow: 'hidden', boxShadow: '0 0 14px rgba(201,168,76,0.12)' }}>
              <div style={{ background: 'rgba(201,168,76,0.06)', padding: '3px 7px', borderBottom: '1px solid rgba(201,168,76,0.3)', display: 'flex', justifyContent: 'space-between', fontSize: 8, fontFamily: 'monospace', color: '#c9a84c', letterSpacing: '0.16em', textTransform: 'uppercase' }}>
                <span>SECURE HOLD</span><span style={{ color: '#4a4a2a' }}>CELL-GB2</span>
              </div>
              <svg viewBox="0 0 220 138" style={{ width: '100%', display: 'block' }}>
                <g transform="translate(110, 92)">
                  <animateTransform attributeName="transform" type="translate" values="110,92;110,89;110,92;110,94;110,92" dur="3.8s" repeatCount="indefinite" />
                  <ellipse cx="0" cy="0" rx="43" ry="30" fill="#ce4118" />
                  <ellipse cx="0" cy="4" rx="25" ry="16" fill="#d9561e" opacity="0.5" />
                  {/* Eyes looking up at the bars */}
                  <circle cx="-16" cy="-15" r="10" fill="#f0ebe0" />
                  <circle cx="-15" cy="-19" r="5.5" fill="#1a0a06" />
                  <circle cx="-13" cy="-21" r="1.6" fill="#fff" />
                  <circle cx="16" cy="-15" r="10" fill="#f0ebe0" />
                  <circle cx="17" cy="-19" r="5.5" fill="#1a0a06" />
                  <circle cx="19" cy="-21" r="1.6" fill="#fff" />
                  {/* Bars */}
                  <g opacity="0.78">{[18,46,74,102,130,158,186].map((x,i)=><rect key={i} x={x} y="-58" width="4.5" height="138" fill="#c9a84c" />)}</g>
                  <rect x="0" y="-58" width="220" height="4" fill="#c9a84c" opacity="0.6" />
                </g>
              </svg>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
