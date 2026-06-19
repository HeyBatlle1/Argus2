'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Clock, Zap, Send, CheckCircle2, XCircle, Loader2, AlertCircle } from 'lucide-react';
import { useAgentStore } from '@/hooks/useAgentState';
import { MODEL_CONFIG, MODELS_IN_ORDER } from '@/lib/models';
import { ModelId, ScheduledTask } from '@/lib/types';

interface Props {
  onClose: () => void;
}

export function TaskScheduler({ onClose }: Props) {
  const scheduledTasks = useAgentStore((s) => s.scheduledTasks);
  const scheduleTask = useAgentStore((s) => s.scheduleTask);

  const [selectedAgent, setSelectedAgent] = useState<ModelId>('claude-haiku');
  const [timing, setTiming] = useState<'now' | 'later'>('now');
  const [scheduledAt, setScheduledAt] = useState('');
  const [description, setDescription] = useState('');
  const [submitted, setSubmitted] = useState(false);

  function handleDeploy() {
    if (!description.trim()) return;
    const runAt = timing === 'later' && scheduledAt ? new Date(scheduledAt).toISOString() : null;
    scheduleTask(selectedAgent, runAt, description.trim());
    setSubmitted(true);
    setTimeout(() => {
      setSubmitted(false);
      setDescription('');
      setTiming('now');
      setScheduledAt('');
    }, 2000);
  }

  // Build a local datetime string for min attribute (now)
  const nowLocal = new Date(Date.now() - new Date().getTimezoneOffset() * 60000)
    .toISOString()
    .slice(0, 16);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.18 }}
      className="fixed inset-0 z-50 flex items-center justify-center"
      style={{ background: 'rgba(5,5,10,0.85)', backdropFilter: 'blur(6px)' }}
      onClick={onClose}
    >
      <motion.div
        initial={{ opacity: 0, y: 16, scale: 0.97 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: 16, scale: 0.97 }}
        transition={{ duration: 0.2, ease: 'easeOut' }}
        className="relative flex w-full max-w-4xl mx-4 rounded-xl overflow-hidden shadow-2xl"
        style={{
          background: '#0d0d14',
          border: '1px solid #1e1e32',
          maxHeight: '85vh',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Amber glow top */}
        <div
          className="absolute top-0 left-0 right-0 h-px"
          style={{ background: 'linear-gradient(90deg, transparent, rgba(201,168,76,0.5), transparent)' }}
        />

        {/* ─── LEFT PANE: Deploy Form ─────────────────────────── */}
        <div className="flex flex-col w-[420px] shrink-0 border-r border-argus-border overflow-y-auto">
          {/* Header */}
          <div className="flex items-center justify-between px-5 pt-5 pb-4 border-b border-argus-border">
            <div>
              <div className="flex items-center gap-2">
                <Zap size={13} className="text-argus-amber" />
                <span className="font-mono text-[11px] tracking-[0.2em] uppercase text-argus-amber">
                  Deploy Task
                </span>
              </div>
              <p className="text-[10px] font-mono text-argus-textDim mt-0.5">
                Dispatch an agent at any time
              </p>
            </div>
            <button
              onClick={onClose}
              className="text-argus-textDim hover:text-argus-text transition-colors"
            >
              <X size={15} />
            </button>
          </div>

          <div className="flex flex-col gap-5 p-5">
            {/* Agent picker */}
            <div>
              <label className="block text-[9px] font-mono tracking-widest uppercase text-argus-textDim mb-2">
                Agent
              </label>
              <div className="grid grid-cols-2 gap-1.5">
                {MODELS_IN_ORDER.map((id) => {
                  const m = MODEL_CONFIG[id];
                  const active = selectedAgent === id;
                  return (
                    <button
                      key={id}
                      onClick={() => setSelectedAgent(id)}
                      className="flex items-center gap-2 px-3 py-2 rounded text-left transition-all"
                      style={{
                        background: active
                          ? m.tier === 'royal'
                            ? 'rgba(201,168,76,0.1)'
                            : 'rgba(57,211,83,0.08)'
                          : 'rgba(255,255,255,0.03)',
                        border: active
                          ? m.tier === 'royal'
                            ? '1px solid rgba(201,168,76,0.4)'
                            : '1px solid rgba(57,211,83,0.35)'
                          : '1px solid #1e1e32',
                      }}
                    >
                      <span className="text-xs">{m.icon}</span>
                      <span
                        className="text-[11px] font-mono truncate"
                        style={{
                          color: active
                            ? m.tier === 'royal' ? '#c9a84c' : '#39d353'
                            : '#7a7a9a',
                        }}
                      >
                        {m.name}
                      </span>
                    </button>
                  );
                })}
              </div>
            </div>

            {/* Timing */}
            <div>
              <label className="block text-[9px] font-mono tracking-widest uppercase text-argus-textDim mb-2">
                Timing
              </label>
              <div className="flex gap-2 mb-3">
                <TimingTab
                  label="Immediately"
                  icon={<Zap size={10} />}
                  active={timing === 'now'}
                  onClick={() => setTiming('now')}
                />
                <TimingTab
                  label="Scheduled"
                  icon={<Clock size={10} />}
                  active={timing === 'later'}
                  onClick={() => setTiming('later')}
                />
              </div>
              <AnimatePresence>
                {timing === 'later' && (
                  <motion.div
                    initial={{ opacity: 0, height: 0 }}
                    animate={{ opacity: 1, height: 'auto' }}
                    exit={{ opacity: 0, height: 0 }}
                    transition={{ duration: 0.15 }}
                    className="overflow-hidden"
                  >
                    <input
                      type="datetime-local"
                      min={nowLocal}
                      value={scheduledAt}
                      onChange={(e) => setScheduledAt(e.target.value)}
                      className="w-full px-3 py-2 rounded text-[11px] font-mono transition-colors outline-none"
                      style={{
                        background: 'rgba(255,255,255,0.04)',
                        border: '1px solid #2a2a42',
                        color: '#c8c8d8',
                        colorScheme: 'dark',
                      }}
                    />
                  </motion.div>
                )}
              </AnimatePresence>
            </div>

            {/* Task description */}
            <div>
              <label className="block text-[9px] font-mono tracking-widest uppercase text-argus-textDim mb-2">
                Task
              </label>
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Describe what this agent should do..."
                rows={5}
                className="w-full px-3 py-2.5 rounded text-[12px] font-mono resize-none outline-none transition-colors placeholder:text-argus-textDim"
                style={{
                  background: 'rgba(255,255,255,0.03)',
                  border: '1px solid #2a2a42',
                  color: '#c8c8d8',
                }}
                onFocus={(e) => (e.currentTarget.style.borderColor = 'rgba(201,168,76,0.4)')}
                onBlur={(e) => (e.currentTarget.style.borderColor = '#2a2a42')}
              />
            </div>

            {/* Deploy button */}
            <button
              onClick={handleDeploy}
              disabled={!description.trim() || submitted}
              className="flex items-center justify-center gap-2 w-full py-2.5 rounded font-mono text-[11px] tracking-[0.15em] uppercase transition-all"
              style={{
                background: submitted
                  ? 'rgba(57,211,83,0.12)'
                  : description.trim()
                  ? 'rgba(201,168,76,0.12)'
                  : 'rgba(255,255,255,0.03)',
                border: submitted
                  ? '1px solid rgba(57,211,83,0.4)'
                  : description.trim()
                  ? '1px solid rgba(201,168,76,0.4)'
                  : '1px solid #2a2a42',
                color: submitted ? '#39d353' : description.trim() ? '#c9a84c' : '#3a3a5a',
                cursor: description.trim() && !submitted ? 'pointer' : 'default',
              }}
            >
              {submitted ? (
                <>
                  <CheckCircle2 size={12} />
                  Deployed
                </>
              ) : (
                <>
                  <Send size={12} />
                  Deploy
                  {timing === 'later' && scheduledAt
                    ? ` · ${formatRelative(scheduledAt)}`
                    : timing === 'now'
                    ? ' · Now'
                    : ''}
                </>
              )}
            </button>
          </div>
        </div>

        {/* ─── RIGHT PANE: Operations Queue ───────────────────── */}
        <div className="flex flex-col flex-1 min-w-0 overflow-hidden">
          {/* Header */}
          <div className="px-5 pt-5 pb-4 border-b border-argus-border">
            <div className="flex items-center gap-2">
              <Clock size={13} className="text-argus-textDim" />
              <span className="font-mono text-[11px] tracking-[0.2em] uppercase text-argus-textDim">
                Operations Queue
              </span>
              {scheduledTasks.length > 0 && (
                <span
                  className="text-[8px] font-mono px-1.5 py-0.5 rounded ml-auto"
                  style={{ background: 'rgba(201,168,76,0.12)', color: '#c9a84c' }}
                >
                  {scheduledTasks.length}
                </span>
              )}
            </div>
          </div>

          <div className="flex-1 overflow-y-auto p-5">
            {scheduledTasks.length === 0 ? (
              <EmptyQueue />
            ) : (
              <div className="flex flex-col gap-2">
                {scheduledTasks.map((task) => (
                  <TaskCard key={task.id} task={task} />
                ))}
              </div>
            )}
          </div>
        </div>
      </motion.div>
    </motion.div>
  );
}

function TimingTab({
  label,
  icon,
  active,
  onClick,
}: {
  label: string;
  icon: React.ReactNode;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="flex items-center gap-1.5 px-3 py-1.5 rounded font-mono text-[10px] tracking-wider uppercase transition-all flex-1 justify-center"
      style={{
        background: active ? 'rgba(201,168,76,0.1)' : 'rgba(255,255,255,0.03)',
        border: active ? '1px solid rgba(201,168,76,0.4)' : '1px solid #2a2a42',
        color: active ? '#c9a84c' : '#5a5a7a',
      }}
    >
      {icon}
      {label}
    </button>
  );
}

function TaskCard({ task }: { task: ScheduledTask }) {
  const m = MODEL_CONFIG[task.agent as ModelId] ?? { icon: '?', name: task.agent, tier: 'allied' };

  const statusConfig = {
    pending: { icon: <Clock size={11} />, color: '#7a7a9a', label: 'Pending' },
    running: { icon: <Loader2 size={11} className="animate-spin" />, color: '#c9a84c', label: 'Running' },
    done: { icon: <CheckCircle2 size={11} />, color: '#39d353', label: 'Done' },
    failed: { icon: <XCircle size={11} />, color: '#ef4444', label: 'Failed' },
  }[task.status] ?? { icon: <AlertCircle size={11} />, color: '#7a7a9a', label: task.status };

  return (
    <motion.div
      initial={{ opacity: 0, x: 12 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.2 }}
      className="rounded-lg p-3"
      style={{
        background: 'rgba(255,255,255,0.02)',
        border: '1px solid #1e1e32',
      }}
    >
      {/* Top row */}
      <div className="flex items-center gap-2 mb-1.5">
        <span className="text-sm">{m.icon}</span>
        <span className="text-[11px] font-mono" style={{ color: m.tier === 'royal' ? '#c9a84c' : '#39d353' }}>
          {m.name}
        </span>
        <div className="flex items-center gap-1 ml-auto" style={{ color: statusConfig.color }}>
          {statusConfig.icon}
          <span className="text-[9px] font-mono tracking-wider">{statusConfig.label}</span>
        </div>
      </div>

      {/* Description */}
      <p className="text-[11px] font-mono text-argus-text leading-relaxed line-clamp-2 mb-1.5">
        {task.description}
      </p>

      {/* Timing row */}
      <div className="flex items-center gap-3">
        {task.runAt ? (
          <span className="text-[9px] font-mono text-argus-textDim flex items-center gap-1">
            <Clock size={8} />
            {formatDatetime(task.runAt)}
          </span>
        ) : (
          <span className="text-[9px] font-mono text-argus-textDim flex items-center gap-1">
            <Zap size={8} />
            Immediately
          </span>
        )}
        <span className="text-[9px] font-mono" style={{ color: '#2a2a42' }}>
          #{task.id.slice(0, 8)}
        </span>
      </div>
    </motion.div>
  );
}

function EmptyQueue() {
  return (
    <div className="flex flex-col items-center justify-center h-48 text-center">
      <div
        className="w-10 h-10 rounded-full flex items-center justify-center mb-3"
        style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid #1e1e32' }}
      >
        <Clock size={16} style={{ color: '#2a2a42' }} />
      </div>
      <span className="text-[10px] font-mono text-argus-textDim tracking-wider">No tasks queued</span>
      <span className="text-[9px] font-mono mt-1" style={{ color: '#2a2a42' }}>
        Deploy a task to see it here
      </span>
    </div>
  );
}

function formatRelative(isoOrLocal: string): string {
  try {
    const d = new Date(isoOrLocal);
    const diff = d.getTime() - Date.now();
    if (diff < 0) return 'Now';
    const h = Math.floor(diff / 3600000);
    const m = Math.floor((diff % 3600000) / 60000);
    if (h > 0) return `in ${h}h ${m}m`;
    return `in ${m}m`;
  } catch {
    return '';
  }
}

function formatDatetime(iso: string): string {
  try {
    return new Date(iso).toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return iso;
  }
}
