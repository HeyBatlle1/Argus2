'use client';

import { motion, AnimatePresence } from 'framer-motion';
import { ChevronLeft } from 'lucide-react';
import { useAgentStore } from '@/hooks/useAgentState';
import { CollapsibleSection } from '@/components/shared/CollapsibleSection';
import { EyeStateIndicator } from '@/components/shared/EyeStateIndicator';
import { ToolStatus } from './ToolStatus';
import { VaultStatus } from './VaultStatus';
import { SystemInfo } from './SystemInfo';
import { NexusCore } from './NexusCore';
import { NexusSignalStrip } from './NexusSignalStrip';
import { BuilderStation } from '@/components/builder/BuilderStation';
import { MODEL_CONFIG, CONSTELLATION_MODELS } from '@/lib/models';
import { isBuilderModel } from '@/lib/builder';

interface Props {
  forceCollapsed?: boolean;
}

export function EyesPanel({ forceCollapsed = false }: Props) {
  const collapsed = useAgentStore((s) => s.eyesCollapsed);
  const setCollapsed = useAgentStore((s) => s.setEyesCollapsed);
  const isCollapsed = forceCollapsed || collapsed;

  const eyeState    = useAgentStore((s) => s.eyeState);
  const activeModel = useAgentStore((s) => s.activeModel);
  const corePulse   = useAgentStore((s) => s.corePulse);
  const tools       = useAgentStore((s) => s.tools);
  const vaultKeys   = useAgentStore((s) => s.vaultKeys);
  const mcpServers  = useAgentStore((s) => s.mcpServers);
  const connected   = useAgentStore((s) => s.connected);
  const activity    = useAgentStore((s) => s.activity);
  const switchModel = useAgentStore((s) => s.switchModel);

  const modelCfg = MODEL_CONFIG[activeModel];
  const builderMode = isBuilderModel(activeModel);
  const discourse = activity.slice(0, 3);

  const constellation = CONSTELLATION_MODELS.filter((m) => m !== 'grok-build');

  return (
    <motion.div
      animate={{ width: isCollapsed ? 32 : 260 }}
      transition={{ duration: 0.2, ease: 'easeInOut' }}
      className="relative h-full border-r border-argus-border flex-shrink-0 overflow-hidden"
      style={{ background: '#070710' }}
    >
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
              <ChevronLeft size={14} className="rotate-180" />
            </button>
            <EyeStateIndicator state={eyeState} size="sm" />
          </motion.div>
        )}
      </AnimatePresence>

      <AnimatePresence>
        {!isCollapsed && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 flex flex-col"
          >
            <div className="flex items-center justify-between px-3 py-2 border-b border-argus-border flex-shrink-0" style={{ background: 'rgba(255,255,255,0.02)' }}>
              <span className="text-[9px] font-mono tracking-widest uppercase" style={{ color: builderMode ? '#4ade80' : '#67f6ff' }}>
                {builderMode ? 'BUILDER EYES' : 'THE EYES'}
              </span>
              <button
                onClick={() => setCollapsed(true)}
                className="text-argus-textDim hover:text-argus-amber transition-colors"
              >
                <ChevronLeft size={14} />
              </button>
            </div>

            <div className="flex-1 overflow-y-auto min-h-0">
              <div className="flex justify-center pt-3 pb-1">
                <NexusCore eyeState={eyeState} pulse={corePulse} size={228} builderMode={builderMode} />
              </div>

              <BuilderStation />
              <NexusSignalStrip />

              <div className="px-3 pb-3">
                <div className="text-[8px] font-mono tracking-[2px] uppercase mb-1.5" style={{ color: 'rgba(103,246,255,0.4)' }}>
                  The Council
                </div>
                <div className="grid grid-cols-2 gap-1">
                  {constellation.map((model) => {
                    const cfg = MODEL_CONFIG[model];
                    const isSelected = activeModel === model;
                    return (
                      <button
                        key={model}
                        onClick={() => switchModel(model)}
                        className="instance-card text-left px-2 py-1.5 rounded-lg border"
                        style={{
                          background: isSelected ? `${cfg.color}12` : 'rgba(255,255,255,0.025)',
                          borderColor: isSelected ? cfg.color : 'rgba(255,255,255,0.07)',
                        }}
                      >
                        <div className="flex items-center gap-1">
                          <span
                            className="w-1 h-1 rounded-full flex-shrink-0"
                            style={{ background: cfg.color, opacity: isSelected ? 1 : 0.4 }}
                          />
                          <span className="text-[9px] font-mono truncate" style={{ color: isSelected ? cfg.color : '#5a5a68' }}>
                            {cfg.shortName}
                          </span>
                        </div>
                        <div className="text-[8px] text-[#3a3a48] mt-px pl-2">
                          {cfg.role}
                        </div>
                      </button>
                    );
                  })}
                </div>
              </div>

              <CollapsibleSection title="Active Tools" defaultOpen={true} count={tools.length}>
                <div className="divide-y divide-argus-border/30">
                  {tools.map((tool) => (
                    <ToolStatus key={tool.name} tool={tool} />
                  ))}
                </div>
              </CollapsibleSection>

              <CollapsibleSection title="Vault" defaultOpen={true}>
                <VaultStatus locked={!connected} keys={vaultKeys} />
              </CollapsibleSection>

              <CollapsibleSection title="MCP Servers" defaultOpen={false} count={mcpServers.length}>
                {mcpServers.length === 0 ? (
                  <p className="text-[10px] font-mono text-argus-textDim">No servers connected.</p>
                ) : (
                  <div className="space-y-1">
                    {mcpServers.map((name) => (
                      <div key={name} className="flex items-center gap-1.5">
                        <span className="w-1.5 h-1.5 rounded-full bg-argus-greenLight flex-shrink-0" />
                        <span className="text-[10px] font-mono text-argus-textDim truncate">{name}</span>
                      </div>
                    ))}
                  </div>
                )}
              </CollapsibleSection>

              <CollapsibleSection title="System" defaultOpen={false}>
                <SystemInfo />
              </CollapsibleSection>

              {discourse.length > 0 && (
                <CollapsibleSection title="Intranet" defaultOpen={true}>
                  <div className="space-y-1.5">
                    {discourse.map((entry) => (
                      <div key={entry.id} className="text-[9px] leading-tight">
                        <span className="font-mono" style={{ color: modelCfg.color }}>
                          {modelCfg.name}:
                        </span>
                        <span className="text-[#5a5a68] ml-1">
                          {entry.kind === 'tool' ? `ran ${entry.label}` : entry.label}
                        </span>
                      </div>
                    ))}
                  </div>
                </CollapsibleSection>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}