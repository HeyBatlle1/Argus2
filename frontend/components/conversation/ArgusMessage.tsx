'use client';

import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { dracula } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { Message } from '@/lib/types';

interface Props {
  message: Message;
}

function formatTime(d: Date) {
  return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
}

export function ArgusMessage({ message }: Props) {
  if (!message.content) return null;

  return (
    <div className="animate-fade-in">
      <div className="argus-prose text-argus-text text-sm leading-relaxed">
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
                      border: '1px solid #1a1a2e',
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
      <div className="mt-1.5">
        <span className="text-[10px] font-mono text-argus-textDim">{formatTime(message.timestamp)}</span>
      </div>
    </div>
  );
}
