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
          style={{ background: '#1e1e38', color: '#e8e5dc', borderLeft: '3px solid #5aafef' }}
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
