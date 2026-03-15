'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronLeft } from 'lucide-react';
import { useAgentStore } from '@/hooks/useAgentState';
import { CollapsibleSection } from '@/components/shared/CollapsibleSection';
import { EyeStateIndicator } from '@/components/shared/EyeStateIndicator';
import { ToolStatus } from './ToolStatus';
import { VaultStatus } from './VaultStatus';
import { SystemInfo } from './SystemInfo';
import { EYE_LABELS } from '@/lib/constants';
import { MODEL_CONFIG } from '@/lib/models';

export function EyesPanel() {
  const [collapsed, setCollapsed] = useState(false);
  const eyeState = useAgentStore((s) => s.eyeState);
  const activeModel = useAgentStore((s) => s.activeModel);
  const tools = useAgentStore((s) => s.tools);
  const startTime = useAgentStore((s) => s.startTime);

  const modelCfg = MODEL_CONFIG[activeModel];

  return (
    <motion.div
      animate={{ width: collapsed ? 32 : 240 }}
      transition={{ duration: 0.2, ease: 'easeInOut' }}
      className="relative h-full border-r border-argus-border flex-shrink-0 overflow-hidden"
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
              <ChevronLeft size={14} className="rotate-180" />
            </button>
            <EyeStateIndicator state={eyeState} size="sm" />
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
            className="absolute inset-0 flex flex-col overflow-y-auto"
          >
            {/* Panel header */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-argus-border flex-shrink-0">
              <span className="text-[9px] font-mono tracking-widest uppercase text-argus-amberDim">
                THE EYES
              </span>
              <button
                onClick={() => setCollapsed(true)}
                className="text-argus-textDim hover:text-argus-amber transition-colors"
              >
                <ChevronLeft size={14} />
              </button>
            </div>

            {/* Agent Status */}
            <CollapsibleSection title="Agent Status" defaultOpen={true}>
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <EyeStateIndicator state={eyeState} size="sm" showLabel={true} />
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-[9px] font-mono tracking-widest uppercase text-argus-textDim">MODEL</span>
                  <span className="text-[11px] font-mono text-argus-amber">{modelCfg.name}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-[9px] font-mono tracking-widest uppercase text-argus-textDim">TIER</span>
                  <span className="text-[11px] font-mono" style={{ color: modelCfg.color }}>
                    {modelCfg.icon} {modelCfg.tierLabel}
                  </span>
                </div>
              </div>
            </CollapsibleSection>

            {/* Active Tools */}
            <CollapsibleSection title="Active Tools" defaultOpen={true} count={tools.length}>
              <div className="divide-y divide-argus-border/30">
                {tools.map((tool) => (
                  <ToolStatus key={tool.name} tool={tool} />
                ))}
              </div>
            </CollapsibleSection>

            {/* Vault */}
            <CollapsibleSection title="Vault" defaultOpen={true}>
              <VaultStatus
                locked={false}
                keys={['openrouter_api_key', 'brave_search_api_key', 'telegram_bot_token']}
              />
            </CollapsibleSection>

            {/* MCP Servers */}
            <CollapsibleSection title="MCP Servers" defaultOpen={false} count={0}>
              <p className="text-[10px] font-mono text-argus-textDim">No servers connected.</p>
            </CollapsibleSection>

            {/* System */}
            <CollapsibleSection title="System" defaultOpen={false}>
              <SystemInfo />
            </CollapsibleSection>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}
