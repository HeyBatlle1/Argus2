'use client';

import { useEffect, useRef } from 'react';
import { EyeState } from '@/lib/types';

interface Props {
  eyeState: EyeState;
  pulse: number;
  size?: number;
  builderMode?: boolean;
}

const EYE_COLORS: Record<EyeState, string> = {
  watching:  '#39d353',
  thinking:  '#f5b800',
  executing: '#67f6ff',
  complete:  '#ffffff',
};

const EYE_RGB: Record<EyeState, [number, number, number]> = {
  watching:  [57, 211, 83],
  thinking:  [245, 184, 0],
  executing: [103, 246, 255],
  complete:  [255, 255, 255],
};

export function NexusCore({ eyeState, pulse, size = 200, builderMode = false }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rafRef = useRef<number | null>(null);
  const liveRef = useRef({ eyeState, pulse, builderMode });

  useEffect(() => { liveRef.current = { eyeState, pulse, builderMode }; }, [eyeState, pulse, builderMode]);

  useEffect(() => {
    const c = canvasRef.current;
    if (!c) return;
    const ctx = c.getContext('2d', { alpha: true })!;
    const W = 480, H = 480;
    c.width = W;
    c.height = H;
    const cx = W / 2, cy = H / 2;
    const EYE_COUNT = 19;
    let t = 0;

    function drawSphere(
      x: number, y: number, radius: number,
      rgb: [number, number, number], alpha: number,
      highlight: { x: number; y: number; strength: number },
    ) {
      const [r, g, b] = rgb;
      const grad = ctx.createRadialGradient(
        x + highlight.x * radius, y + highlight.y * radius, radius * 0.05,
        x, y, radius * 1.05,
      );
      grad.addColorStop(0, `rgba(255, 255, 255, ${(0.92 * alpha).toFixed(3)})`);
      grad.addColorStop(0.18, `rgba(${r}, ${g}, ${b}, ${(0.85 * alpha).toFixed(3)})`);
      grad.addColorStop(0.55, `rgba(${Math.floor(r * 0.5)}, ${Math.floor(g * 0.5)}, ${Math.floor(b * 0.5)}, ${(0.7 * alpha).toFixed(3)})`);
      grad.addColorStop(0.85, `rgba(8, 10, 18, ${(0.5 * alpha).toFixed(3)})`);
      grad.addColorStop(1, 'rgba(5, 5, 8, 0)');
      ctx.fillStyle = grad;
      ctx.beginPath();
      ctx.arc(x, y, radius, 0, Math.PI * 2);
      ctx.fill();

      // Specular glint
      const glint = ctx.createRadialGradient(
        x - radius * 0.32, y - radius * 0.38, 0,
        x - radius * 0.32, y - radius * 0.38, radius * 0.55,
      );
      glint.addColorStop(0, `rgba(255, 255, 255, ${(highlight.strength * alpha).toFixed(3)})`);
      glint.addColorStop(1, 'rgba(255, 255, 255, 0)');
      ctx.fillStyle = glint;
      ctx.beginPath();
      ctx.arc(x, y, radius, 0, Math.PI * 2);
      ctx.fill();
    }

    function draw() {
      const { eyeState: es, pulse: p, builderMode: bm } = liveRef.current;
      const col = bm && es === 'watching' ? '#4ade80' : EYE_COLORS[es];
      const rgb: [number, number, number] = bm && es === 'watching' ? [74, 222, 128] : EYE_RGB[es];

      ctx.clearRect(0, 0, W, H);

      // Ambient halo — volumetric presence
      const haloR = 195 + Math.sin(t * 1.2) * 8 + (p - 3) * 2;
      const halo = ctx.createRadialGradient(cx, cy, haloR * 0.15, cx, cy, haloR);
      halo.addColorStop(0, col + '22');
      halo.addColorStop(0.45, col + '0a');
      halo.addColorStop(1, 'rgba(5, 5, 8, 0)');
      ctx.fillStyle = halo;
      ctx.beginPath();
      ctx.arc(cx, cy, haloR, 0, Math.PI * 2);
      ctx.fill();

      // Outer glass shell (back)
      const shellR = 78 + Math.sin(t * 1.4) * 2;
      drawSphere(cx, cy, shellR, rgb, 0.35, { x: -0.28, y: -0.32, strength: 0.25 });

      // Circuit rings — depth layers
      for (let i = 0; i < 5; i++) {
        const ringR = 88 + i * 20 + Math.sin(t + i * 0.7) * 3;
        const tilt = 0.88 + i * 0.02;
        ctx.save();
        ctx.translate(cx, cy);
        ctx.scale(1, tilt);
        ctx.strokeStyle = `rgba(103, 246, 255, ${0.06 + i * 0.02})`;
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.arc(0, 0, ringR, 0, Math.PI * 2);
        ctx.stroke();
        ctx.restore();
      }

      // Inner molten core
      const coreR = 38 + Math.sin(t * 2.2) * 2 + (p - 3) * 0.5;
      drawSphere(cx, cy, coreR, rgb, 1, { x: -0.35, y: -0.4, strength: 0.55 });

      // Mid luminous layer
      const midR = 58 + Math.sin(t * 1.8) * 3 + (p - 3) * 0.8;
      drawSphere(cx, cy, midR, rgb, 0.75, { x: -0.3, y: -0.35, strength: 0.4 });

      // Rim light — bottom edge catch
      ctx.save();
      ctx.globalCompositeOperation = 'lighter';
      const rim = ctx.createLinearGradient(cx, cy - midR, cx, cy + midR);
      rim.addColorStop(0, 'rgba(255,255,255,0)');
      rim.addColorStop(0.7, 'rgba(255,255,255,0)');
      rim.addColorStop(1, col + '33');
      ctx.strokeStyle = rim;
      ctx.lineWidth = 3;
      ctx.beginPath();
      ctx.arc(cx, cy, midR - 2, 0.15 * Math.PI, 0.85 * Math.PI);
      ctx.stroke();
      ctx.restore();

      // Orbiting eyes — depth-sorted
      const eyes: { angle: number; dist: number; i: number; depth: number }[] = [];
      for (let i = 0; i < EYE_COUNT; i++) {
        const angle = (i / EYE_COUNT) * Math.PI * 2 + t * (0.28 + i * 0.008);
        const dist = 128 + Math.sin(t * 2 + i) * 7 + (p - 4) * 0.7;
        const depth = (Math.sin(angle - 0.5) + 1) / 2;
        eyes.push({ angle, dist, i, depth });
      }
      eyes.sort((a, b) => a.depth - b.depth);

      for (const { angle, dist, i, depth } of eyes) {
        const ex = cx + Math.cos(angle) * dist;
        const ey = cy + Math.sin(angle) * dist * 0.9;
        const sz = (3.2 + depth * 2.8) + Math.sin(t * 3 + i * 1.3) * 0.6;
        const alpha = 0.35 + depth * 0.65;

        drawSphere(ex, ey, sz, rgb, alpha, { x: -0.25, y: -0.3, strength: 0.35 + depth * 0.3 });

        // Pupil — looks toward center
        ctx.fillStyle = `rgba(4, 4, 8, ${(0.7 + depth * 0.3).toFixed(2)})`;
        ctx.beginPath();
        ctx.arc(
          ex - Math.cos(angle) * sz * 0.42,
          ey - Math.sin(angle) * sz * 0.42,
          sz * 0.38, 0, Math.PI * 2,
        );
        ctx.fill();
      }

      // Pulse spokes
      ctx.strokeStyle = col + '20';
      ctx.lineWidth = 1.2;
      for (let i = 0; i < EYE_COUNT; i += 2) {
        const a = (i / EYE_COUNT) * Math.PI * 2 + t * 0.35;
        const len = 95 + Math.sin(t * 2 + i) * 8;
        ctx.beginPath();
        ctx.moveTo(cx, cy);
        ctx.lineTo(cx + Math.cos(a) * len, cy + Math.sin(a) * len * 0.9);
        ctx.stroke();
      }

      t += 0.013;
      rafRef.current = requestAnimationFrame(draw);
    }

    draw();
    return () => { if (rafRef.current) cancelAnimationFrame(rafRef.current); };
  }, []);

  return (
    <div
      className="relative flex items-center justify-center flex-shrink-0 nexus-core-wrap"
      style={{ width: size, height: size }}
    >
      <canvas
        ref={canvasRef}
        className="nexus-core-canvas"
        style={{ width: size, height: size }}
      />
      <div className="absolute text-center pointer-events-none select-none nexus-core-label">
        <div className="font-mono uppercase text-[#67f6ff]/55" style={{ fontSize: 7, letterSpacing: '3px' }}>
          ARGUS
        </div>
        <div className="font-display text-white/92 -mt-0.5" style={{ fontSize: 11, letterSpacing: '-0.5px' }}>
          NEXUS
        </div>
        <div
          className="font-mono uppercase mt-0.5 transition-colors duration-500"
          style={{ fontSize: 6, letterSpacing: '1.5px', color: EYE_COLORS[eyeState] + '99' }}
        >
          {eyeState}
        </div>
      </div>
    </div>
  );
}