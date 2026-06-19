'use client';

import { History, Radio, Maximize2, Minimize2, CalendarClock, Command } from 'lucide-react';
import { ArgusEye } from './ArgusEye';
import { ConnectionStatus } from './ConnectionStatus';
import { ModelSelector } from './ModelSelector';
import { SentryDropdown } from './SentryDropdown';
import { useAgentStore } from '../../hooks/useAgentState';

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

export function Header(props: Props) {
  const {
    onToggleHistory, onToggleScheduler, schedulerOpen, paneCount, onSetPaneCount,
    meetingMode, onStartMeeting, focusMode, onToggleFocus,
  } = props;

  const title = useAgentStore((s) => s.currentConversationTitle);
  const conversations = useAgentStore((s) => s.conversations);
  const toggleCommand = useAgentStore((s) => s.toggleCommandPalette);

  return (
    <header className="fixed top-0 left-0 right-0 z-40 h-14 flex items-center justify-between px-4 border-b"
            style={{ background: '#0d0d14', borderColor: '#1e1e32' }}>
      <div className="flex items-center gap-3">
        <ArgusEye />
        <div className="flex flex-col">
          <span className="font-mono text-sm font-bold tracking-[0.22em] uppercase text-[#f5b800] leading-none">ARGUS</span>
          <span className="font-mono text-[9px] tracking-widest uppercase text-[#b8b5ac] leading-none mt-0.5">The Hundred-Eyed Agent • Grok Build 2</span>
        </div>

        <div className="h-5 w-px mx-1.5" style={{ background: '#1e1e32' }} />

        <button onClick={onToggleHistory} className="flex items-center gap-1.5 px-2.5 py-1.5 rounded text-[9px] font-mono tracking-wider uppercase transition-all"
                style={{ border: '1px solid #2a2a42', color: '#7a7a9a', background: 'rgba(255,255,255,0.025)' }}>
          <History size={11} /> HISTORY
          {conversations.length > 0 && <span className="text-[8px] px-1 rounded" style={{ background: 'rgba(255,176,0,0.15)', color: '#ffb000' }}>{conversations.length}</span>}
        </button>

        <button onClick={onToggleScheduler} className="flex items-center gap-1.5 px-2.5 py-1.5 rounded text-[9px] font-mono tracking-wider uppercase transition-all"
                style={{ border: schedulerOpen ? '1px solid rgba(57,211,83,0.5)' : '1px solid #2a2a42', color: schedulerOpen ? '#39d353' : '#7a7a9a', background: schedulerOpen ? 'rgba(57,211,83,0.08)' : 'rgba(255,255,255,0.025)' }}>
          <CalendarClock size={11} /> TASKS
        </button>

        <button onClick={onStartMeeting} disabled={meetingMode} className="flex items-center gap-1.5 px-2.5 py-1.5 rounded text-[9px] font-mono tracking-wider uppercase transition-all"
                style={{ border: meetingMode ? '1px solid rgba(201,168,76,0.55)' : '1px solid #2a2a42', color: meetingMode ? '#c9a84c' : '#7a7a9a', background: meetingMode ? 'rgba(201,168,76,0.1)' : 'rgba(255,255,255,0.025)' }}>
          <Radio size={11} className={meetingMode ? 'animate-pulse' : ''} /> {meetingMode ? 'MEETING LIVE' : 'MEETING'}
        </button>
      </div>

      {title && !meetingMode && (
        <div className="absolute left-1/2 -translate-x-1/2 max-w-xs text-center text-[10px] font-mono" style={{ color: '#3a3a5a' }}>{title}</div>
      )}
      {meetingMode && (
        <div className="absolute left-1/2 -translate-x-1/2 flex items-center gap-2 text-[10px] font-mono tracking-widest uppercase" style={{ color: '#c9a84c' }}>
          <span className="w-1.5 h-1.5 rounded-full animate-pulse" style={{ background: '#c9a84c' }} /> MONTHLY MEETING IN PROGRESS <span className="w-1.5 h-1.5 rounded-full animate-pulse" style={{ background: '#c9a84c', animationDelay: '520ms' }} />
        </div>
      )}

      <div className="flex items-center gap-2">
        <button onClick={onToggleFocus} title={focusMode ? 'Exit focus' : 'Focus mode'} className="flex items-center justify-center transition-all" style={{ width: 28, height: 26, borderRadius: 7, background: focusMode ? 'rgba(201,168,76,0.18)' : 'rgba(255,255,255,0.03)', border: focusMode ? '1px solid rgba(201,168,76,0.5)' : '1px solid #2a2a42', color: focusMode ? '#c9a84c' : '#7a7a9a' }}>
          {focusMode ? <Minimize2 size={12} /> : <Maximize2 size={12} />}
        </button>

        {!meetingMode && (
          <div className="flex items-center rounded overflow-hidden" style={{ border: '1px solid #2a2a42' }}>
            {[1, 2, 3].map(n => (
              <button key={n} onClick={() => onSetPaneCount(n as 1 | 2 | 3)} className="flex items-center justify-center transition-all" style={{ width: 28, height: 26, background: paneCount === n ? 'rgba(201,168,76,0.18)' : 'rgba(255,255,255,0.02)', borderRight: n < 3 ? '1px solid #2a2a42' : undefined }}>
                <PaneIcon count={n} active={paneCount === n} />
              </button>
            ))}
          </div>
        )}

        <div className="h-4 w-px" style={{ background: '#1e1e32' }} />

        <button onClick={() => toggleCommand(true)} className="flex items-center gap-1 px-2 py-1 text-[10px] font-mono rounded border" style={{ borderColor: '#2a2a42', color: '#7a7a9a' }} title="Command palette (⌘K)">
          <Command size={12} /> ⌘K
        </button>

        <ConnectionStatus />
        <SentryDropdown />
        <ModelSelector />
      </div>
    </header>
  );
}

function PaneIcon({ count, active }: { count: number; active: boolean }) {
  const c = active ? '#c9a84c' : '#3a3a5a';
  if (count === 1) return <svg width="14" height="12" viewBox="0 0 14 12" fill="none"><rect x="1" y="1" width="12" height="10" rx="1" stroke={c} strokeWidth="1.2"/></svg>;
  if (count === 2) return <svg width="14" height="12" viewBox="0 0 14 12" fill="none"><rect x="1" y="1" width="5.5" height="10" rx="1" stroke={c} strokeWidth="1.2"/><rect x="7.5" y="1" width="5.5" height="10" rx="1" stroke={c} strokeWidth="1.2"/></svg>;
  return <svg width="14" height="12" viewBox="0 0 14 12" fill="none"><rect x="1" y="1" width="3.5" height="10" rx="0.8" stroke={c} strokeWidth="1.2"/><rect x="5.2" y="1" width="3.5" height="10" rx="0.8" stroke={c} strokeWidth="1.2"/><rect x="9.4" y="1" width="3.5" height="10" rx="0.8" stroke={c} strokeWidth="1.2"/></svg>;
}
