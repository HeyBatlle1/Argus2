'use client';
import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Send, Clock, Zap, CheckCircle2 } from 'lucide-react';
import { useAgentStore } from '../../hooks/useAgentState';
import { MODEL_CONFIG, MODELS_IN_ORDER } from '../../lib/models';
import { ModelId } from '../../lib/types';

export function TaskScheduler({ onClose }: { onClose: () => void }) {
  const tasks = useAgentStore(s => s.scheduledTasks);
  const schedule = useAgentStore(s => s.scheduleTask);
  const [agent, setAgent] = useState<ModelId>('grok-build');
  const [when, setWhen] = useState<'now' | 'later'>('now');
  const [at, setAt] = useState('');
  const [desc, setDesc] = useState('');
  const [done, setDone] = useState(false);

  const deploy = () => {
    if (!desc.trim()) return;
    const runAt = when === 'later' && at ? new Date(at).toISOString() : null;
    schedule(agent, runAt, desc.trim());
    setDone(true);
    setTimeout(() => { setDone(false); setDesc(''); setWhen('now'); setAt(''); onClose(); }, 1100);
  };

  return (
    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="fixed inset-0 z-50 flex items-center justify-center" style={{ background: 'rgba(5,5,10,0.8)' }} onClick={onClose}>
      <motion.div initial={{ y: 12, scale: .985, opacity: 0 }} animate={{ y: 0, scale: 1, opacity: 1 }} onClick={e => e.stopPropagation()} className="w-full max-w-4xl mx-3 rounded-xl overflow-hidden flex" style={{ background: '#0d0d14', border: '1px solid #1e1e32', maxHeight: '82vh' }}>
        <div className="w-[400px] border-r p-5 flex flex-col gap-4" style={{ borderColor: '#32325a' }}>
          <div className="flex justify-between"><div className="text-[#f5b800] font-mono tracking-[0.16em] text-xs flex items-center gap-1"><Zap size={13} />DEPLOY TASK</div><button onClick={onClose}><X /></button></div>

          <div>
            <div className="text-[9px] text-[#b8b5ac] mb-1 font-mono tracking-widest">AGENT</div>
            <div className="grid grid-cols-2 gap-1">
              {MODELS_IN_ORDER.map(id => { const m = MODEL_CONFIG[id]; const act = agent === id; return <button key={id} onClick={() => setAgent(id)} className="text-left px-2 py-1.5 rounded text-xs flex gap-1.5 font-mono" style={{ background: act ? (m.tier === 'royal' ? 'rgba(201,168,76,0.12)' : 'rgba(57,211,83,0.1)') : 'rgba(255,255,255,0.03)', border: act ? '1px solid #f5b800' : '1px solid #1e1e32' }}>{m.icon} {m.name}</button>; })}
            </div>
          </div>

          <div>
            <div className="text-[9px] text-[#b8b5ac] mb-1 font-mono tracking-widest">TIMING</div>
            <div className="flex gap-1 mb-2">
              <button onClick={() => setWhen('now')} className="flex-1 py-1 rounded text-xs font-mono" style={{ background: when === 'now' ? 'rgba(201,168,76,0.12)' : 'rgba(255,255,255,0.03)', border: when === 'now' ? '1px solid #f5b800' : '1px solid #2a2a42' }}><Zap size={10} className="inline mr-1" />NOW</button>
              <button onClick={() => setWhen('later')} className="flex-1 py-1 rounded text-xs font-mono" style={{ background: when === 'later' ? 'rgba(201,168,76,0.12)' : 'rgba(255,255,255,0.03)', border: when === 'later' ? '1px solid #f5b800' : '1px solid #2a2a42' }}><Clock size={10} className="inline mr-1" />SCHEDULE</button>
            </div>
            {when === 'later' && <input type="datetime-local" value={at} onChange={e => setAt(e.target.value)} className="w-full bg-[#11111a] border border-[#2a2a42] rounded px-2 py-1 text-xs" />}
          </div>

          <div>
            <div className="text-[9px] text-[#b8b5ac] mb-1 font-mono tracking-widest">TASK</div>
            <textarea value={desc} onChange={e => setDesc(e.target.value)} rows={4} placeholder="What should this agent do?" className="w-full bg-[#11111a] border border-[#2a2a42] rounded p-2 text-sm font-mono resize-y" />
          </div>

          <button onClick={deploy} disabled={!desc.trim() || done} className="py-2 rounded font-mono text-xs tracking-widest uppercase mt-auto" style={{ background: done ? '#39d35322' : desc.trim() ? '#f5b80022' : '#ffffff08', border: done ? '1px solid #39d353' : desc.trim() ? '1px solid #f5b800' : '1px solid #2a2a42', color: done ? '#39d353' : desc.trim() ? '#f5b800' : '#5a5a7a' }}>{done ? <><CheckCircle2 size={13} className="inline mr-1" />DEPLOYED</> : <><Send size={13} className="inline mr-1" />DEPLOY</>}</button>
        </div>

        <div className="flex-1 p-5 overflow-auto">
          <div className="font-mono text-xs tracking-widest text-[#b8b5ac] mb-3">OPERATIONS QUEUE</div>
          {tasks.length === 0 && <div className="text-[#3a3a5a] text-xs font-mono mt-12 text-center">No tasks scheduled. Deploy one on the left.</div>}
          {tasks.map(t => {
            const m = MODEL_CONFIG[t.agent] || { icon: '◉', name: t.agent };
            return <div key={t.id} className="mb-2 p-3 rounded border" style={{ borderColor: '#1e1e32', background: 'rgba(255,255,255,0.015)' }}>
              <div className="flex justify-between text-xs"><span>{m.icon} {m.name}</span><span className="text-[#3a3a5a]">{t.status}</span></div>
              <div className="text-sm mt-1">{t.description}</div>
              <div className="text-[9px] text-[#5a5a7a] mt-1 font-mono">{t.runAt ? new Date(t.runAt).toLocaleString() : 'IMMEDIATE'}</div>
            </div>;
          })}
        </div>
      </motion.div>
    </motion.div>
  );
}
