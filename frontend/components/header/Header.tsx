'use client';

import { History, Radio, Maximize2, Minimize2, CalendarClock } from 'lucide-react';
import { ArgusEye } from './ArgusEye';
import { ConnectionStatus } from './ConnectionStatus';
import { ModelSelector } from './ModelSelector';
import { SentryDropdown } from './SentryDropdown';
import { useAgentStore } from '@/hooks/useAgentState';

interface Props {
  onToggleHistory: () => void;
  onToggleScheduler: () => void;
  schedulerOpen: boolean;
  paneCount: 1 | 2 | 3;
  onSetPaneCount: (n: 1 | 2 | 3) => void;
  meetingMode: boolean;
  onStartMeeting: () => void;
  focusMode: boolean;
  onToggleFocus: () => void;
}

export function Header({ onToggleHistory, onToggleScheduler, schedulerOpen, paneCount, onSetPaneCount, meetingMode, onStartMeeting, focusMode, onToggleFocus }: Props) {
  const title = useAgentStore((s) => s.currentConversationTitle);
  const conversations = useAgentStore((s) => s.conversations);

  return (
    <header
      className="fixed top-0 left-0 right-0 z-30 h-14 flex items-center justify-between px-4 border-b border-argus-border"
      style={{ background: '#0d0d14' }}
    >
      {/* Left: Logo + History */}
      <div className="flex items-center gap-3">
        <ArgusEye />
        <div className="flex flex-col">
          <span className="font-mono text-sm font-bold tracking-[0.2em] uppercase text-argus-amber leading-none">
            ARGUS
          </span>
          <span className="font-mono text-[9px] tracking-widest uppercase text-argus-textDim leading-none mt-0.5">
            The Hundred-Eyed Agent
          </span>
        </div>

        {/* Divider */}
        <div className="h-6 w-px mx-1" style={{ background: '#1e1e32' }} />

        {/* History button */}
        <button
          onClick={onToggleHistory}
          className="flex items-center gap-1.5 px-2.5 py-1.5 rounded transition-all text-[9px] font-mono tracking-wider uppercase cursor-pointer"
          style={{
            border: '1px solid #2a2a42',
            color: '#7a7a9a',
            background: 'rgba(255,255,255,0.03)',
          }}
          onMouseEnter={(e) => {
            (e.currentTarget as HTMLButtonElement).style.color = '#ffb000';
            (e.currentTarget as HTMLButtonElement).style.borderColor = 'rgba(255,176,0,0.4)';
            (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,176,0,0.07)';
          }}
          onMouseLeave={(e) => {
            (e.currentTarget as HTMLButtonElement).style.color = '#7a7a9a';
            (e.currentTarget as HTMLButtonElement).style.borderColor = '#2a2a42';
            (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,255,255,0.03)';
          }}
          title={`${conversations.length} conversation${conversations.length !== 1 ? 's' : ''}`}
        >
          <History size={11} />
          HISTORY
          {conversations.length > 0 && (
            <span
              className="text-[8px] font-mono px-1 rounded"
              style={{ background: 'rgba(255,176,0,0.15)', color: '#ffb000' }}
            >
              {conversations.length}
            </span>
          )}
        </button>

        {/* Task Scheduler button */}
        <button
          onClick={onToggleScheduler}
          className="flex items-center gap-1.5 px-2.5 py-1.5 rounded transition-all text-[9px] font-mono tracking-wider uppercase cursor-pointer"
          style={{
            border: schedulerOpen ? '1px solid rgba(57,211,83,0.5)' : '1px solid #2a2a42',
            color: schedulerOpen ? '#39d353' : '#7a7a9a',
            background: schedulerOpen ? 'rgba(57,211,83,0.08)' : 'rgba(255,255,255,0.03)',
          }}
          onMouseEnter={(e) => {
            if (schedulerOpen) return;
            (e.currentTarget as HTMLButtonElement).style.color = '#39d353';
            (e.currentTarget as HTMLButtonElement).style.borderColor = 'rgba(57,211,83,0.4)';
            (e.currentTarget as HTMLButtonElement).style.background = 'rgba(57,211,83,0.06)';
          }}
          onMouseLeave={(e) => {
            if (schedulerOpen) return;
            (e.currentTarget as HTMLButtonElement).style.color = '#7a7a9a';
            (e.currentTarget as HTMLButtonElement).style.borderColor = '#2a2a42';
            (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,255,255,0.03)';
          }}
          title="Task scheduler — deploy agents on a schedule"
        >
          <CalendarClock size={11} />
          TASKS
        </button>

        {/* Monthly Meeting button */}
        <button
          onClick={onStartMeeting}
          disabled={meetingMode}
          className="flex items-center gap-1.5 px-2.5 py-1.5 rounded transition-all text-[9px] font-mono tracking-wider uppercase cursor-pointer"
          style={{
            border: meetingMode ? '1px solid rgba(201,168,76,0.55)' : '1px solid #2a2a42',
            color: meetingMode ? '#c9a84c' : '#7a7a9a',
            background: meetingMode ? 'rgba(201,168,76,0.1)' : 'rgba(255,255,255,0.03)',
          }}
          onMouseEnter={(e) => {
            if (meetingMode) return;
            (e.currentTarget as HTMLButtonElement).style.color = '#c9a84c';
            (e.currentTarget as HTMLButtonElement).style.borderColor = 'rgba(201,168,76,0.45)';
            (e.currentTarget as HTMLButtonElement).style.background = 'rgba(201,168,76,0.07)';
          }}
          onMouseLeave={(e) => {
            if (meetingMode) return;
            (e.currentTarget as HTMLButtonElement).style.color = '#7a7a9a';
            (e.currentTarget as HTMLButtonElement).style.borderColor = '#2a2a42';
            (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,255,255,0.03)';
          }}
          title="Run monthly meeting — all 4 models research simultaneously"
        >
          <Radio size={11} className={meetingMode ? 'animate-pulse' : ''} />
          {meetingMode ? 'MEETING LIVE' : 'MEETING'}
        </button>
      </div>

      {/* Center: current conversation title */}
      {title && !meetingMode && (
        <div className="absolute left-1/2 -translate-x-1/2 max-w-xs truncate text-center">
          <span className="text-[10px] font-mono" style={{ color: '#3a3a5a' }}>
            {title}
          </span>
        </div>
      )}

      {/* Center: meeting mode indicator */}
      {meetingMode && (
        <div className="absolute left-1/2 -translate-x-1/2 flex items-center gap-2">
          <span
            className="w-1.5 h-1.5 rounded-full animate-pulse"
            style={{ background: '#c9a84c' }}
          />
          <span className="text-[10px] font-mono tracking-widest uppercase" style={{ color: '#c9a84c' }}>
            Monthly Meeting in Progress
          </span>
          <span
            className="w-1.5 h-1.5 rounded-full animate-pulse"
            style={{ background: '#c9a84c', animationDelay: '0.5s' }}
          />
        </div>
      )}

      {/* Right: Pane toggles + Connection + Sentry + Model */}
      <div className="flex items-center gap-2">
        {/* Focus mode toggle */}
        <button
          onClick={onToggleFocus}
          className="flex items-center justify-center transition-all cursor-pointer"
          style={{
            width: 28,
            height: 26,
            borderRadius: '7px',
            background: focusMode ? 'rgba(201,168,76,0.18)' : 'rgba(255,255,255,0.03)',
            border: focusMode ? '1px solid rgba(201,168,76,0.5)' : '1px solid #2a2a42',
            color: focusMode ? '#c9a84c' : '#7a7a9a',
          }}
          onMouseEnter={(e) => {
            if (focusMode) return;
            (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,255,255,0.07)';
            (e.currentTarget as HTMLButtonElement).style.color = '#c8c8d8';
          }}
          onMouseLeave={(e) => {
            if (focusMode) return;
            (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,255,255,0.03)';
            (e.currentTarget as HTMLButtonElement).style.color = '#7a7a9a';
          }}
          title={focusMode ? 'Exit focus mode' : 'Focus mode — hide side panels'}
        >
          {focusMode ? <Minimize2 size={12} /> : <Maximize2 size={12} />}
        </button>

        {/* Split-pane controls — hidden during meeting mode */}
        {!meetingMode && (
          <div
            className="flex items-center rounded overflow-hidden"
            style={{ border: '1px solid #2a2a42' }}
            title="Split panes"
          >
            {([1, 2, 3] as const).map((n) => (
              <button
                key={n}
                onClick={() => onSetPaneCount(n)}
                className="flex items-center justify-center transition-all cursor-pointer"
                style={{
                  width: 28,
                  height: 26,
                  background: paneCount === n ? 'rgba(201,168,76,0.18)' : 'rgba(255,255,255,0.02)',
                  borderRight: n < 3 ? '1px solid #2a2a42' : undefined,
                }}
                onMouseEnter={(e) => {
                  if (paneCount === n) return;
                  (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,255,255,0.07)';
                }}
                onMouseLeave={(e) => {
                  if (paneCount === n) return;
                  (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,255,255,0.02)';
                }}
                title={`${n} pane${n > 1 ? 's' : ''}`}
              >
                <PaneIcon count={n} active={paneCount === n} />
              </button>
            ))}
          </div>
        )}

        <div className="h-4 w-px" style={{ background: '#1e1e32' }} />
        <ConnectionStatus />
        <SentryDropdown />
        <ModelSelector />
      </div>
    </header>
  );
}

function PaneIcon({ count, active }: { count: number; active: boolean }) {
  const color = active ? '#c9a84c' : '#3a3a5a';
  if (count === 1) return (
    <svg width="14" height="12" viewBox="0 0 14 12" fill="none">
      <rect x="1" y="1" width="12" height="10" rx="1" stroke={color} strokeWidth="1.2"/>
    </svg>
  );
  if (count === 2) return (
    <svg width="14" height="12" viewBox="0 0 14 12" fill="none">
      <rect x="1" y="1" width="5.5" height="10" rx="1" stroke={color} strokeWidth="1.2"/>
      <rect x="7.5" y="1" width="5.5" height="10" rx="1" stroke={color} strokeWidth="1.2"/>
    </svg>
  );
  return (
    <svg width="14" height="12" viewBox="0 0 14 12" fill="none">
      <rect x="1"   y="1" width="3.5" height="10" rx="0.8" stroke={color} strokeWidth="1.2"/>
      <rect x="5.2" y="1" width="3.5" height="10" rx="0.8" stroke={color} strokeWidth="1.2"/>
      <rect x="9.4" y="1" width="3.5" height="10" rx="0.8" stroke={color} strokeWidth="1.2"/>
    </svg>
  );
}
