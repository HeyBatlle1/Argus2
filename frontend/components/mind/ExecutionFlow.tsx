'use client';

import { useEffect } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
} from '@xyflow/react';
import { useAgentStore } from '@/hooks/useAgentState';

export function ExecutionFlow() {
  const activeToolCalls = useAgentStore((s) => s.activeToolCalls);
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  useEffect(() => {
    const newNodes: Node[] = activeToolCalls.map((tc, idx) => ({
      id: tc.id,
      position: { x: 60 + idx * 180, y: 80 },
      data: { label: `${tc.name}\n${tc.state}` },
      style: {
        background: tc.state === 'complete' ? '#0a2218' : '#111118',
        border: `1px solid ${tc.success === false ? '#ff5577' : '#67f6ff'}`,
        color: '#e8e8f0',
        fontSize: '11px',
        fontFamily: "'JetBrains Mono', monospace",
        borderRadius: '6px',
        padding: '8px 12px',
        whiteSpace: 'pre-line' as const,
      },
    }));

    const newEdges: Edge[] = activeToolCalls.slice(1).map((_, i) => ({
      id: 'e' + i,
      source: activeToolCalls[i].id,
      target: activeToolCalls[i + 1].id,
      style: { stroke: '#67f6ff44', strokeWidth: 1.5 },
    }));

    setNodes(newNodes);
    setEdges(newEdges);
  }, [activeToolCalls, setNodes, setEdges]);

  if (activeToolCalls.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center px-4">
        <div className="text-2xl mb-2 opacity-20">⟳</div>
        <div className="font-mono text-[10px] text-[#3a3a48] tracking-[1px]">
          NO ACTIVE TOOL CALLS
        </div>
        <div className="text-[9px] text-[#3a3a48] mt-1">
          execution graph appears during tool use
        </div>
      </div>
    );
  }

  return (
    <div className="h-full w-full flex flex-col">
      <div className="px-3 pt-2 pb-1 flex-shrink-0">
        <div className="font-mono text-[9px] tracking-[1.5px] text-[#f5b800]/50 uppercase">
          Execution Flow — live tool graph
        </div>
      </div>
      <div className="flex-1 min-h-0" style={{ background: '#08080e' }}>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          fitView
          proOptions={{ hideAttribution: true }}
        >
          <Background color="#1a1a24" gap={20} size={1} />
          <Controls
            style={{ background: '#111118', border: '1px solid #2a2a38', borderRadius: '6px' }}
          />
        </ReactFlow>
      </div>
    </div>
  );
}
