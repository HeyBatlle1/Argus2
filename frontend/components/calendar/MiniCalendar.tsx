'use client';

import { useState, useRef } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { DayBubble } from './DayBubble';

interface ScheduledItem {
  id: string;
  title: string;
  time?: string;
  risk_level: 'low' | 'medium' | 'high' | 'ghost';
  status: 'pending' | 'running' | 'complete' | 'cancelled';
  created_by: 'bradlee' | 'argus' | 'claude';
}

interface CalendarData {
  [dateKey: string]: ScheduledItem[];
}

interface Props {
  data?: CalendarData;
  onAddTask?: (date: Date, title: string) => void;
}

const DAYS = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];
const MONTHS = [
  'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
  'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
];

function dateKey(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

function creatorDot(creator: string): string {
  switch (creator) {
    case 'argus':  return '#f0a500';
    case 'claude': return '#5aafef';
    default:       return '#39d353';
  }
}

export function MiniCalendar({ data = {}, onAddTask }: Props) {
  const today = new Date();
  const [current, setCurrent] = useState({ year: today.getFullYear(), month: today.getMonth() });
  const [hoveredDay, setHoveredDay] = useState<Date | null>(null);
  const [bubblePos, setBubblePos] = useState({ x: 0, y: 0 });
  const hoverTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const leaveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const firstDay = new Date(current.year, current.month, 1).getDay();
  const daysInMonth = new Date(current.year, current.month + 1, 0).getDate();

  function prevMonth() {
    setCurrent(c => c.month === 0 ? { year: c.year - 1, month: 11 } : { ...c, month: c.month - 1 });
  }
  function nextMonth() {
    setCurrent(c => c.month === 11 ? { year: c.year + 1, month: 0 } : { ...c, month: c.month + 1 });
  }

  function handleDayEnter(e: React.MouseEvent, date: Date) {
    if (leaveTimer.current) clearTimeout(leaveTimer.current);
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    hoverTimer.current = setTimeout(() => {
      setBubblePos({ x: rect.left, y: rect.top });
      setHoveredDay(date);
    }, 120);
  }

  function handleDayLeave() {
    if (hoverTimer.current) clearTimeout(hoverTimer.current);
    leaveTimer.current = setTimeout(() => setHoveredDay(null), 200);
  }

  const cells: (Date | null)[] = [
    ...Array(firstDay).fill(null),
    ...Array.from({ length: daysInMonth }, (_, i) => new Date(current.year, current.month, i + 1)),
  ];
  while (cells.length % 7 !== 0) cells.push(null);

  const isToday = (d: Date) =>
    d.getDate() === today.getDate() &&
    d.getMonth() === today.getMonth() &&
    d.getFullYear() === today.getFullYear();

  return (
    <div style={{ position: 'relative' }}>
      <div style={{
        width: '192px', background: '#12121e', border: '1px solid #2a2a44',
        borderRadius: '6px', padding: '8px',
        fontFamily: "'JetBrains Mono', monospace", userSelect: 'none',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '6px' }}>
          <button onClick={prevMonth} style={{ background: 'none', border: 'none', color: '#9d9a91', cursor: 'pointer', padding: '0 2px' }}>
            <ChevronLeft size={11} />
          </button>
          <span style={{ color: '#f0a500', fontSize: '10px', fontWeight: 600, letterSpacing: '0.1em' }}>
            {MONTHS[current.month]} {current.year}
          </span>
          <button onClick={nextMonth} style={{ background: 'none', border: 'none', color: '#9d9a91', cursor: 'pointer', padding: '0 2px' }}>
            <ChevronRight size={11} />
          </button>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', marginBottom: '3px' }}>
          {DAYS.map((d, i) => (
            <div key={i} style={{ textAlign: 'center', fontSize: '8px', color: '#5a5a7a', fontWeight: 600 }}>{d}</div>
          ))}
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(7, 1fr)', gap: '1px' }}>
          {cells.map((date, i) => {
            if (!date) return <div key={i} style={{ height: '22px' }} />;
            const key = dateKey(date);
            const items = data[key] || [];
            const todayFlag = isToday(date);
            const isHovered = hoveredDay && dateKey(hoveredDay) === key;
            return (
              <div key={i}
                onMouseEnter={(e) => handleDayEnter(e, date)}
                onMouseLeave={handleDayLeave}
                style={{
                  height: '22px', display: 'flex', flexDirection: 'column',
                  alignItems: 'center', justifyContent: 'center', borderRadius: '3px', cursor: 'pointer',
                  background: todayFlag ? 'rgba(240,165,0,0.15)' : isHovered ? 'rgba(240,165,0,0.07)' : 'transparent',
                  border: todayFlag ? '1px solid rgba(240,165,0,0.4)' : '1px solid transparent',
                  transition: 'background 0.1s ease',
                }}
              >
                <span style={{ fontSize: '9px', color: todayFlag ? '#f0a500' : '#d4d0c8', fontWeight: todayFlag ? 700 : 400 }}>
                  {date.getDate()}
                </span>
                {items.length > 0 && (
                  <div style={{ display: 'flex', gap: '1px', marginTop: '1px' }}>
                    {items.slice(0, 3).map((item, j) => (
                      <div key={j} style={{ width: '3px', height: '3px', borderRadius: '50%', background: creatorDot(item.created_by), opacity: item.status === 'complete' ? 0.4 : 1 }} />
                    ))}
                  </div>
                )}
              </div>
            );
          })}
        </div>

        <div style={{ display: 'flex', gap: '8px', marginTop: '6px', paddingTop: '5px', borderTop: '1px solid #1a1a2e' }}>
          {[{ color: '#f0a500', label: 'Argus' }, { color: '#5aafef', label: 'Claude' }, { color: '#39d353', label: 'You' }].map(({ color, label }) => (
            <div key={label} style={{ display: 'flex', alignItems: 'center', gap: '3px' }}>
              <div style={{ width: '4px', height: '4px', borderRadius: '50%', background: color }} />
              <span style={{ fontSize: '7px', color: '#5a5a7a' }}>{label}</span>
            </div>
          ))}
        </div>
      </div>

      {hoveredDay && (
        <DayBubble
          date={hoveredDay}
          items={data[dateKey(hoveredDay)] || []}
          anchorPos={bubblePos}
          onMouseEnter={() => { if (leaveTimer.current) clearTimeout(leaveTimer.current); }}
          onMouseLeave={() => { leaveTimer.current = setTimeout(() => setHoveredDay(null), 150); }}
          onAddTask={(title) => { onAddTask?.(hoveredDay, title); setHoveredDay(null); }}
        />
      )}
    </div>
  );
}
