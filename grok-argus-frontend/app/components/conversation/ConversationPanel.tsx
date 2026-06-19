'use client';
import { useEffect, useRef } from 'react';
import { useAgentStore } from '../../hooks/useAgentState';
import { MessageList } from './MessageList';
import { InputArea } from './InputArea';
import { Artifact } from '../../lib/types';
import { Plus } from 'lucide-react';

const FERRIS = `<svg viewBox="0 0 320 200" xmlns="http://www.w3.org/2000/svg" fill="#c9a84c"><g transform="translate(160,102)"><ellipse cx="0" cy="0" rx="48" ry="35"/><ellipse cx="0" cy="-4" rx="30" ry="19" fill="none" stroke-width="1.4"/><circle cx="-19" cy="-18" r="12"/><circle cx="-17" cy="-18" r="6.5" fill="#0a0a0f"/><circle cx="-14" cy="-21" r="1.8"/><circle cx="19" cy="-18" r="12"/><circle cx="21" cy="-18" r="6.5" fill="#0a0a0f"/><circle cx="24" cy="-21" r="1.8"/><line x1="-28" y1="-31" x2="-11" y2="-28" stroke-width="2.2" stroke-linecap="round"/><line x1="11" y1="-28" x2="28" y2="-31" stroke-width="2.2" stroke-linecap="round"/></g></svg>`;

interface Props { onOpenArtifact: (a: Artifact[], i: number) => void; meetingBrief?: string; }

export function ConversationPanel({ onOpenArtifact, meetingBrief }: Props) {
  const title = useAgentStore(s => s.currentConversationTitle);
  const newConv = useAgentStore(s => s.newConversation);
  const isStreaming = useAgentStore(s => s.isStreaming);
  const connected = useAgentStore(s => s.connected);
  const send = useAgentStore(s => s.sendMessage);
  const msgs = useAgentStore(s => s.messages);

  const sentRef = useRef(false);
  useEffect(() => {
    if (connected && meetingBrief && !sentRef.current && msgs.length === 0) {
      sentRef.current = true;
      setTimeout(() => send(meetingBrief), 900);
    }
  }, [connected, meetingBrief, msgs.length, send]);

  return (
    <div className="flex-1 flex flex-col h-full min-w-[420px] scanlines relative" style={{ borderRight: '1px solid #32325a' }}>
      <div className="absolute inset-0 flex items-center justify-center pointer-events-none" style={{ opacity: 0.065, zIndex: 0 }} dangerouslySetInnerHTML={{ __html: FERRIS }} />

      <div className="flex-shrink-0 px-4 py-2 border-b flex items-center justify-between relative z-10" style={{ background: '#1e1e38', borderColor: '#5a5a8a' }}>
        <div className="flex items-center gap-3 min-w-0">
          <span className="text-[9px] font-mono tracking-[0.16em] uppercase text-[#f5b800] flex-shrink-0">CONVERSATION</span>
          {title && <span className="text-[9px] font-mono text-[#3a3a5a] truncate max-w-[240px]" title={title}>{title}</span>}
        </div>
        <button onClick={newConv} disabled={isStreaming} className="flex items-center gap-1 text-[9px] font-mono px-2 py-1 rounded transition" style={{ color: isStreaming ? '#5a5a7a' : '#5a5a7a', opacity: isStreaming ? 0.4 : 1 }}>
          <Plus size={10} /> NEW
        </button>
      </div>

      <div className="flex-1 flex flex-col overflow-hidden relative z-10">
        <MessageList onOpenArtifact={onOpenArtifact} />
        <InputArea />
      </div>
    </div>
  );
}
