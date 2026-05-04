'use client';

import { MessageList } from './MessageList';
import { InputArea } from './InputArea';
import { Artifact } from '@/lib/types';

interface Props {
  onOpenArtifact: (artifacts: Artifact[], index: number) => void;
}

export function ConversationPanel({ onOpenArtifact }: Props) {
  return (
    <div
      className="flex-1 flex flex-col h-full min-w-[400px] scanlines"
      style={{ borderRight: '1px solid #32325a' }}
    >
      <div className="flex-shrink-0 px-4 py-2 border-b border-argus-borderBright flex items-center" style={{ background: 'var(--surface-hi)' }}>
        <span className="text-[9px] font-mono tracking-widest uppercase text-argus-amber">
          THE CONVERSATION
        </span>
      </div>
      <MessageList onOpenArtifact={onOpenArtifact} />
      <InputArea />
    </div>
  );
}
