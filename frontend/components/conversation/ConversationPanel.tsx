'use client';

import { useEffect, useRef } from 'react';
import { useAgentStore } from '@/hooks/useAgentState';
import { MessageList } from './MessageList';
import { InputArea } from './InputArea';
import { Artifact } from '@/lib/types';
import { StarfieldBackground } from '@/components/shared/StarfieldBackground';
import { Plus } from 'lucide-react';
import { MODEL_CONFIG } from '@/lib/models';
import { EYE_COLORS } from '@/lib/constants';
import { isBuilderModel, BUILDER_THEME } from '@/lib/builder';

interface Props {
  onOpenArtifact: (artifacts: Artifact[], index: number) => void;
  meetingBrief?: string;
  focusMode?: boolean;
}

export function ConversationPanel({ onOpenArtifact, meetingBrief, focusMode = false }: Props) {
  const title = useAgentStore((s) => s.currentConversationTitle);
  const newConversation = useAgentStore((s) => s.newConversation);
  const isStreaming = useAgentStore((s) => s.isStreaming);
  const connected = useAgentStore((s) => s.connected);
  const sendMessage = useAgentStore((s) => s.sendMessage);
  const messages = useAgentStore((s) => s.messages);
  const eyeState = useAgentStore((s) => s.eyeState);
  const activeModel = useAgentStore((s) => s.activeModel);
  const wsLatency = useAgentStore((s) => s.wsLatency);
  const activeToolCalls = useAgentStore((s) => s.activeToolCalls);
  const modelCfg = MODEL_CONFIG[activeModel];
  const builderMode = isBuilderModel(activeModel);

  const briefSentRef = useRef(false);
  useEffect(() => {
    if (connected && meetingBrief && !briefSentRef.current && messages.length === 0) {
      briefSentRef.current = true;
      setTimeout(() => sendMessage(meetingBrief), 1200);
    }
  }, [connected, meetingBrief, messages.length, sendMessage]);

  return (
    <div
      className="flex-1 flex flex-col h-full min-w-[400px] relative overflow-hidden"
      style={{ borderRight: '1px solid #32325a' }}
    >
      <StarfieldBackground
        eyeState={eyeState}
        motionScale={focusMode ? 0.35 : 1}
        builderMode={builderMode}
      />

      <div
        className="flex-shrink-0 px-4 py-2 border-b border-argus-borderBright flex items-center justify-between relative z-10 chat-panel-header"
      >
        <div className="flex items-center gap-2 min-w-0 flex-1">
          <span
            className="text-[9px] font-mono tracking-widest uppercase flex-shrink-0"
            style={{ color: builderMode ? BUILDER_THEME.accent : '#f5b800' }}
          >
            {builderMode ? 'BUILDER STATION' : 'CONVERSATION'}
          </span>
          {title && (
            <span
              className="text-[9px] font-mono truncate hidden sm:inline"
              style={{ color: '#6a6a8a', maxWidth: '200px' }}
              title={title}
            >
              {title}
            </span>
          )}
          <div className="flex items-center gap-1.5 ml-1 flex-shrink-0">
            <span
              className="w-1.5 h-1.5 rounded-full"
              style={{
                background: connected ? '#39d353' : '#3a3a48',
                boxShadow: connected ? '0 0 6px rgba(57,211,83,0.5)' : 'none',
              }}
            />
            <span className="text-[8px] font-mono" style={{ color: '#4a4a5a' }}>
              {connected ? `${wsLatency}ms` : 'offline'}
            </span>
            <span
              className="text-[8px] font-mono px-1.5 py-px rounded"
              style={{ background: `${modelCfg.color}14`, color: modelCfg.color, border: `1px solid ${modelCfg.color}33` }}
            >
              {modelCfg.name.toUpperCase()}
            </span>
            {isStreaming && (
              <span className="text-[8px] font-mono px-1.5 py-px rounded animate-pulse" style={{ color: EYE_COLORS[eyeState], background: `${EYE_COLORS[eyeState]}18` }}>
                {eyeState.toUpperCase()}
                {activeToolCalls.length > 0 ? ` · ${activeToolCalls.length} tool${activeToolCalls.length > 1 ? 's' : ''}` : ''}
              </span>
            )}
          </div>
        </div>

        <button
          onClick={newConversation}
          disabled={isStreaming}
          className="panel-action-btn flex items-center gap-1 text-[9px] font-mono px-2 py-1 rounded flex-shrink-0"
          style={{ opacity: isStreaming ? 0.4 : 1, cursor: isStreaming ? 'not-allowed' : 'pointer' }}
          title="New conversation"
        >
          <Plus size={10} />
          NEW
        </button>
      </div>

      <div className="flex-1 flex flex-col overflow-hidden relative z-10">
        <MessageList onOpenArtifact={onOpenArtifact} />
        <InputArea />
      </div>
    </div>
  );
}