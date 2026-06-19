'use client';

import { useEffect, useRef } from 'react';
import { EyeState } from '@/lib/types';

interface Props {
  eyeState?: EyeState;
  /** 0 = still, 1 = full motion. Focus mode passes ~0.35 */
  motionScale?: number;
  /** Grok Build active — greener nebula wash */
  builderMode?: boolean;
  className?: string;
}

interface Star {
  x: number;
  y: number;
  r: number;
  phase: number;
  twinkle: number;
  drift: number;
  layer: 0 | 1 | 2;
  opacity: number;
}

interface Streak {
  x: number;
  y: number;
  len: number;
  speed: number;
  angle: number;
  life: number;
  maxLife: number;
}

const STREAK_TINT: Record<EyeState, string> = {
  watching:  'rgba(180, 220, 255,',
  thinking:  'rgba(245, 184, 0,',
  executing: 'rgba(103, 246, 255,',
  complete:  'rgba(255, 255, 255,',
};

function seedStars(w: number, h: number): Star[] {
  const stars: Star[] = [];
  const counts = [140, 48, 18] as const;
  counts.forEach((count, layer) => {
    for (let i = 0; i < count; i++) {
      stars.push({
        x: Math.random() * w,
        y: Math.random() * h,
        r: layer === 0 ? Math.random() * 0.9 + 0.3 : layer === 1 ? Math.random() * 1.2 + 0.5 : Math.random() * 1.6 + 0.8,
        phase: Math.random() * Math.PI * 2,
        twinkle: 0.4 + Math.random() * 1.2,
        drift: (layer + 1) * (0.08 + Math.random() * 0.18),
        layer: layer as 0 | 1 | 2,
        opacity: layer === 0 ? 0.25 + Math.random() * 0.35 : layer === 1 ? 0.35 + Math.random() * 0.4 : 0.5 + Math.random() * 0.35,
      });
    }
  });
  return stars;
}

function spawnStreak(w: number, h: number): Streak {
  return {
    x: Math.random() * w,
    y: -20 - Math.random() * h * 0.3,
    len: 28 + Math.random() * 72,
    speed: 2.2 + Math.random() * 3.5,
    angle: Math.PI / 2 + (Math.random() - 0.5) * 0.35,
    life: 0,
    maxLife: 80 + Math.random() * 60,
  };
}

export function StarfieldBackground({ eyeState = 'watching', motionScale = 1, builderMode = false, className = '' }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const liveRef = useRef({ eyeState, motionScale, builderMode });

  useEffect(() => { liveRef.current = { eyeState, motionScale, builderMode }; }, [eyeState, motionScale, builderMode]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const context = canvas.getContext('2d', { alpha: true });
    if (!context) return;
    const ctx = context;

    let w = 0;
    let h = 0;
    let stars: Star[] = [];
    let streaks: Streak[] = [];
    let raf = 0;
    let t = 0;
    let streakTimer = 0;
    const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    function resize() {
      const parent = canvas!.parentElement;
      if (!parent) return;
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      w = parent.clientWidth;
      h = parent.clientHeight;
      canvas!.width = Math.floor(w * dpr);
      canvas!.height = Math.floor(h * dpr);
      canvas!.style.width = `${w}px`;
      canvas!.style.height = `${h}px`;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      stars = seedStars(w, h);
      streaks = [];
    }

    const ro = new ResizeObserver(resize);
    ro.observe(canvas.parentElement!);
    resize();

    function draw() {
      const { eyeState: es, motionScale: ms, builderMode: bm } = liveRef.current;
      const motion = reducedMotion ? 0 : ms;
      const speedMul = es === 'executing' ? 1.45 : es === 'thinking' ? 1.15 : 1;

      // Deep void with faint nebula wash
      const neb = ctx.createRadialGradient(w * 0.72, h * 0.18, 0, w * 0.72, h * 0.18, w * 0.55);
      neb.addColorStop(0, bm ? 'rgba(74, 222, 128, 0.06)' : 'rgba(103, 246, 255, 0.04)');
      neb.addColorStop(1, 'transparent');
      const neb2 = ctx.createRadialGradient(w * 0.12, h * 0.82, 0, w * 0.12, h * 0.82, w * 0.45);
      neb2.addColorStop(0, bm ? 'rgba(34, 197, 94, 0.04)' : 'rgba(192, 132, 252, 0.03)');
      neb2.addColorStop(1, 'transparent');

      ctx.fillStyle = '#06060e';
      ctx.fillRect(0, 0, w, h);
      ctx.fillStyle = neb;
      ctx.fillRect(0, 0, w, h);
      ctx.fillStyle = neb2;
      ctx.fillRect(0, 0, w, h);

      // Stars — twinkle + drift (flying past)
      for (const s of stars) {
        if (motion > 0) {
          s.y += s.drift * motion * speedMul;
          if (s.y > h + 4) {
            s.y = -4;
            s.x = Math.random() * w;
          }
        }
        const tw = 0.55 + 0.45 * Math.sin(t * s.twinkle + s.phase);
        const alpha = s.opacity * tw * (s.layer === 2 ? 1.1 : 1);
        ctx.beginPath();
        ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2);
        ctx.fillStyle = s.layer === 2
          ? `rgba(220, 230, 255, ${alpha})`
          : `rgba(180, 195, 230, ${alpha * 0.85})`;
        ctx.fill();
      }

      // Streak particles
      if (motion > 0.2) {
        streakTimer += 1;
        const spawnEvery = es === 'executing' ? 28 : es === 'thinking' ? 48 : 72;
        if (streakTimer % spawnEvery === 0 && streaks.length < 6) {
          streaks.push(spawnStreak(w, h));
        }
        const tint = STREAK_TINT[es];
        streaks = streaks.filter((st) => {
          st.life += 1;
          st.x += Math.cos(st.angle) * st.speed * motion * speedMul;
          st.y += Math.sin(st.angle) * st.speed * motion * speedMul;
          const fade = 1 - st.life / st.maxLife;
          if (fade <= 0) return false;
          const g = ctx.createLinearGradient(st.x, st.y, st.x, st.y + st.len);
          g.addColorStop(0, `${tint}0)`);
          g.addColorStop(0.4, `${tint}${(0.35 * fade).toFixed(3)})`);
          g.addColorStop(1, `${tint}${(0.7 * fade).toFixed(3)})`);
          ctx.strokeStyle = g;
          ctx.lineWidth = 1.2;
          ctx.beginPath();
          ctx.moveTo(st.x, st.y);
          ctx.lineTo(st.x + Math.cos(st.angle) * st.len * 0.15, st.y + st.len);
          ctx.stroke();
          return st.y < h + 40;
        });
      }

      // Soft vignette — keeps text readable
      const vig = ctx.createRadialGradient(w / 2, h / 2, h * 0.2, w / 2, h / 2, Math.max(w, h) * 0.72);
      vig.addColorStop(0, 'rgba(6, 6, 14, 0)');
      vig.addColorStop(1, 'rgba(6, 6, 14, 0.55)');
      ctx.fillStyle = vig;
      ctx.fillRect(0, 0, w, h);

      t += 0.016;
      raf = requestAnimationFrame(draw);
    }

    draw();
    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      className={`starfield-canvas pointer-events-none ${className}`}
      aria-hidden="true"
    />
  );
}