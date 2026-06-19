'use client';

import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { dracula } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { Layers } from 'lucide-react';
import { Message, Artifact } from '@/lib/types';
import { artifactLabel } from '@/lib/artifacts';

interface Props {
  message: Message;
  onOpenArtifact?: (artifacts: Artifact[], index: number) => void;
}

function formatTime(d: Date) {
  return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
}

export function ArgusMessage({ message, onOpenArtifact }: Props) {
  if (!message.content && !message.artifacts?.length) return null;

  const artifacts = message.artifacts ?? [];

  return (
    <div className="animate-fade-in">
      {message.content && (
        <div className="message-glass argus-prose text-argus-text text-sm leading-relaxed px-4 py-3 rounded max-w-[92%]">
          <ReactMarkdown
            remarkPlugins={[remarkGfm]}
            components={{
              code({ node, className, children, ...props }) {
                const match = /language-(\w+)/.exec(className ?? '');
                const isBlock = match !== null;
                if (isBlock) {
                  return (
                    <SyntaxHighlighter
                      style={dracula}
                      language={match[1]}
                      PreTag="div"
                      customStyle={{
                        margin: '0.75em 0',
                        borderRadius: '4px',
                        fontSize: '12px',
                        background: '#191a21',
                        border: '1px solid #32325a',
                      }}
                    >
                      {String(children).replace(/\n$/, '')}
                    </SyntaxHighlighter>
                  );
                }
                return (
                  <code className={className} {...props}>
                    {children}
                  </code>
                );
              },
            }}
          >
            {message.content}
          </ReactMarkdown>
        </div>
      )}

      {/* Artifact buttons */}
      {artifacts.length > 0 && (
        <div className="flex flex-wrap gap-2 mt-2">
          {artifacts.map((artifact, i) => (
            <button
              key={artifact.id}
              onClick={() => onOpenArtifact?.(artifacts, i)}
              className="flex items-center gap-1.5 px-2.5 py-1 rounded text-[11px] font-mono transition-all"
              style={{
                background: '#12122a',
                border: '1px solid #3a3a6a',
                color: '#9898d8',
              }}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLButtonElement).style.background = '#1a1a3a';
                (e.currentTarget as HTMLButtonElement).style.borderColor = '#6060b0';
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLButtonElement).style.background = '#12122a';
                (e.currentTarget as HTMLButtonElement).style.borderColor = '#3a3a6a';
              }}
            >
              <Layers size={11} />
              <span>{artifact.title}</span>
              <span
                className="text-[9px] px-1 py-0.5 rounded"
                style={{ background: '#1e1e3a', color: '#6060a0' }}
              >
                {artifactLabel(artifact.type)}
              </span>
            </button>
          ))}
        </div>
      )}

      <div className="mt-1.5">
        <span className="text-[10px] font-mono text-argus-textDim">{formatTime(message.timestamp)}</span>
      </div>
    </div>
  );
}
