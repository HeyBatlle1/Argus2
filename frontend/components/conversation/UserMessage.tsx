'use client';

import { Message } from '@/lib/types';

interface Props {
  message: Message;
}

function formatTime(d: Date) {
  return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
}

export function UserMessage({ message }: Props) {
  return (
    <div className="flex justify-end animate-fade-in">
      <div className="max-w-[72%]">
        <div
          className="px-4 py-3 rounded text-sm leading-relaxed"
          style={{ background: '#1a1a2e', color: '#d4d0c8' }}
        >
          {message.content}
        </div>
        <div className="flex justify-end mt-1 px-1">
          <span className="text-[10px] font-mono text-argus-textDim">{formatTime(message.timestamp)}</span>
        </div>
      </div>
    </div>
  );
}
