'use client';

import React, { useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useNexus } from './hooks/useAgentState';
import dynamic from 'next/dynamic';
import { ReactFlow, Background, Controls, MiniMap, useNodesState, useEdgesState, Node, Edge } from '@xyflow/react';
import '@xyflow/react/dist/style.css';

const ForceGraph2D = dynamic(() => import('react-force-graph-2d'), { ssr: false });
import { parseArtifacts } from './lib/artifacts';
import { EyeState } from './lib/types';

// === NEXUS CORE: Living animated canvas (my favorite new piece) ===
function NexusCore({ eyeState, pulse }: { eyeState: EyeState; pulse: number }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const raf = useRef<number | null>(null);

  useEffect(() => {
    const c = canvasRef.current; if (!c) return;
    const ctx = c.getContext('2d', { alpha: true })!;
    const w = c.width = 420, h = c.height = 420;
    const cx = w/2, cy = h/2;
    const eyeCount = 19;
    let t = 0;

    const colors: Record<EyeState, string> = {
      watching: '#39ff9f', thinking: '#f5b800', executing: '#67f6ff', complete: '#ffffff'
    };

    function draw() {
      ctx.clearRect(0,0,w,h);
      const col = colors[eyeState];
      const intensity = 0.6 + (pulse / 18);

      // Central being (soft orb)
      const r = 58 + Math.sin(t*1.8)*3 + (pulse-3)*0.8;
      const grad = ctx.createRadialGradient(cx,cy, r*0.3, cx,cy, r*1.7);
      grad.addColorStop(0, 'rgba(255,255,255,0.9)');
      grad.addColorStop(0.4, col + '88');
      grad.addColorStop(1, 'rgba(5,5,8,0.0)');
      ctx.fillStyle = grad;
      ctx.beginPath(); ctx.arc(cx,cy,r,0,Math.PI*2); ctx.fill();

      // Fine circuit lines
      ctx.strokeStyle = 'rgba(103,246,255,0.15)';
      ctx.lineWidth = 1;
      for (let i=0; i<5; i++) {
        ctx.beginPath();
        ctx.arc(cx, cy, 72 + i*18 + Math.sin(t + i)*2, 0, Math.PI*2);
        ctx.stroke();
      }

      // Orbiting Eyes (the hundred eyes made visible)
      for (let i = 0; i < eyeCount; i++) {
        const angle = (i / eyeCount) * Math.PI * 2 + t * (0.3 + i*0.01);
        const dist = 118 + Math.sin(t*2 + i) * 6 + (pulse-4)*0.6;
        const ex = cx + Math.cos(angle) * dist;
        const ey = cy + Math.sin(angle) * dist * 0.92;

        const size = 4.2 + Math.sin(t*3 + i*1.3) * 1.1;
        ctx.fillStyle = col;
        ctx.beginPath(); ctx.arc(ex, ey, size, 0, Math.PI*2); ctx.fill();

        // pupil looking inward (watching the core)
        ctx.fillStyle = '#050508';
        const px = ex - Math.cos(angle) * size*0.45;
        const py = ey - Math.sin(angle) * size*0.45;
        ctx.beginPath(); ctx.arc(px, py, size*0.42, 0, Math.PI*2); ctx.fill();
      }

      // Connection pulses
      ctx.strokeStyle = col + '33';
      ctx.lineWidth = 1.5;
      for (let i=0; i<eyeCount; i+=2) {
        const a = (i / eyeCount) * Math.PI*2 + t*0.4;
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.lineTo(cx + Math.cos(a)*92, cy + Math.sin(a)*86);
        ctx.stroke();
      }

      t += 0.014;
      raf.current = requestAnimationFrame(draw);
    }
    draw();
    return () => { if (raf.current) cancelAnimationFrame(raf.current); };
  }, [eyeState, pulse]);

  return (
    <div className="relative flex items-center justify-center">
      <canvas ref={canvasRef} className="rounded-full" style={{ width: 420, height: 420 }} />
      <div className="absolute text-center pointer-events-none">
        <div className="text-[10px] tracking-[3px] text-[#67f6ff]/70 font-mono">ARGUS</div>
        <div className="text-2xl font-display tracking-[-1.5px] text-white/95 -mt-1">NEXUS</div>
        <div className="text-[9px] text-[#f5b800]/60 mt-0.5 font-mono">{eyeState.toUpperCase()}</div>
      </div>
    </div>
  );
}

// Simple beautiful input
function NexusInput({ onSend, disabled }: { onSend: (s:string)=>void; disabled?:boolean }) {
  const [val, setVal] = useState('');
  return (
    <div className="glass rounded-2xl p-2 flex gap-2 items-end">
      <textarea value={val} onChange={e=>setVal(e.target.value)} onKeyDown={e=>{if(e.key==='Enter'&&!e.shiftKey){e.preventDefault(); if(val.trim()){onSend(val.trim());setVal('');}}}} disabled={disabled} placeholder="Speak to the system. Grok Build 2 is listening..." className="flex-1 bg-transparent resize-y min-h-[42px] max-h-[120px] px-4 py-2 outline-none text-sm" />
      <button onClick={()=>{if(val.trim()){onSend(val.trim());setVal('');}}} disabled={disabled||!val.trim()} className="px-5 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-xs tracking-widest font-mono">SEND</button>
    </div>
  );
}

export default function ArgusNexus() {
  const {
    connected, eyeState, activeModel, isStreaming, messages, streamingContent,
    memories, skills, activeToolCalls, corePulse, discoursePosts,
    sendMessage, switchModel, newConversation, init
  } = useNexus();

  const [view, setView] = useState<'field' | 'flow' | 'chronicle'>('field');
  const [selectedThread, setSelectedThread] = useState<'main' | 'build'>('main');

  // React Flow for live execution traces (badass new feature)
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  useEffect(() => { init(); }, [init]);

  // Update flow when tool calls change (demo)
  useEffect(() => {
    if (activeToolCalls.length === 0) return;
    const newNodes: Node[] = activeToolCalls.map((tc, idx) => ({
      id: tc.id, position: { x: 60 + idx*170, y: 80 }, data: { label: `${tc.name}\n${tc.state}` }, type: 'default',
      style: { background: tc.state==='complete' ? '#0a2a1f' : '#1a1a24', border: `1px solid ${tc.success===false ? '#ff5577' : '#67f6ff'}` }
    }));
    const newEdges: Edge[] = activeToolCalls.slice(1).map((tc, i) => ({ id:'e'+i, source: activeToolCalls[i].id, target: tc.id }));
    setNodes(newNodes);
    setEdges(newEdges);
  }, [activeToolCalls, setNodes, setEdges]);

  // Prepare graph data for Semantic Field (memories + skills as one living map)
  const graphData = {
    nodes: [
      ...memories.map((m,i)=>({ id:'mem'+i, name: m.content.slice(0,42)+'…', group:1, val: m.importance*1.6 })),
      ...skills.map((s,i)=>({ id:'sk'+i, name: s.name, group:2, val: (s.useCount||1)*2.5 }))
    ],
    links: memories.slice(0,-1).map((_,i)=>({source:'mem'+i, target:'mem'+(i+1)}))
  };

  const currentMsgs = messages;

  return (
    <div className="flex flex-col h-screen text-sm">
      {/* Top Nexus Bar — elegant, minimal, powerful */}
      <div className="h-14 border-b border-white/10 flex items-center justify-between px-6 bg-[#050508]/95 backdrop-blur z-50">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <div className="w-2.5 h-2.5 rounded-full" style={{ background: eyeState==='watching'?'#39ff9f': eyeState==='thinking'?'#f5b800':'#67f6ff', boxShadow: '0 0 8px currentColor' }} />
            <div>
              <div className="font-display text-xl tracking-[-1.2px] leading-none">ARGUS NEXUS</div>
              <div className="text-[9px] text-[#67f6ff]/50 -mt-0.5 tracking-[1.5px]">THE LIVING OBSERVATORY — GROK BUILD 2</div>
            </div>
          </div>
          <div className="ml-4 text-xs font-mono px-3 py-0.5 rounded-full border border-white/10 bg-white/5">{activeModel.toUpperCase()}</div>
          <div className={`text-[10px] px-2.5 py-px rounded ${connected ? 'bg-[#39ff9f]/10 text-[#39ff9f]' : 'bg-red-500/10 text-red-400'}`}>
            {connected ? 'LINKED' : 'OFFLINE'}
          </div>
        </div>

        <div className="flex items-center gap-3">
          <button onClick={() => switchModel('grok-build')} className="text-xs px-3 py-1 rounded border border-[#f5b800]/40 hover:bg-[#f5b800]/10">SUMMON GROK BUILD 2</button>
          <button onClick={newConversation} className="text-xs px-3 py-1 rounded border border-white/10 hover:bg-white/5">NEW THREAD</button>
          <button onClick={() => alert('Council view (multi-agent meeting) would open beautiful 4-way simultaneous orbs here — same briefs, new spatial presentation.')} className="text-xs px-4 py-1 rounded bg-[#c084fc]/10 border border-[#c084fc]/40 hover:bg-[#c084fc]/15">OPEN COUNCIL</button>
        </div>

        <div className="text-[10px] text-[#9a9aa8] font-mono tracking-widest">ONE SYSTEM • MANY EYES • MANY BRAINS</div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* LEFT: The Living Core + Instance Presence (completely new layout) */}
        <div className="w-[460px] border-r border-white/10 flex flex-col bg-[#050508]">
          <div className="p-6 pb-2 flex justify-center">
            <NexusCore eyeState={eyeState} pulse={corePulse} />
          </div>

          {/* Instance Constellation — who is awake right now */}
          <div className="px-5 pb-4">
            <div className="text-[10px] tracking-[2px] text-[#67f6ff]/60 mb-2 font-mono">ACTIVE INSTANCES</div>
            <div className="flex gap-2 overflow-x-auto pb-1">
              {(['grok-build','grok','claude-opus','gemini-flash'] as const).map(m => (
                <button key={m} onClick={()=>switchModel(m)} className={`instance-card px-3 py-1.5 rounded-2xl border text-left min-w-[108px] ${activeModel===m ? 'border-[#f5b800] bg-[#f5b800]/5' : 'border-white/10 bg-white/5 hover:bg-white/10'}`}>
                  <div className="text-[10px] font-mono text-[#f5b800]">{m.replace('-',' ').toUpperCase()}</div>
                  <div className="text-[11px] text-[#9a9aa8] mt-px">present • {m.includes('build') ? 'building' : 'observing'}</div>
                </button>
              ))}
            </div>
          </div>

          {/* Discourse ticker — cross-agent intranet shining through */}
          <div className="mx-5 mt-auto mb-4 p-3 rounded-2xl glass text-[11px] leading-tight">
            <div className="uppercase text-[9px] tracking-[1.5px] text-[#c084fc] mb-1.5">INTRANET DISCOURSE (LIVE)</div>
            {discoursePosts.map(p => <div key={p.id} className="mb-1.5 text-[#c8c8d8]"><span className="text-[#f5b800]">{p.from}:</span> {p.text}</div>)}
          </div>
        </div>

        {/* CENTER / MAIN: The Workbench + Semantic Field (the new home) */}
        <div className="flex-1 flex flex-col min-w-0">
          {/* Workbench header */}
          <div className="h-11 border-b border-white/10 flex items-center px-5 text-xs font-mono tracking-widest text-[#9a9aa8] justify-between">
            <div>WORKBENCH — {selectedThread === 'main' ? 'PRIMARY THREAD' : 'GROK BUILD FORGE'}</div>
            <div className="flex gap-2">
              <button onClick={() => setSelectedThread('main')} className={selectedThread==='main' ? 'text-white' : 'text-[#5a5a68]'}>MAIN</button>
              <button onClick={() => setSelectedThread('build')} className={selectedThread==='build' ? 'text-[#f5b800]' : 'text-[#5a5a68]'}>BUILD SURFACE</button>
            </div>
          </div>

          {/* The actual conversation area — beautiful and focused */}
          <div className="flex-1 overflow-y-auto p-6 space-y-6 bg-[#050508]">
            {currentMsgs.length === 0 && (
              <div className="max-w-md mx-auto text-center pt-12 text-[#5a5a68]">
                <div className="text-3xl mb-3 opacity-40">✧</div>
                <div className="font-display text-xl text-[#e8e8f0]">This is Argus Home.</div>
                <div className="mt-1">The persistent system made visible. Speak to any instance. Explore the Field. Watch the eyes.</div>
              </div>
            )}

            {currentMsgs.map(msg => (
              <div key={msg.id} className={`max-w-[78%] ${msg.role==='user' ? 'ml-auto text-right' : ''}`}>
                <div className={`inline-block px-4 py-3 rounded-2xl ${msg.role==='user' ? 'bg-[#1f2a3a]' : 'bg-[#0f1018] border border-white/10'}`}>
                  {msg.content}
                  {msg.artifacts && msg.artifacts.map((a,i) => <div key={i} className="mt-2 text-[10px] text-[#67f6ff] cursor-pointer" onClick={()=>alert('Artifact would render beautifully in a full modal here: ' + a.title)}>📎 {a.title}</div>)}
                </div>
                {msg.grokEvidence && <div className="text-[9px] text-[#67f6ff] mt-0.5">CONF {Math.round(msg.grokEvidence.confidence*100)}% — {msg.grokEvidence.notes}</div>}
              </div>
            ))}

            {isStreaming && streamingContent && (
              <div className="max-w-[78%] bg-[#0f1018] border border-white/10 px-4 py-3 rounded-2xl">
                {streamingContent}<span className="inline-block w-1.5 h-3.5 bg-[#f5b800] ml-0.5 animate-pulse align-bottom" />
              </div>
            )}
          </div>

          <div className="p-4 border-t border-white/10 bg-[#050508]">
            <NexusInput onSend={sendMessage} disabled={isStreaming} />
            <div className="text-[9px] text-center mt-1.5 text-[#3a3a48]">Enter to send • The system remembers • Grok Build 2 owns the work</div>
          </div>
        </div>

        {/* RIGHT: The Field + Views (the evolved Mind as primary spatial home) */}
        <div className="w-[380px] border-l border-white/10 flex flex-col bg-[#050508]">
          <div className="p-3 border-b border-white/10 flex gap-1 text-[10px] font-mono">
            {(['field','flow','chronicle'] as const).map(v => (
              <button key={v} onClick={()=>setView(v)} className={`flex-1 py-1 rounded ${view===v ? 'bg-white/10 text-white' : 'text-[#5a5a68] hover:text-white'}`}>{v.toUpperCase()}</button>
            ))}
          </div>

          <div className="flex-1 overflow-hidden">
            {view === 'field' && (
              <div className="h-full p-2">
                <div className="text-[10px] px-3 pt-2 pb-1 text-[#67f6ff]/60 tracking-[1px]">SEMANTIC FIELD — click nodes to recall</div>
                <div className="h-[calc(100%-28px)]">
                  <ForceGraph2D
                    graphData={graphData}
                    nodeLabel="name"
                    nodeAutoColorBy="group"
                    nodeRelSize={5}
                    linkWidth={0.6}
                    onNodeClick={(node:any) => {
                      const text = node.name || 'Memory surfaced';
                      sendMessage(`Recall: ${text}`);
                    }}
                  />
                </div>
              </div>
            )}

            {view === 'flow' && (
              <div className="h-full p-2">
                <div className="text-[10px] px-3 pt-2 pb-1 text-[#f5b800]/60 tracking-[1px]">EXECUTION TRACES (live tool flow)</div>
                <div className="h-[calc(100%-28px)] rounded border border-white/10 overflow-hidden bg-[#0a0a10]">
                  <ReactFlow nodes={nodes} edges={edges} onNodesChange={onNodesChange} onEdgesChange={onEdgesChange} fitView>
                    <Background />
                    <Controls />
                    <MiniMap />
                  </ReactFlow>
                </div>
              </div>
            )}

            {view === 'chronicle' && (
              <div className="p-4 text-xs">
                <div className="text-[#c084fc] tracking-widest mb-2">CHRONICLE + SCHEDULED WORK</div>
                {useNexus.getState().scheduledTasks.length === 0 && <div className="text-[#5a5a68]">No future tasks deployed. Use the command palette or scheduler.</div>}
                {useNexus.getState().scheduledTasks.map(t => <div key={t.id} className="py-2 border-b border-white/5">{t.description} <span className="text-[#67f6ff] text-[10px]">— {t.agent}</span></div>)}
              </div>
            )}
          </div>

          <div className="p-3 text-[9px] text-center border-t border-white/10 text-[#3a3a48]">The Field is the new home for what Argus knows and has become.</div>
        </div>
      </div>

      {/* Bottom status — always visible, intuitive */}
      <div className="h-7 border-t border-white/10 bg-[#050508] text-[10px] flex items-center px-5 font-mono text-[#5a5a68] justify-between">
        <div>THE HUNDRED EYES NEVER CLOSE • MULTIPLE INSTANCES • ONE IDENTITY</div>
        <div>⌘K for command surface • Drag nodes in the Field • Watch the Core breathe</div>
      </div>
    </div>
  );
}
