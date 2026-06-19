'use client';
import { useState } from 'react';
import { motion } from 'framer-motion';
import { X, Copy, Check, ExternalLink, ChevronLeft, ChevronRight } from 'lucide-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { dracula } from 'react-syntax-highlighter/dist/esm/styles/prism';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Artifact } from '../../lib/types';
import { artifactLabel, isVisualArtifact } from '../../lib/artifacts';

export function ArtifactPanel({ artifacts, initialIndex = 0, onClose }: { artifacts: Artifact[]; initialIndex?: number; onClose: () => void }) {
  const [i, setI] = useState(initialIndex);
  const [copied, setCopied] = useState(false);
  const a = artifacts[i]; if (!a) return null;
  const vis = isVisualArtifact(a.type);

  const copy = () => { navigator.clipboard.writeText(a.content); setCopied(true); setTimeout(() => setCopied(false), 1600); };
  const open = () => { const u = URL.createObjectURL(new Blob([a.content], { type: 'text/html' })); window.open(u, '_blank'); };

  return (
    <motion.div initial={{ opacity: 0, x: 30 }} animate={{ opacity: 1, x: 0 }} className="flex flex-col h-full" style={{ background: '#0d0d1a', borderLeft: '1px solid #2a2a4a' }}>
      <div className="flex items-center px-3 py-2 text-[10px] font-mono border-b flex-shrink-0" style={{ borderColor: '#2a2a4a', background: '#0a0a14' }}>
        {artifacts.length > 1 && <div className="flex items-center gap-1 mr-2 text-[#b8b5ac]"><button onClick={() => setI(Math.max(0, i - 1))}><ChevronLeft size={13} /></button><span>{i + 1}/{artifacts.length}</span><button onClick={() => setI(Math.min(artifacts.length - 1, i + 1))}><ChevronRight size={13} /></button></div>}
        <span className="px-1.5 py-px rounded text-[#7878c8] bg-[#1e1e3a]">{artifactLabel(a.type)}</span>
        <span className="flex-1 ml-2 truncate">{a.title}</span>
        <button onClick={copy} className="p-1 text-[#b8b5ac] hover:text-white">{copied ? <Check size={13} className="text-[#39d353]" /> : <Copy size={13} />}</button>
        {a.type === 'html' && <button onClick={open} className="p-1 text-[#b8b5ac] hover:text-white"><ExternalLink size={13} /></button>}
        <button onClick={onClose} className="p-1 text-[#b8b5ac] hover:text-white"><X size={14} /></button>
      </div>
      <div className="flex-1 overflow-hidden">
        {a.type === 'html' && <iframe srcDoc={a.content} sandbox="allow-scripts allow-same-origin" className="w-full h-full border-0" />}
        {a.type === 'svg' && <div className="w-full h-full p-8 flex items-center justify-center bg-[#fafafa]" dangerouslySetInnerHTML={{ __html: a.content }} />}
        {a.type === 'markdown' && <div className="p-6 max-w-3xl mx-auto argus-prose"><ReactMarkdown remarkPlugins={[remarkGfm]}>{a.content}</ReactMarkdown></div>}
        {!vis && <SyntaxHighlighter style={dracula} language={a.type} showLineNumbers customStyle={{ margin: 0, height: '100%', fontSize: 12, background: '#191a21' }}>{a.content}</SyntaxHighlighter>}
      </div>
    </motion.div>
  );
}
