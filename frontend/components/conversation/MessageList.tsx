'use client';

import { useEffect, useRef } from 'react';
import { useAgentStore } from '@/hooks/useAgentState';
import { UserMessage } from './UserMessage';
import { ArgusMessage } from './ArgusMessage';
import { ToolCallBlock } from './ToolCallBlock';

export function MessageList() {
  const messages = useAgentStore((s) => s.messages);
  const streamingContent = useAgentStore((s) => s.streamingContent);
  const isStreaming = useAgentStore((s) => s.isStreaming);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingContent]);

  return (
    <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
      {messages.map((msg) => {
        if (msg.role === 'user') {
          return <UserMessage key={msg.id} message={msg} />;
        }
        // Assistant message — may have tool calls, content, or both
        return (
          <div key={msg.id}>
            {msg.toolCalls?.map((tc) => (
              <ToolCallBlock key={tc.id} toolCall={tc} />
            ))}
            {msg.content && <ArgusMessage message={msg} />}
          </div>
        );
      })}

      {/* Streaming in-progress */}
      {isStreaming && streamingContent && (
        <div className="animate-fade-in">
          <div className="argus-prose text-argus-text text-sm leading-relaxed whitespace-pre-wrap">
            {streamingContent}
            <span className="inline-block w-1.5 h-4 bg-argus-amber ml-0.5 animate-pulse-rapid align-bottom" />
          </div>
        </div>
      )}

      <div ref={bottomRef} />
    </div>
  );
}
