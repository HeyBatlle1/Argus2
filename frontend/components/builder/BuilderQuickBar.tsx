'use client';

import { BUILDER_QUICK_PROMPTS } from '@/lib/builder';
import { useAgentStore } from '@/hooks/useAgentState';

export function BuilderQuickBar() {
  const sendMessage = useAgentStore((s) => s.sendMessage);
  const isStreaming = useAgentStore((s) => s.isStreaming);

  return (
    <div className="flex flex-wrap gap-1.5 mb-2">
      {BUILDER_QUICK_PROMPTS.map(({ id, label, prompt }) => (
        <button
          key={id}
          disabled={isStreaming}
          onClick={() => sendMessage(prompt)}
          className="text-[8px] font-mono px-2 py-1 rounded transition-all uppercase tracking-wider"
          style={{
            border: '1px solid rgba(74, 222, 128, 0.25)',
            color: isStreaming ? '#3a3a48' : '#4ade80',
            background: 'rgba(74, 222, 128, 0.06)',
            opacity: isStreaming ? 0.45 : 1,
            cursor: isStreaming ? 'not-allowed' : 'pointer',
          }}
        >
          {label}
        </button>
      ))}
    </div>
  );
}