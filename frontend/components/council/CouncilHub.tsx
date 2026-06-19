'use client';

import {
  forwardRef, useCallback, useEffect, useImperativeHandle,
  useRef, useState,
} from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Send, BookMarked } from 'lucide-react';
import { RealConnection } from '@/hooks/useWebSocket';
import { useAgentStore } from '@/hooks/useAgentState';
import { ServerMessage, ModelId, EyeState } from '@/lib/types';
import { WS_URL } from '@/lib/constants';
import { parseArtifacts } from '@/lib/artifacts';

// ─── Model orb config ─────────────────────────────────────────────────────────

const ORB_CFG: Record<string, { color: string; label: string; role: string }> = {
  'claude-haiku':  { color: '#c9a84c', label: 'HAIKU',      role: 'Operations' },
  'claude-sonnet': { color: '#e8b84b', label: 'SONNET',     role: 'Core'       },
  'claude-opus':   { color: '#c084fc', label: 'OPUS',       role: 'Synthesis'  },
  'grok':          { color: '#39d353', label: 'GROK',       role: 'Analyst'    },
  'grok-build':    { color: '#4ade80', label: 'GROK BUILD', role: 'Builder'    },
  'grok-multi':    { color: '#34d399', label: 'GROK MULTI', role: 'Multi'      },
  'gemini-flash':  { color: '#67f6ff', label: 'GEMINI',     role: 'Intel'      },
};

// ─── Single-orb WS connection + canvas ───────────────────────────────────────

export interface OrbHandle {
  send: (text: string) => void;
}

interface OrbProps {
  model: ModelId;
  openingBrief?: string;
  isSynthesis?: boolean;
  size?: number;
  onComplete?: (text: string) => void;
}

export const CouncilOrb = forwardRef<OrbHandle, OrbProps>(function CouncilOrb(
  { model, openingBrief, isSynthesis = false, size = 160, onComplete },
  ref,
) {
  const cfg = ORB_CFG[model] ?? { color: '#c9a84c', label: model, role: '' };

  const [connected,       setConnected]       = useState(false);
  const [eyeState,        setEyeState]        = useState<EyeState>('watching');
  const [isStreaming,     setIsStreaming]      = useState(false);
  const [streamText,      setStreamText]      = useState('');
  const [lastResponse,    setLastResponse]    = useState('');

  const wsRef      = useRef<RealConnection | null>(null);
  const canvasRef  = useRef<HTMLCanvasElement>(null);
  const rafRef     = useRef<number | null>(null);

  // live refs so canvas closure sees current state
  const liveRef = useRef({ eyeState, isStreaming, connected });
  useEffect(() => { liveRef.current = { eyeState, isStreaming, connected }; },
    [eyeState, isStreaming, connected]);

  // ── WS message handler ────────────────────────────────────────────────────
  const handleMsg = useCallback((msg: ServerMessage) => {
    switch (msg.type) {
      case 'connected':
        setConnected(true);
        break;
      case 'thinking':
        setEyeState('thinking');
        setIsStreaming(true);
        setStreamText('');
        break;
      case 'tool_call':
        setEyeState('executing');
        break;
      case 'tool_result':
        break;
      case 'response_chunk':
        setStreamText(prev => prev + msg.content);
        break;
      case 'response_complete': {
        const { cleanText } = parseArtifacts(msg.content);
        setLastResponse(cleanText);
        setStreamText('');
        setIsStreaming(false);
        setEyeState('complete');
        onComplete?.(cleanText);
        setTimeout(() => setEyeState('watching'), 2200);
        break;
      }
      case 'status':
        setEyeState(msg.eye_state);
        break;
      case 'error':
        setIsStreaming(false);
        setEyeState('watching');
        break;
    }
  }, [onComplete]);

  // ── Connect on mount ──────────────────────────────────────────────────────
  useEffect(() => {
    if (!WS_URL) return;
    const conn = new RealConnection(WS_URL, handleMsg, setConnected, {
      model,
      surface: 'council',
    });
    wsRef.current = conn;
    return () => { conn.close(); wsRef.current = null; };
  }, [handleMsg]);

  // ── Opening brief ─────────────────────────────────────────────────────────
  const briefSent = useRef(false);
  useEffect(() => {
    if (!connected || !openingBrief || briefSent.current) return;
    briefSent.current = true;
    setTimeout(() => {
      wsRef.current?.send({ type: 'user_message', content: openingBrief });
    }, 400 + Math.random() * 600);
  }, [connected, openingBrief]);

  // ── Expose send ───────────────────────────────────────────────────────────
  useImperativeHandle(ref, () => ({
    send: (text) => wsRef.current?.send({ type: 'user_message', content: text }),
  }));

  // ── Canvas orb animation ──────────────────────────────────────────────────
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d')!;
    canvas.width  = size;
    canvas.height = size;
    const cx = size / 2, cy = size / 2;
    let t = 0;

    function draw() {
      ctx.clearRect(0, 0, size, size);
      const { eyeState: es, isStreaming: streaming, connected: conn } = liveRef.current;

      const speed     = es === 'executing' ? 4.0 : es === 'thinking' ? 2.5 : 1.0;
      const intensity = streaming ? 1.0 : conn ? 0.65 : 0.2;
      const baseR     = size * 0.26 + Math.sin(t * speed * 0.7) * (streaming ? size * 0.035 : size * 0.012);

      // Outer ambient glow
      const ambient = ctx.createRadialGradient(cx, cy, baseR * 0.6, cx, cy, baseR * 2.2);
      ambient.addColorStop(0, cfg.color + '18');
      ambient.addColorStop(1, 'transparent');
      ctx.fillStyle = ambient;
      ctx.beginPath(); ctx.arc(cx, cy, baseR * 2.2, 0, Math.PI * 2); ctx.fill();

      // Core orb gradient
      const grad = ctx.createRadialGradient(cx, cy, baseR * 0.08, cx, cy, baseR);
      grad.addColorStop(0, `rgba(255,255,255,${intensity * 0.95})`);
      grad.addColorStop(0.35, cfg.color + Math.round(intensity * 200).toString(16).padStart(2, '0'));
      grad.addColorStop(0.75, cfg.color + Math.round(intensity * 80).toString(16).padStart(2, '0'));
      grad.addColorStop(1, 'rgba(5,5,8,0)');
      ctx.fillStyle = grad;
      ctx.beginPath(); ctx.arc(cx, cy, baseR, 0, Math.PI * 2); ctx.fill();

      // Orbit ring
      ctx.beginPath();
      ctx.arc(cx, cy, baseR * 1.35, 0, Math.PI * 2);
      ctx.strokeStyle = cfg.color + (conn ? '30' : '10');
      ctx.lineWidth = 1;
      ctx.stroke();

      // Streaming particles
      if (streaming || es === 'thinking' || es === 'executing') {
        const count = es === 'executing' ? 3 : 6;
        for (let i = 0; i < count; i++) {
          const a    = (i / count) * Math.PI * 2 + t * (es === 'executing' ? 5 : 3);
          const dist = baseR * 1.3 + Math.sin(t * 2.5 + i * 1.1) * (size * 0.025);
          const px   = cx + Math.cos(a) * dist;
          const py   = cy + Math.sin(a) * dist * 0.9;

          ctx.beginPath(); ctx.arc(px, py, size * 0.018, 0, Math.PI * 2);
          ctx.fillStyle = cfg.color;
          ctx.fill();

          ctx.beginPath(); ctx.arc(px, py, size * 0.008, 0, Math.PI * 2);
          ctx.fillStyle = 'rgba(255,255,255,0.9)';
          ctx.fill();
        }
      }

      // Synthesis extra arc (only for Opus)
      if (isSynthesis) {
        const arcLen = (Math.sin(t * 0.5) * 0.5 + 0.5) * Math.PI * 2;
        ctx.beginPath();
        ctx.arc(cx, cy, baseR * 1.55, -Math.PI / 2, -Math.PI / 2 + arcLen);
        ctx.strokeStyle = cfg.color + '70';
        ctx.lineWidth   = 2;
        ctx.stroke();
      }

      t += 0.016;
      rafRef.current = requestAnimationFrame(draw);
    }

    draw();
    return () => { if (rafRef.current) cancelAnimationFrame(rafRef.current); };
  }, [cfg.color, isSynthesis, size]);

  const displayText = isStreaming ? streamText : lastResponse;

  return (
    <div className="flex flex-col items-center" style={{ width: size + 32 }}>
      {/* Canvas orb */}
      <div className="relative">
        <canvas ref={canvasRef} style={{ width: size, height: size }} />
        <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none select-none">
          <span className="text-[7px] font-mono tracking-[2px]" style={{ color: cfg.color + '99' }}>
            {cfg.role.toUpperCase()}
          </span>
          <span className="text-[11px] font-mono font-bold tracking-widest leading-tight" style={{ color: cfg.color }}>
            {cfg.label}
          </span>
          <span className="text-[7px] font-mono mt-0.5" style={{ color: '#4a4a6a' }}>
            {!connected ? 'LINKING' : eyeState.toUpperCase()}
          </span>
        </div>
      </div>

      {/* Streaming / last response snippet */}
      <div style={{ width: size, minHeight: 36 }} className="text-center mt-1 px-1">
        <AnimatePresence mode="wait">
          {displayText && (
            <motion.p
              key={isStreaming ? 'stream' : 'last'}
              initial={{ opacity: 0, y: 4 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className="text-[9px] leading-relaxed line-clamp-3"
              style={{ color: cfg.color + 'bb' }}
            >
              {displayText.slice(0, 140)}
              {isStreaming && (
                <span className="inline-block w-1 h-2.5 ml-0.5 align-bottom animate-pulse" style={{ background: cfg.color }} />
              )}
            </motion.p>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
});

// ─── Synthesis card (must-hear insight) ──────────────────────────────────────

function SynthesisCard({ text, index }: { text: string; index: number }) {
  return (
    <motion.div
      initial={{ opacity: 0, x: -16, scale: 0.96 }}
      animate={{ opacity: 1, x: 0, scale: 1 }}
      transition={{ duration: 0.35, delay: index * 0.12, ease: 'easeOut' }}
      className="flex items-start gap-2 px-3 py-2 rounded-lg"
      style={{ background: 'rgba(192,132,252,0.07)', border: '1px solid rgba(192,132,252,0.25)' }}
    >
      <span className="text-[10px] mt-px shrink-0" style={{ color: '#c084fc' }}>⚡</span>
      <span className="text-[10px] font-mono leading-relaxed" style={{ color: '#d4b8fc' }}>{text}</span>
    </motion.div>
  );
}

// ─── SVG particle line ────────────────────────────────────────────────────────

interface Particle {
  id:    number;
  x1: number; y1: number;
  x2: number; y2: number;
  color: string;
}

// ─── Council Hub ──────────────────────────────────────────────────────────────

interface CouncilBrief {
  model: ModelId;
  brief: string;
}

interface CouncilHubProps {
  briefs: CouncilBrief[];   // exactly 4
  onClose: () => void;
}

export function CouncilHub({ briefs, onClose }: CouncilHubProps) {
  const sendMessage = useAgentStore((s) => s.sendMessage);
  const [inputValue,     setInputValue]     = useState('');
  const [particles,      setParticles]      = useState<Particle[]>([]);
  const [synthesisCards, setSynthesisCards] = useState<string[]>([]);
  const [synopsis,       setSynopsis]       = useState('');
  const [synthesisSaved, setSynthesisSaved] = useState(false);

  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef     = useRef<HTMLTextAreaElement>(null);
  const orbDivRefs   = useRef<Array<HTMLDivElement | null>>([null, null, null, null]);
  const orbHandles   = useRef<Array<OrbHandle | null>>([null, null, null, null]);

  // ── Extract must-hear insights from synthesis model (Opus) ────────────────
  function handleSynthesisComplete(text: string) {
    const sentences = text
      .split(/(?<=[.!?])\s+/)
      .map(s => s.trim())
      .filter(s => s.length > 50 && s.length < 220);

    // Surface up to 3 key sentences
    const cards = sentences.slice(0, 3);
    if (cards.length > 0) {
      setSynthesisCards(cards);
    }
    // Full synopsis is Opus's complete response
    setSynopsis(text.slice(0, 400));
  }

  // ── Broadcast + particle animation ───────────────────────────────────────
  function broadcast() {
    const text = inputValue.trim();
    if (!text) return;
    setInputValue('');
    setSynthesisCards([]);
    setSynopsis('');

    // Kick off particle animation
    const containerEl = containerRef.current;
    const inputEl     = inputRef.current;
    if (containerEl && inputEl) {
      const cRect = containerEl.getBoundingClientRect();
      const iRect = inputEl.getBoundingClientRect();
      const x1 = iRect.left + iRect.width / 2 - cRect.left;
      const y1 = iRect.top  + iRect.height / 2 - cRect.top;

      const newParticles: Particle[] = orbDivRefs.current
        .map((div, i) => {
          if (!div) return null;
          const r = div.getBoundingClientRect();
          return {
            id:    Date.now() + i,
            x1, y1,
            x2: r.left + r.width  / 2 - cRect.left,
            y2: r.top  + r.height / 2 - cRect.top,
            color: ORB_CFG[briefs[i]?.model]?.color ?? '#c9a84c',
          };
        })
        .filter(Boolean) as Particle[];

      setParticles(newParticles);
      setTimeout(() => setParticles([]), 750);
    }

    // Send to all 4 orbs
    orbHandles.current.forEach(h => h?.send(text));
  }

  function onKeyDown(e: React.KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); broadcast(); }
  }

  function commitSynthesisToMemory() {
    const body = synopsis || synthesisCards.join('\n\n');
    if (!body.trim()) return;
    const date = new Date().toISOString().slice(0, 10);
    sendMessage(
      `[COUNCIL SYNTHESIS — commit to memory]\n\n` +
      `Call remember once: type "learning", importance 8, content summarizing this council synthesis. ` +
      `Subject line: "[COUNCIL ${date}] Monthly synthesis".\n\n${body}`,
    );
    setSynthesisSaved(true);
    setTimeout(() => setSynthesisSaved(false), 4000);
  }

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
      className="fixed inset-0 z-50 flex flex-col"
      style={{ background: '#050508' }}
      ref={containerRef}
    >
      {/* Top bar */}
      <div
        className="flex-shrink-0 h-14 flex items-center justify-between px-6 border-b"
        style={{ borderColor: '#1a1a28', background: '#07070c' }}
      >
        <div className="flex items-center gap-3">
          <div
            className="w-2 h-2 rounded-full animate-pulse"
            style={{ background: '#c084fc', boxShadow: '0 0 8px #c084fc' }}
          />
          <span className="font-mono text-[11px] tracking-[0.3em] uppercase" style={{ color: '#c084fc' }}>
            Council Chamber
          </span>
          <span className="text-[9px] font-mono" style={{ color: '#3a3a5a' }}>
            · all models present · broadcast mode
          </span>
        </div>

        <div className="flex items-center gap-2">
          {(synopsis || synthesisCards.length > 0) && (
            <button
              onClick={commitSynthesisToMemory}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded font-mono text-[9px] tracking-widest uppercase transition-all panel-action-btn"
              style={{
                border: '1px solid rgba(192,132,252,0.4)',
                color: synthesisSaved ? '#39d353' : '#c084fc',
                background: synthesisSaved ? 'rgba(57,211,83,0.08)' : 'rgba(192,132,252,0.06)',
              }}
            >
              <BookMarked size={10} />
              {synthesisSaved ? 'COMMITTED' : 'SAVE TO MEMORY'}
            </button>
          )}
          <button
            onClick={onClose}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded font-mono text-[9px] tracking-widest uppercase panel-action-btn"
            style={{ border: '1px solid #2a2a3a', color: '#5a5a7a' }}
          >
            <X size={10} /> Dismiss
          </button>
        </div>
      </div>

      {/* Body */}
      <div className="flex-1 overflow-hidden flex flex-col">
        {/* Orb grid */}
        <div className="flex-1 flex items-center justify-center px-8 pt-6">
          <div className="flex flex-col gap-6 items-center w-full max-w-4xl">

            {/* Top row: first 3 models */}
            <div className="flex gap-8 justify-center">
              {briefs.slice(0, 3).map((b, i) => (
                <div
                  key={b.model}
                  ref={el => { orbDivRefs.current[i] = el; }}
                >
                  <CouncilOrb
                    ref={el => { orbHandles.current[i] = el; }}
                    model={b.model}
                    openingBrief={b.brief}
                    size={164}
                  />
                </div>
              ))}
            </div>

            {/* Bottom: Synthesis orb (Opus) centered + larger */}
            {briefs[3] && (
              <div className="flex flex-col items-center gap-3">
                <div
                  className="text-[8px] font-mono tracking-[3px] uppercase"
                  style={{ color: '#c084fc66' }}
                >
                  ── Synthesis ──
                </div>
                <div ref={el => { orbDivRefs.current[3] = el; }}>
                  <CouncilOrb
                    ref={el => { orbHandles.current[3] = el; }}
                    model={briefs[3].model}
                    openingBrief={briefs[3].brief}
                    isSynthesis
                    size={196}
                    onComplete={handleSynthesisComplete}
                  />
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Synthesis panel */}
        <AnimatePresence>
          {(synthesisCards.length > 0 || synopsis) && (
            <motion.div
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: 'auto' }}
              exit={{ opacity: 0, height: 0 }}
              transition={{ duration: 0.3 }}
              className="mx-8 mb-3 rounded-xl overflow-hidden"
              style={{ border: '1px solid rgba(192,132,252,0.2)', background: 'rgba(192,132,252,0.04)' }}
            >
              <div className="p-4">
                {synthesisCards.length > 0 && (
                  <div className="flex flex-col gap-2 mb-3">
                    {synthesisCards.map((card, i) => (
                      <SynthesisCard key={i} text={card} index={i} />
                    ))}
                  </div>
                )}
                {synopsis && (
                  <motion.p
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    transition={{ delay: 0.4 }}
                    className="text-[10px] font-mono leading-relaxed"
                    style={{ color: '#7a6a9a' }}
                  >
                    {synopsis}{synopsis.length >= 400 ? '…' : ''}
                  </motion.p>
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Council input */}
        <div
          className="flex-shrink-0 px-8 pb-6"
          style={{ borderTop: '1px solid #1a1a28', paddingTop: 16 }}
        >
          <div
            className="flex items-end gap-3 rounded-2xl px-4 py-3 transition-all"
            style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid #2a2a3a' }}
            onFocus={() => {}}
          >
            <textarea
              ref={inputRef}
              value={inputValue}
              onChange={e => setInputValue(e.target.value)}
              onKeyDown={onKeyDown}
              placeholder="Speak to the Council — broadcasts to all four simultaneously…"
              rows={1}
              className="flex-1 bg-transparent resize-none outline-none leading-relaxed text-sm"
              style={{ color: '#e2e2ea', fontFamily: "'Instrument Sans', sans-serif", maxHeight: 96 }}
              onInput={e => {
                const el = e.currentTarget;
                el.style.height = 'auto';
                el.style.height = Math.min(el.scrollHeight, 96) + 'px';
              }}
            />
            <button
              onClick={broadcast}
              disabled={!inputValue.trim()}
              className="flex-shrink-0 flex items-center justify-center rounded-xl transition-all"
              style={{
                width: 36, height: 36,
                background: inputValue.trim() ? 'rgba(192,132,252,0.18)' : 'rgba(255,255,255,0.04)',
                border: inputValue.trim() ? '1px solid rgba(192,132,252,0.5)' : '1px solid #2a2a3a',
                color: inputValue.trim() ? '#c084fc' : '#3a3a5a',
                cursor: inputValue.trim() ? 'pointer' : 'not-allowed',
              }}
            >
              <Send size={13} />
            </button>
          </div>
          <p className="text-center text-[8px] font-mono mt-2" style={{ color: '#2a2a3a' }}>
            ENTER TO SEND · MESSAGE REACHES ALL FOUR SIMULTANEOUSLY
          </p>
        </div>
      </div>

      {/* SVG particle overlay */}
      <svg
        className="absolute inset-0 pointer-events-none"
        style={{ width: '100%', height: '100%', zIndex: 99 }}
      >
        <defs>
          {particles.map(p => (
            <marker key={`m-${p.id}`} id={`dot-${p.id}`} markerWidth="6" markerHeight="6" refX="3" refY="3">
              <circle cx="3" cy="3" r="3" fill={p.color} />
            </marker>
          ))}
        </defs>
        {particles.map(p => (
          <g key={p.id}>
            {/* Fading trail */}
            <motion.line
              initial={{ x1: p.x1, y1: p.y1, x2: p.x1, y2: p.y1, opacity: 0.5 }}
              animate={{ x2: p.x2, y2: p.y2, opacity: 0 }}
              transition={{ duration: 0.65, ease: 'easeIn' }}
              stroke={p.color}
              strokeWidth={1}
            />
            {/* Particle head */}
            <motion.circle
              initial={{ cx: p.x1, cy: p.y1, r: 4, opacity: 1 }}
              animate={{ cx: p.x2, cy: p.y2, r: 2, opacity: 0 }}
              transition={{ duration: 0.65, ease: 'easeIn' }}
              fill={p.color}
            />
          </g>
        ))}
      </svg>

      {/* Ambient background gradient */}
      <div
        className="absolute inset-0 pointer-events-none"
        style={{
          background: 'radial-gradient(ellipse 70% 50% at 50% 40%, rgba(192,132,252,0.04) 0%, transparent 70%)',
          zIndex: 0,
        }}
      />
    </motion.div>
  );
}
