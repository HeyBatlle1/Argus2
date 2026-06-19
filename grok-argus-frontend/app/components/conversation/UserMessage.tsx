'use client';
import { Message } from '../../lib/types';

export function UserMessage({ message }: { message: Message }) {
  return (
    <div className="flex justify-end">
      <div className="max-w-[68%]">
        <div className="px-4 py-2.5 rounded text-sm leading-relaxed" style={{ background: '#1e1e38', borderLeft: '3px solid #5aafef', color: '#e8e5dc' }}>{message.content}</div>
        <div className="text-right mt-0.5 px-1 text-[9px] font-mono text-[#b8b5ac]">{message.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</div>
      </div>
    </div>
  );
}
