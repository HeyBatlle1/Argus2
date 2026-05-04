'use client';

import { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, ExternalLink, Copy, Check, ChevronLeft, ChevronRight } from 'lucide-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { dracula } from 'react-syntax-highlighter/dist/esm/styles/prism';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Artifact } from '@/lib/types';
import { artifactLabel, isVisualArtifact } from '@/lib/artifacts';

interface Props {
  artifacts: Artifact[];
  initialIndex?: number;
  onClose: () => void;
}

export function ArtifactPanel({ artifacts, initialIndex = 0, onClose }: Props) {
  const [activeIdx, setActiveIdx] = useState(initialIndex);
  const [copied, setCopied] = useState(false);
  const iframeRef = useRef<HTMLIFrameElement>(null);

  const artifact = artifacts[activeIdx];
  if (!artifact) return null;

  const isVisual = isVisualArtifact(artifact.type);

  function handleCopy() {
    navigator.clipboard.writeText(artifact.content).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1800);
    });
  }

  function handleOpenPage() {
    const blob = new Blob([artifact.content], { type: 'text/html' });
    const url = URL.createObjectURL(blob);
    window.open(url, '_blank');
  }

  function renderContent() {
    switch (artifact.type) {
      case 'html':
        return (
          <iframe
            ref={iframeRef}
            srcDoc={artifact.content}
            sandbox="allow-scripts allow-same-origin"
            className="w-full h-full border-0"
            title={artifact.title}
          />
        );

      case 'svg':
        return (
          <div
            className="w-full h-full flex items-center justify-center p-6 overflow-auto"
            style={{ background: '#fafafa' }}
            dangerouslySetInnerHTML={{ __html: artifact.content }}
          />
        );

      case 'markdown':
        return (
          <div className="w-full h-full overflow-auto p-6" style={{ background: '#0d0d1a' }}>
            <div className="argus-prose text-argus-text text-sm leading-relaxed max-w-3xl mx-auto">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {artifact.content}
              </ReactMarkdown>
            </div>
          </div>
        );

      default:
        return (
          <div className="w-full h-full overflow-auto" style={{ background: '#191a21' }}>
            <SyntaxHighlighter
              style={dracula}
              language={artifact.type}
              showLineNumbers
              customStyle={{
                margin: 0,
                borderRadius: 0,
                fontSize: '13px',
                minHeight: '100%',
                background: '#191a21',
              }}
            >
              {artifact.content}
            </SyntaxHighlighter>
          </div>
        );
    }
  }

  return (
    <motion.div
      initial={{ opacity: 0, x: 40 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 40 }}
      transition={{ duration: 0.22, ease: 'easeOut' }}
      className="flex flex-col h-full"
      style={{
        background: '#0d0d1a',
        borderLeft: '1px solid #2a2a4a',
      }}
    >
      {/* Header */}
      <div
        className="flex items-center gap-2 px-4 py-3 flex-shrink-0"
        style={{ borderBottom: '1px solid #2a2a4a', background: '#0a0a14' }}
      >
        {/* Artifact nav — only shown when multiple */}
        {artifacts.length > 1 && (
          <div className="flex items-center gap-1 mr-1">
            <button
              onClick={() => setActiveIdx((i) => Math.max(0, i - 1))}
              disabled={activeIdx === 0}
              className="p-0.5 text-argus-textDim hover:text-argus-text disabled:opacity-30 transition-colors"
            >
              <ChevronLeft size={14} />
            </button>
            <span className="text-[10px] font-mono text-argus-textDim">
              {activeIdx + 1}/{artifacts.length}
            </span>
            <button
              onClick={() => setActiveIdx((i) => Math.min(artifacts.length - 1, i + 1))}
              disabled={activeIdx === artifacts.length - 1}
              className="p-0.5 text-argus-textDim hover:text-argus-text disabled:opacity-30 transition-colors"
            >
              <ChevronRight size={14} />
            </button>
          </div>
        )}

        {/* Type badge */}
        <span
          className="text-[9px] font-mono tracking-widest px-1.5 py-0.5 rounded"
          style={{ background: '#1e1e3a', color: '#7878c8', border: '1px solid #3a3a6a' }}
        >
          {artifactLabel(artifact.type)}
        </span>

        {/* Title */}
        <span className="flex-1 text-[12px] font-mono text-argus-text truncate">
          {artifact.title}
        </span>

        {/* Actions */}
        <div className="flex items-center gap-2 flex-shrink-0">
          <button
            onClick={handleCopy}
            className="flex items-center gap-1 text-[10px] font-mono text-argus-textDim hover:text-argus-text transition-colors"
            title="Copy content"
          >
            {copied ? <Check size={12} className="text-green-400" /> : <Copy size={12} />}
          </button>

          {artifact.type === 'html' && (
            <button
              onClick={handleOpenPage}
              className="flex items-center gap-1 text-[10px] font-mono text-argus-textDim hover:text-argus-text transition-colors"
              title="Open in new tab"
            >
              <ExternalLink size={12} />
            </button>
          )}

          <button
            onClick={onClose}
            className="p-0.5 text-argus-textDim hover:text-argus-text transition-colors"
            title="Close"
          >
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Artifact type tabs (when multiple) */}
      {artifacts.length > 1 && (
        <div
          className="flex gap-0 flex-shrink-0 overflow-x-auto"
          style={{ borderBottom: '1px solid #2a2a4a' }}
        >
          {artifacts.map((a, i) => (
            <button
              key={a.id}
              onClick={() => setActiveIdx(i)}
              className="px-3 py-1.5 text-[10px] font-mono whitespace-nowrap transition-colors"
              style={{
                color: i === activeIdx ? '#9898d8' : '#666688',
                borderBottom: i === activeIdx ? '1px solid #9898d8' : '1px solid transparent',
                background: i === activeIdx ? '#12121f' : 'transparent',
              }}
            >
              {a.title.length > 20 ? a.title.slice(0, 20) + '…' : a.title}
            </button>
          ))}
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        <AnimatePresence mode="wait">
          <motion.div
            key={artifact.id}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.15 }}
            className="w-full h-full"
          >
            {renderContent()}
          </motion.div>
        </AnimatePresence>
      </div>
    </motion.div>
  );
}
