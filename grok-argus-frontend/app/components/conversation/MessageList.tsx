'use client';
import { useEffect, useRef } from 'react';
import { useAgentStore } from '../../hooks/useAgentState';
import { UserMessage } from './UserMessage';
import { ArgusMessage } from './ArgusMessage';
import { ToolCallBlock } from './ToolCallBlock';
import { Artifact } from '../../lib/types';

interface Props { onOpenArtifact: (arts: Artifact[], idx: number) => void; }

export function MessageList({ onOpenArtifact }: Props) {
  const msgs = useAgentStore(s => s.messages);
  const stream = useAgentStore(s => s.streamingContent);
  const streaming = useAgentStore(s => s.isStreaming);
  const bottom = useRef<HTMLDivElement>(null);

  useEffect(() => { bottom.current?.scrollIntoView({ behavior: 'smooth' }); }, [msgs, stream]);

  return (
    <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
      {msgs.map(msg => {
        if (msg.role === 'user') return <UserMessage key={msg.id} message={msg} />;
        return (
          <div key={msg.id}>
            {msg.toolCalls?.map(tc => <ToolCallBlock key={tc.id} toolCall={tc} />)}
            {(msg.content || msg.artifacts?.length) && <ArgusMessage message={msg} onOpenArtifact={onOpenArtifact} />}
          </div>
        );
      })}
      {streaming && stream && (
        <div className="animate-[fade-in_120ms]">
          <div className="argus-prose text-sm leading-relaxed whitespace-pre-wrap">{stream}<span className="inline-block w-1.5 h-3.5 bg-[#f5b800] ml-0.5 animate-[pulse-rapid_800ms_infinite] align-bottom" /></div>
        </div>
      )}
      <div ref={bottom} />
    </div>
  );
}
