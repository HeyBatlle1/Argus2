'use client';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { dracula } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { Layers } from 'lucide-react';
import { Message, Artifact } from '../../lib/types';
import { artifactLabel } from '../../lib/artifacts';

export function ArgusMessage({ message, onOpenArtifact }: { message: Message; onOpenArtifact?: (a: Artifact[], i: number) => void }) {
  if (!message.content && !message.artifacts?.length) return null;
  const arts = message.artifacts ?? [];
  return (
    <div>
      {message.content && (
        <div className="argus-prose text-sm leading-relaxed">
          <ReactMarkdown remarkPlugins={[remarkGfm]} components={{
            code({ className, children, ...p }: any) {
              const match = /language-(\w+)/.exec(className || '');
              if (match) return <SyntaxHighlighter style={dracula} language={match[1]} customStyle={{ margin: '0.7em 0', borderRadius: 4, fontSize: 12, background: '#191a21', border: '1px solid #32325a' }}>{String(children).replace(/\n$/, '')}</SyntaxHighlighter>;
              return <code className={className} {...p}>{children}</code>;
            }
          }}>{message.content}</ReactMarkdown>
        </div>
      )}
      {message.grokEvidence && <div className="mt-1"><span className="truth-badge">CONF {Math.round(message.grokEvidence.confidence * 100)}%</span> {message.grokEvidence.notes && <span className="text-[10px] text-[#6a6a8a] ml-1.5">{message.grokEvidence.notes}</span>}</div>}
      {arts.length > 0 && (
        <div className="flex flex-wrap gap-1.5 mt-2">
          {arts.map((a, i) => (
            <button key={a.id} onClick={() => onOpenArtifact?.(arts, i)} className="flex items-center gap-1 px-2 py-0.5 text-[10px] rounded font-mono border transition" style={{ background: '#12122a', borderColor: '#3a3a6a', color: '#9898d8' }}>
              <Layers size={10} /> {a.title} <span className="bg-[#1e1e3a] px-1 text-[9px] text-[#6060a0]">{artifactLabel(a.type)}</span>
            </button>
          ))}
        </div>
      )}
      <div className="mt-1 text-[10px] text-[#b8b5ac] font-mono">{message.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</div>
    </div>
  );
}
