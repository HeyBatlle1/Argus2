'use client';

import { useEffect, useRef, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { useAgentStore } from '@/hooks/useAgentState';
import { ModelId } from '@/lib/types';

interface Command {
  id: string;
  label: string;
  group: string;
  hint?: string;
  action: () => void;
}

interface Props {
  open: boolean;
  onClose: () => void;
  onStartMeeting: () => void;
  onToggleFocus: () => void;
  onOpenScheduler: () => void;
  onOpenHistory: () => void;
  paneCount: 1 | 2 | 3;
  onSetPaneCount: (n: 1 | 2 | 3) => void;
}

const MODEL_COMMANDS: { model: ModelId; label: string }[] = [
  { model: 'claude-haiku', label: 'Haiku — Royal' },
  { model: 'claude-opus',  label: 'Opus — Royal' },
  { model: 'grok-build',  label: 'Grok Build — Allied' },
  { model: 'grok',        label: 'Grok — Allied' },
  { model: 'gemini-flash', label: 'Gemini Flash — Allied' },
];

export function CommandPalette({
  open, onClose, onStartMeeting, onToggleFocus, onOpenScheduler, onOpenHistory,
  paneCount, onSetPaneCount,
}: Props) {
  const [query, setQuery] = useState('');
  const [activeIdx, setActiveIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const switchModel = useAgentStore((s) => s.switchModel);
  const summonBuilder = useAgentStore((s) => s.summonBuilder);
  const newConversation = useAgentStore((s) => s.newConversation);
  const setMindView = useAgentStore((s) => s.setMindView);
  const expandAllPanels = useAgentStore((s) => s.expandAllPanels);

  const commands: Command[] = [
    {
      id: 'summon-builder',
      label: 'Summon Grok Build — Primary Coder',
      group: 'Builder',
      hint: '⌘B · full tools · 12 rounds',
      action: () => { summonBuilder(); onClose(); },
    },
    ...MODEL_COMMANDS.map((m) => ({
      id: 'model-' + m.model,
      label: 'Switch to ' + m.label,
      group: 'Models',
      action: () => { switchModel(m.model); onClose(); },
    })),
    {
      id: 'meeting',
      label: 'Open Council Chamber',
      group: 'System',
      hint: 'Four-model monthly meeting',
      action: () => { onStartMeeting(); onClose(); },
    },
    {
      id: 'new-thread',
      label: 'New Conversation',
      group: 'System',
      hint: 'Fresh thread on main connection',
      action: () => { newConversation(); onClose(); },
    },
    {
      id: 'focus',
      label: 'Toggle Focus Mode',
      group: 'View',
      hint: 'Collapse side panels · slow starfield',
      action: () => { onToggleFocus(); onClose(); },
    },
    {
      id: 'expand-panels',
      label: 'Expand Eyes + Mind Panels',
      group: 'View',
      action: () => { expandAllPanels(); onClose(); },
    },
    {
      id: 'pane-1',
      label: 'Single Pane Layout',
      group: 'Layout',
      action: () => { onSetPaneCount(1); onClose(); },
    },
    {
      id: 'pane-2',
      label: 'Dual Pane Layout',
      group: 'Layout',
      hint: 'Main + allied model',
      action: () => { onSetPaneCount(2); onClose(); },
    },
    {
      id: 'pane-3',
      label: 'Triple Pane Layout',
      group: 'Layout',
      hint: 'Main + Grok + Gemini',
      action: () => { onSetPaneCount(3); onClose(); },
    },
    {
      id: 'mind-mind',
      label: 'Mind Panel → Memory',
      group: 'Mind',
      action: () => { setMindView('mind'); onClose(); },
    },
    {
      id: 'mind-graph',
      label: 'Mind Panel → Knowledge Graph',
      group: 'Mind',
      action: () => { setMindView('field'); onClose(); },
    },
    {
      id: 'mind-flow',
      label: 'Mind Panel → Execution Flow',
      group: 'Mind',
      action: () => { setMindView('flow'); onClose(); },
    },
    {
      id: 'mind-sched',
      label: 'Mind Panel → Schedule',
      group: 'Mind',
      action: () => { setMindView('schedule'); onClose(); },
    },
    {
      id: 'scheduler',
      label: 'Open Task Scheduler',
      group: 'View',
      action: () => { onOpenScheduler(); onClose(); },
    },
    {
      id: 'history',
      label: 'Open Conversation History',
      group: 'View',
      action: () => { onOpenHistory(); onClose(); },
    },
  ];

  const filtered = query.trim()
    ? commands.filter((c) =>
        c.label.toLowerCase().includes(query.toLowerCase()) ||
        c.group.toLowerCase().includes(query.toLowerCase()) ||
        (c.hint?.toLowerCase().includes(query.toLowerCase()) ?? false),
      )
    : commands;

  useEffect(() => {
    if (open) {
      setQuery('');
      setActiveIdx(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  useEffect(() => { setActiveIdx(0); }, [query]);

  function handleKey(e: React.KeyboardEvent) {
    if (e.key === 'Escape') { onClose(); return; }
    if (e.key === 'ArrowDown') { e.preventDefault(); setActiveIdx((i) => Math.min(i + 1, filtered.length - 1)); }
    if (e.key === 'ArrowUp')   { e.preventDefault(); setActiveIdx((i) => Math.max(i - 1, 0)); }
    if (e.key === 'Enter' && filtered[activeIdx]) { filtered[activeIdx].action(); }
  }

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          className="cmdk"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.12 }}
          onClick={onClose}
        >
          <motion.div
            className="cmdk-box"
            initial={{ opacity: 0, y: -12, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -12, scale: 0.97 }}
            transition={{ duration: 0.14 }}
            onClick={(e) => e.stopPropagation()}
          >
            <input
              ref={inputRef}
              className="cmdk-input"
              placeholder="Command the chamber…"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKey}
            />

            <div style={{ maxHeight: 380, overflowY: 'auto' }}>
              {!query.trim() && (
                <div className="cmdk-hint" style={{ paddingTop: 8 }}>
                  Layout: pane {paneCount} · ⌘K anytime
                </div>
              )}
              {filtered.length === 0 && (
                <div className="cmdk-hint">No commands match</div>
              )}
              {filtered.map((cmd, i) => (
                <div
                  key={cmd.id}
                  className={`cmdk-item ${i === activeIdx ? 'active' : ''}`}
                  onMouseEnter={() => setActiveIdx(i)}
                  onClick={cmd.action}
                >
                  <span className="text-[9px] text-[#3a3a48] w-14 flex-shrink-0">{cmd.group}</span>
                  <span className="flex-1">{cmd.label}</span>
                  {cmd.hint && <span className="text-[9px] text-[#3a3a48] hidden sm:block">{cmd.hint}</span>}
                </div>
              ))}
            </div>
            <div className="cmdk-hint">↑↓ navigate · Enter select · Esc dismiss</div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}