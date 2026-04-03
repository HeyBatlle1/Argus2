'use client';

import { useState, useEffect, useRef } from 'react';
import { Plus } from 'lucide-react';

interface ScheduledItem {
  id: string;
  title: string;
  time?: string;
  risk_level: 'low' | 'medium' | 'high' | 'ghost';
  status: 'pending' | 'running' | 'complete' | 'cancelled';
  created_by: 'bradlee' | 'argus' | 'claude';
}

interface Props {
  date: Date;
  items: ScheduledItem[];
  anchorPos: { x: number; y: number };
  onMouseEnter: () => void;
  onMouseLeave: () => void;
  onAddTask: (title: string) => void;
}

const MONTHS = ['January','February','March','April','May','June','July','August','September','October','November','December'];
const DAYS_FULL = ['Sunday','Monday','Tuesday','Wednesday','Thursday','Friday','Saturday'];

function creatorColor(creator: string): string {
  switch (creator) {
    case 'argus':  return '#f0a500';
    case 'claude': return '#5aafef';
    default:       return '#39d353';
  }
}

export function DayBubble({ date, items, anchorPos, onMouseEnter, onMouseLeave, onAddTask }: Props) {
  const [input, setInput] = useState('');
  const [pos, setPos] = useState({ top: 0, left: 0 });

  useEffect(() => {
    const bubbleH = 200; const bubbleW = 220;
    const top = anchorPos.y > bubbleH + 10 ? anchorPos.y - bubbleH - 8 : anchorPos.y + 26;
    const left = Math.max(8, Math.min(anchorPos.x - bubbleW / 2 + 11, window.innerWidth - bubbleW - 8));
    setPos({ top, left });
  }, [anchorPos]);

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter' && input.trim()) { onAddTask(input.trim()); setInput(''); }
    if (e.key === 'Escape') onMouseLeave();
  }

  return (
    <div
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
      style={{
        position: 'fixed', top: pos.top, left: pos.left, width: '220px',
        background: '#0c0c18', border: '1px solid #2a2a44', borderRadius: '8px',
        padding: '10px', fontFamily: "'JetBrains Mono', monospace", zIndex: 1000,
        boxShadow: '0 8px 24px rgba(0,0,0,0.5)',
        animation: 'bubbleIn 0.12s ease-out',
      }}
    >
      <div style={{ marginBottom: '8px' }}>
        <div style={{ color: '#f0a500', fontSize: '11px', fontWeight: 700 }}>{DAYS_FULL[date.getDay()]}</div>
        <div style={{ color: '#9d9a91', fontSize: '9px', letterSpacing: '0.08em' }}>
          {MONTHS[date.getMonth()]} {date.getDate()}, {date.getFullYear()}
        </div>
      </div>

      {items.length > 0 ? (
        <div style={{ marginBottom: '8px', maxHeight: '100px', overflowY: 'auto' }}>
          {items.map((item) => (
            <div key={item.id} style={{ display: 'flex', alignItems: 'flex-start', gap: '6px', padding: '4px 0', borderBottom: '1px solid rgba(42,42,68,0.5)' }}>
              <div style={{ width: '4px', height: '4px', borderRadius: '50%', background: creatorColor(item.created_by), marginTop: '4px', flexShrink: 0 }} />
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ fontSize: '10px', color: item.status === 'complete' ? '#5a5a7a' : '#f0ede4', textDecoration: item.status === 'complete' ? 'line-through' : 'none', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                  {item.title}
                </div>
                {item.time && <span style={{ fontSize: '8px', color: '#5a5a7a' }}>{item.time}</span>}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div style={{ fontSize: '9px', color: '#5a5a7a', marginBottom: '8px', fontStyle: 'italic' }}>Nothing scheduled</div>
      )}

      <div style={{ display: 'flex', alignItems: 'center', gap: '6px', background: '#1e1e30', border: '1px solid #3a3a5a', borderRadius: '4px', padding: '4px 8px' }}>
        <Plus size={10} style={{ color: '#5a5a7a', flexShrink: 0 }} />
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Add task... (Enter)"
          autoFocus
          style={{ background: 'transparent', border: 'none', outline: 'none', color: '#f0ede4', fontSize: '10px', fontFamily: "'JetBrains Mono', monospace", width: '100%' }}
        />
      </div>

      <style>{`@keyframes bubbleIn { from { opacity: 0; transform: scale(0.94) translateY(4px); } to { opacity: 1; transform: scale(1) translateY(0); } }`}</style>
    </div>
  );
}
