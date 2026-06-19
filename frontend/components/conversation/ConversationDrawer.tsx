'use client';

import { useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Plus, Search } from 'lucide-react';
import { useAgentStore } from '@/hooks/useAgentState';
import { Conversation } from '@/lib/types';

function timeAgo(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const m = Math.floor(diff / 60000);
  if (m < 1) return 'just now';
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  const d = Math.floor(h / 24);
  if (d < 7) return `${d}d ago`;
  return new Date(iso).toLocaleDateString();
}

function modelShort(model: string | null): string {
  if (!model) return '';
  if (model.includes('haiku'))  return 'Haiku';
  if (model.includes('sonnet')) return 'Sonnet';
  if (model.includes('opus'))   return 'Opus';
  if (model.includes('gemini')) return 'Gemini';
  if (model.includes('grok'))   return 'Grok';
  return model.split('/').pop() ?? model;
}

interface RowProps {
  conv: Conversation;
  active: boolean;
  onLoad: (id: string) => void;
}

function ConversationRow({ conv, active, onLoad }: RowProps) {
  return (
    <button
      onClick={() => onLoad(conv.id)}
      className="w-full text-left px-3 py-2.5 rounded-md transition-all group"
      style={{
        background: active ? 'rgba(255,176,0,0.1)' : 'transparent',
        border: active ? '1px solid rgba(255,176,0,0.3)' : '1px solid transparent',
      }}
      onMouseEnter={(e) => {
        if (!active) (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,255,255,0.04)';
      }}
      onMouseLeave={(e) => {
        if (!active) (e.currentTarget as HTMLButtonElement).style.background = 'transparent';
      }}
    >
      <div className="flex items-start justify-between gap-2">
        <span
          className="text-[11px] font-mono leading-tight"
          style={{ color: active ? '#ffb000' : '#c8c8d8', maxWidth: '75%', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden' }}
        >
          {conv.title}
        </span>
        <span className="text-[9px] font-mono flex-shrink-0 pt-0.5" style={{ color: '#5a5a7a' }}>
          {timeAgo(conv.lastActiveAt)}
        </span>
      </div>
      <div className="flex items-center gap-2 mt-1">
        {conv.model && (
          <span className="text-[9px] font-mono px-1 py-px rounded" style={{ background: 'rgba(255,255,255,0.05)', color: '#5a5a7a' }}>
            {modelShort(conv.model)}
          </span>
        )}
        <span className="text-[9px] font-mono" style={{ color: '#3a3a5a' }}>
          {conv.messageCount} {conv.messageCount === 1 ? 'turn' : 'turns'}
        </span>
      </div>
    </button>
  );
}

interface Props {
  open: boolean;
  onClose: () => void;
}

export function ConversationDrawer({ open, onClose }: Props) {
  const conversations = useAgentStore((s) => s.conversations);
  const currentId = useAgentStore((s) => s.currentConversationId);
  const loadConversation = useAgentStore((s) => s.loadConversation);
  const newConversation = useAgentStore((s) => s.newConversation);
  const [query, setQuery] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus search when opened
  useEffect(() => {
    if (open) setTimeout(() => inputRef.current?.focus(), 150);
    else setQuery('');
  }, [open]);

  const filtered = query.trim()
    ? conversations.filter((c) =>
        c.title.toLowerCase().includes(query.toLowerCase())
      )
    : conversations;

  function handleLoad(id: string) {
    loadConversation(id);
    onClose();
  }

  function handleNew() {
    newConversation();
    onClose();
  }

  return (
    <AnimatePresence>
      {open && (
        <>
          {/* Backdrop */}
          <motion.div
            key="backdrop"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="fixed inset-0 z-40"
            style={{ background: 'rgba(0,0,0,0.6)', backdropFilter: 'blur(2px)' }}
            onClick={onClose}
          />

          {/* Drawer */}
          <motion.div
            key="drawer"
            initial={{ x: -320, opacity: 0 }}
            animate={{ x: 0, opacity: 1 }}
            exit={{ x: -320, opacity: 0 }}
            transition={{ duration: 0.22, ease: [0.32, 0.72, 0, 1] }}
            className="fixed top-0 left-0 bottom-0 z-50 flex flex-col"
            style={{
              width: 300,
              background: '#0d0d18',
              borderRight: '1px solid #1e1e32',
              boxShadow: '8px 0 32px rgba(0,0,0,0.6)',
            }}
          >
            {/* Header */}
            <div
              className="flex items-center justify-between px-4 py-3 flex-shrink-0 border-b"
              style={{ borderColor: '#1e1e32', paddingTop: '68px' }}
            >
              <span className="text-[9px] font-mono tracking-widest uppercase text-argus-amber">
                CONVERSATIONS
              </span>
              <div className="flex items-center gap-1">
                <button
                  onClick={handleNew}
                  className="flex items-center gap-1 text-[9px] font-mono px-2 py-1 rounded transition-all cursor-pointer"
                  style={{ background: 'rgba(255,176,0,0.1)', border: '1px solid rgba(255,176,0,0.35)', color: '#ffb000' }}
                  onMouseEnter={(e) => {
                    (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,176,0,0.18)';
                    (e.currentTarget as HTMLButtonElement).style.borderColor = 'rgba(255,176,0,0.6)';
                  }}
                  onMouseLeave={(e) => {
                    (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,176,0,0.1)';
                    (e.currentTarget as HTMLButtonElement).style.borderColor = 'rgba(255,176,0,0.35)';
                  }}
                  title="New conversation"
                >
                  <Plus size={10} />
                  NEW
                </button>
                <button
                  onClick={onClose}
                  className="flex items-center justify-center rounded transition-all cursor-pointer"
                  style={{ width: 26, height: 26, background: 'rgba(255,255,255,0.04)', border: '1px solid #2a2a42', color: '#7a7a9a' }}
                  onMouseEnter={(e) => {
                    (e.currentTarget as HTMLButtonElement).style.color = '#c9a84c';
                    (e.currentTarget as HTMLButtonElement).style.borderColor = 'rgba(201,168,76,0.4)';
                    (e.currentTarget as HTMLButtonElement).style.background = 'rgba(201,168,76,0.07)';
                  }}
                  onMouseLeave={(e) => {
                    (e.currentTarget as HTMLButtonElement).style.color = '#7a7a9a';
                    (e.currentTarget as HTMLButtonElement).style.borderColor = '#2a2a42';
                    (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,255,255,0.04)';
                  }}
                  title="Close"
                >
                  <X size={13} />
                </button>
              </div>
            </div>

            {/* Search */}
            <div className="px-3 py-2 flex-shrink-0 border-b" style={{ borderColor: '#1a1a2e' }}>
              <div className="flex items-center gap-2 px-2 py-1.5 rounded" style={{ background: '#111120', border: '1px solid #1e1e32' }}>
                <Search size={11} style={{ color: '#5a5a7a', flexShrink: 0 }} />
                <input
                  ref={inputRef}
                  type="text"
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder="Search conversations..."
                  className="flex-1 bg-transparent text-[11px] font-mono outline-none"
                  style={{ color: '#c8c8d8' }}
                />
                {query && (
                  <button onClick={() => setQuery('')} className="text-argus-textDim hover:text-argus-amber">
                    <X size={10} />
                  </button>
                )}
              </div>
            </div>

            {/* List */}
            <div className="flex-1 overflow-y-auto px-2 py-2 space-y-0.5">
              {filtered.length === 0 ? (
                <div className="text-center py-12">
                  <p className="text-[10px] font-mono" style={{ color: '#3a3a5a' }}>
                    {query ? 'No matches' : 'No past conversations'}
                  </p>
                  {!query && (
                    <p className="text-[9px] font-mono mt-2" style={{ color: '#2a2a3a' }}>
                      Start chatting to build history
                    </p>
                  )}
                </div>
              ) : (
                filtered.map((conv) => (
                  <ConversationRow
                    key={conv.id}
                    conv={conv}
                    active={conv.id === currentId}
                    onLoad={handleLoad}
                  />
                ))
              )}
            </div>

            {/* Footer */}
            <div className="px-4 py-3 flex-shrink-0 border-t" style={{ borderColor: '#1a1a2e' }}>
              <p className="text-[9px] font-mono" style={{ color: '#2a2a3a' }}>
                {conversations.length} conversation{conversations.length !== 1 ? 's' : ''} stored
              </p>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
