'use client';

import React, { useState, useEffect } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { Header } from './components/header/Header';
import { EyesPanel } from './components/eyes/EyesPanel';
import { ConversationPanel } from './components/conversation/ConversationPanel';
import { ConversationDrawer } from './components/conversation/ConversationDrawer';
import { MindPanel } from './components/mind/MindPanel';
import { ArtifactPanel } from './components/artifacts/ArtifactPanel';
import { ChatPane } from './components/panes/ChatPane';
import { TaskScheduler } from './components/scheduler/TaskScheduler';
import { CommandPalette } from './components/shared/CommandPalette';
import { Watchtower } from './components/watchtower/Watchtower';
import { Artifact, ModelId } from './lib/types';
import { MEETING_BRIEF_PANE1, MEETING_BRIEFS } from './lib/constants';
import { useAgentStore } from './hooks/useAgentState';

export default function GrokArgusFrontend() {
  const [paneCount, setPaneCount] = useState<1 | 2 | 3>(1);
  const [meetingMode, setMeetingMode] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const [schedulerOpen, setSchedulerOpen] = useState(false);
  const [artifactState, setArtifactState] = useState<{ artifacts: Artifact[]; index: number } | null>(null);

  const [pane2Model] = useState<ModelId>('grok-build');
  const [pane3Model] = useState<ModelId>('gemini-flash');

  const commandPaletteOpen = useAgentStore((s) => s.commandPaletteOpen);
  const toggleCommandPalette = useAgentStore((s) => s.toggleCommandPalette);
  const focusMode = useAgentStore((s) => s.focusMode);
  const toggleFocus = useAgentStore((s) => s.toggleFocus);
  const initConnection = useAgentStore((s) => s.initConnection);

  // Boot connection (dev mock or real WS)
  useEffect(() => { initConnection(); }, [initConnection]);

  // Keyboard: ⌘K / Ctrl+K for command palette, f for focus, m for meeting, etc.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        toggleCommandPalette();
      }
      if (e.key.toLowerCase() === 'f' && !['INPUT', 'TEXTAREA'].includes((e.target as HTMLElement)?.tagName)) {
        toggleFocus();
      }
      if (e.key.toLowerCase() === 'm' && !meetingMode) {
        startMeeting();
      }
      if (e.key === 'Escape') {
        if (commandPaletteOpen) toggleCommandPalette(false);
        if (schedulerOpen) setSchedulerOpen(false);
        if (historyOpen) setHistoryOpen(false);
        if (artifactState) setArtifactState(null);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [commandPaletteOpen, meetingMode, schedulerOpen, historyOpen, artifactState, toggleCommandPalette, toggleFocus]);

  function openArtifact(arts: Artifact[], idx: number) { setArtifactState({ artifacts: arts, index: idx }); }
  function closeArtifact() { setArtifactState(null); }

  function handleSetPaneCount(n: 1 | 2 | 3) {
    setPaneCount(n);
    setMeetingMode(false);
    if (n !== 1) setArtifactState(null);
  }

  function startMeeting() {
    setMeetingMode(true);
    setArtifactState(null);
  }
  function endMeeting() {
    setMeetingMode(false);
    setPaneCount(1);
  }

  return (
    <div className="flex flex-col h-screen overflow-hidden" style={{ background: '#0a0a0f' }}>
      <Header
        onToggleHistory={() => setHistoryOpen(v => !v)}
        onToggleScheduler={() => setSchedulerOpen(v => !v)}
        schedulerOpen={schedulerOpen}
        paneCount={meetingMode ? 1 : paneCount}
        onSetPaneCount={handleSetPaneCount}
        meetingMode={meetingMode}
        onStartMeeting={startMeeting}
        focusMode={focusMode}
        onToggleFocus={toggleFocus}
      />

      <ConversationDrawer open={historyOpen} onClose={() => setHistoryOpen(false)} />

      <AnimatePresence>
        {schedulerOpen && <TaskScheduler onClose={() => setSchedulerOpen(false)} />}
      </AnimatePresence>

      <CommandPalette open={commandPaletteOpen} onClose={() => toggleCommandPalette(false)} />

      {/* Floating watchtower (Grok Build 2 signature — the many eyes) */}
      <Watchtower />

      <main className="flex flex-1 overflow-hidden" style={{ paddingTop: '56px' }}>
        {/* THE EYES — left rail, always present, collapsible via focus */}
        <EyesPanel forceCollapsed={focusMode} />

        {/* Meeting mode: four orchestrated agents */}
        {meetingMode ? (
          <div className="flex flex-1 overflow-hidden">
            <ConversationPanel onOpenArtifact={openArtifact} meetingBrief={MEETING_BRIEF_PANE1} />

            <motion.div
              key="meeting-grid"
              className="flex overflow-hidden"
              style={{ flex: 2, minWidth: 0 }}
              initial={{ opacity: 0, x: 16 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0 }}
            >
              {[2, 3, 4].map((idx) => (
                <ChatPane
                  key={`meeting-${idx}`}
                  paneIndex={idx}
                  initialModel={MEETING_BRIEFS[idx as 2 | 3 | 4].model}
                  openingBrief={MEETING_BRIEFS[idx as 2 | 3 | 4].brief}
                  onClose={endMeeting}
                />
              ))}
            </motion.div>
          </div>
        ) : (
          <>
            {/* Primary conversation (pane 1) */}
            <ConversationPanel onOpenArtifact={openArtifact} />

            {/* Right area: Mind / Artifact or additional ChatPanes */}
            <AnimatePresence mode="wait">
              {paneCount === 1 && (
                <motion.div
                  key="single-right"
                  className="flex flex-1 overflow-hidden"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  transition={{ duration: 0.16 }}
                >
                  {artifactState ? (
                    <div className="flex-1 overflow-hidden min-w-0">
                      <ArtifactPanel
                        artifacts={artifactState.artifacts}
                        initialIndex={artifactState.index}
                        onClose={closeArtifact}
                      />
                    </div>
                  ) : (
                    <MindPanel forceCollapsed={focusMode} />
                  )}
                </motion.div>
              )}

              {paneCount === 2 && (
                <motion.div key="dual" className="flex overflow-hidden" style={{ flex: 1, minWidth: 0 }} initial={{ opacity: 0, x: 12 }} animate={{ opacity: 1, x: 0 }}>
                  <ChatPane paneIndex={2} initialModel={pane2Model} onClose={() => handleSetPaneCount(1)} />
                </motion.div>
              )}

              {paneCount === 3 && (
                <motion.div key="triple" className="flex overflow-hidden" style={{ flex: 1, minWidth: 0 }} initial={{ opacity: 0, x: 12 }} animate={{ opacity: 1, x: 0 }}>
                  <ChatPane paneIndex={2} initialModel={pane2Model} onClose={() => handleSetPaneCount(2)} />
                  <ChatPane paneIndex={3} initialModel={pane3Model} onClose={() => handleSetPaneCount(2)} />
                </motion.div>
              )}
            </AnimatePresence>
          </>
        )}
      </main>

      {/* Tiny footer hint — Grok signature */}
      <div className="fixed bottom-1 right-3 text-[9px] font-mono tracking-widest opacity-30 pointer-events-none select-none z-[60]">
        GROK BUILD 2 • ARGUS1 REIMAGINED • ⌘K
      </div>
    </div>
  );
}
