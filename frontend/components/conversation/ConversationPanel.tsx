'use client';

import { MessageList } from './MessageList';
import { InputArea } from './InputArea';

export function ConversationPanel() {
  return (
    <div
      className="flex-1 flex flex-col h-full min-w-[400px] scanlines"
      style={{ borderRight: '1px solid #1a1a2e' }}
    >
      <div className="flex-shrink-0 px-4 py-2 border-b border-argus-border flex items-center">
        <span className="text-[9px] font-mono tracking-widest uppercase text-argus-amberDim">
          THE CONVERSATION
        </span>
      </div>
      <MessageList />
      <InputArea />
    </div>
  );
}
