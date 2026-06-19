'use client';

import dynamic from 'next/dynamic';
import { useAgentStore } from '@/hooks/useAgentState';
import { ModelId } from '@/lib/types';
import { MODEL_CONFIG } from '@/lib/models';

const ForceGraph2D = dynamic(() => import('react-force-graph-2d'), { ssr: false });

const STATUS_GLOW: Record<string, string> = {
  pending: '#c084fc',
  running: '#67f6ff',
  done:    '#39d353',
  failed:  '#ff5577',
};

export function SemanticField() {
  const memories      = useAgentStore((s) => s.memories);
  const skills        = useAgentStore((s) => s.skills);
  const scheduledTasks = useAgentStore((s) => s.scheduledTasks);
  const sendMessage   = useAgentStore((s) => s.sendMessage);

  const hasSchedule = scheduledTasks.length > 0;

  const nodes: any[] = [
    // Memory nodes — amber
    ...memories.map((m, i) => ({
      id: 'mem' + i,
      name: m.content.length > 44 ? m.content.slice(0, 44) + '…' : m.content,
      group: 1,
      val: (m.importance || 5) * 1.6,
      color: '#f5b800',
    })),
    // Skill nodes — cyan
    ...skills.map((s, i) => ({
      id: 'sk' + i,
      name: s.name,
      group: 2,
      val: ((s.useCount || 1)) * 2.5,
      color: '#67f6ff',
    })),
  ];

  const links: any[] = [
    // Sequential memory links
    ...memories.slice(0, -1).map((_, i) => ({
      source: 'mem' + i,
      target: 'mem' + (i + 1),
      color: 'rgba(245,184,0,0.12)',
    })),
  ];

  // Schedule hub + task nodes — violet cluster
  if (hasSchedule) {
    nodes.push({
      id: '__schedule__',
      name: 'MASTER SCHEDULE',
      group: 3,
      val: 12,
      color: '#c084fc',
    });

    scheduledTasks.forEach((task, i) => {
      const cfg = MODEL_CONFIG[task.agent as ModelId] ?? { name: task.agent, color: '#9a9aa8' };
      const shortDesc = task.description.length > 36
        ? task.description.slice(0, 36) + '…'
        : task.description;
      nodes.push({
        id: 'task' + i,
        name: `${cfg.name}: ${shortDesc}`,
        group: 4,
        val: 5,
        color: STATUS_GLOW[task.status] ?? '#c084fc',
      });
      links.push({
        source: '__schedule__',
        target: 'task' + i,
        color: 'rgba(192,132,252,0.18)',
      });
    });
  }

  if (nodes.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center px-4">
        <div className="text-2xl mb-2 opacity-20">✧</div>
        <div className="font-mono text-[10px] text-[#3a3a48] tracking-[1px]">
          GRAPH AWAITS DATA
        </div>
        <div className="text-[9px] text-[#3a3a48] mt-1">
          memories, skills, and scheduled tasks map here as you work
        </div>
      </div>
    );
  }

  function handleNodeClick(node: any) {
    if (node.id === '__schedule__') {
      // Clicking master schedule node does nothing — just inspecting
      return;
    }
    sendMessage(`Recall: ${node.name}`);
  }

  return (
    <div className="h-full w-full flex flex-col">
      <div className="px-3 pt-2 pb-1 flex-shrink-0 flex items-center justify-between">
        <div className="font-mono text-[9px] tracking-[1.5px] text-[#67f6ff]/50 uppercase">
          Semantic Field — click to recall
        </div>
        <div className="flex gap-3">
          <span className="flex items-center gap-1 text-[8px] font-mono text-[#3a3a48]">
            <span className="w-1.5 h-1.5 rounded-full bg-[#f5b800] inline-block" /> mem
          </span>
          <span className="flex items-center gap-1 text-[8px] font-mono text-[#3a3a48]">
            <span className="w-1.5 h-1.5 rounded-full bg-[#67f6ff] inline-block" /> skill
          </span>
          {hasSchedule && (
            <span className="flex items-center gap-1 text-[8px] font-mono text-[#3a3a48]">
              <span className="w-1.5 h-1.5 rounded-full bg-[#c084fc] inline-block" /> schedule
            </span>
          )}
        </div>
      </div>
      <div className="flex-1 min-h-0">
        <ForceGraph2D
          graphData={{ nodes, links }}
          nodeLabel="name"
          nodeRelSize={5}
          linkWidth={0.5}
          backgroundColor="transparent"
          nodeColor={(node: any) => node.color ?? '#f5b800'}
          linkColor={(link: any) => (link as any).color ?? 'rgba(103,246,255,0.12)'}
          onNodeClick={handleNodeClick}
        />
      </div>
    </div>
  );
}
