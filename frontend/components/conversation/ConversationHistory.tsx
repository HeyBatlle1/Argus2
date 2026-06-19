'use client';

import { useAgentStore } from '@/hooks/useAgentState';
import { Conversation } from '@/lib/types';

function timeAgo(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const m = Math.floor(diff / 60000);
  if (m < 1) return 'just now';
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

function modelShort(model: string | null): string {
  if (!model) return '';
  if (model.includes('haiku'))  return 'Haiku';
  if (model.includes('sonnet')) return 'Sonnet';
  if (model.includes('opus'))   return 'Opus';
  if (model.includes('gemini')) return 'Gemini';
  if (model.includes('grok'))   return 'Grok';
  return model.split('/').pop() ?? model;
}

interface RowProps {
  conv: Conversation;
  active: boolean;
  onLoad: (id: string) => void;
}

function ConversationRow({ conv, active, onLoad }: RowProps) {
  return (
    <button
      onClick={() => onLoad(conv.id)}
      className="w-full text-left px-3 py-2 rounded transition-colors"
      style={{
        background: active ? 'rgba(255,176,0,0.08)' : 'transparent',
        border: active ? '1px solid rgba(255,176,0,0.25)' : '1px solid transparent',
      }}
    >
      <div className="flex items-start justify-between gap-2">
        <span
          className="text-xs font-mono truncate"
          style={{ color: active ? '#ffb000' : '#c8c8d8', maxWidth: '75%' }}
        >
          {conv.title}
        </span>
        <span className="text-[9px] font-mono flex-shrink-0" style={{ color: '#5a5a7a', marginTop: '1px' }}>
          {timeAgo(conv.lastActiveAt)}
        </span>
      </div>
      <div className="flex items-center gap-2 mt-0.5">
        {conv.model && (
          <span className="text-[9px] font-mono" style={{ color: '#5a5a7a' }}>
            {modelShort(conv.model)}
          </span>
        )}
        <span className="text-[9px] font-mono" style={{ color: '#5a5a7a' }}>
          {conv.messageCount} turns
        </span>
      </div>
    </button>
  );
}

export function ConversationHistory() {
  const conversations = useAgentStore((s) => s.conversations);
  const currentId = useAgentStore((s) => s.currentConversationId);
  const loadConversation = useAgentStore((s) => s.loadConversation);
  const newConversation = useAgentStore((s) => s.newConversation);

  return (
    <div className="flex flex-col h-full" style={{ background: '#0d0d16' }}>
      <div
        className="flex items-center justify-between px-4 py-2 border-b flex-shrink-0"
        style={{ borderColor: '#32325a', background: 'var(--surface-hi)' }}
      >
        <span className="text-[9px] font-mono tracking-widest uppercase" style={{ color: '#ffb000' }}>
          PAST CONVERSATIONS
        </span>
        <button
          onClick={newConversation}
          className="text-[9px] font-mono px-2 py-0.5 rounded transition-colors"
          style={{
            background: 'rgba(255,176,0,0.1)',
            border: '1px solid rgba(255,176,0,0.3)',
            color: '#ffb000',
          }}
          title="Start a new conversation"
        >
          + NEW
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-2 py-2 space-y-1">
        {conversations.length === 0 ? (
          <p className="text-[10px] font-mono text-center py-8" style={{ color: '#5a5a7a' }}>
            No past conversations yet
          </p>
        ) : (
          conversations.map((conv) => (
            <ConversationRow
              key={conv.id}
              conv={conv}
              active={conv.id === currentId}
              onLoad={loadConversation}
            />
          ))
        )}
      </div>
    </div>
  );
}
