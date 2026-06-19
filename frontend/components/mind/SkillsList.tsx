'use client';

import { Skill } from '@/lib/types';

function timeAgo(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const h = Math.floor(diff / 3600000);
  if (h < 1) return 'just now';
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

interface Props {
  skills: Skill[];
}

export function SkillsList({ skills }: Props) {
  if (skills.length === 0) {
    return (
      <div className="text-center py-4">
        <p className="text-[10px] font-mono" style={{ color: '#3a3a5a' }}>
          No skills learned yet
        </p>
        <p className="text-[9px] font-mono mt-1" style={{ color: '#2a2a3a' }}>
          Skills auto-learn after repeated tool use
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {skills.map((skill) => (
        <div
          key={skill.id}
          className="rounded p-2"
          style={{ background: '#111120', border: '1px solid #1a1a2e' }}
        >
          <div className="flex items-start justify-between gap-2">
            <span className="text-[10px] font-mono font-medium" style={{ color: '#c8c8d8' }}>
              {skill.name}
            </span>
            <div className="flex items-center gap-1 flex-shrink-0">
              <span className="text-[8px] font-mono px-1 rounded" style={{ background: 'rgba(255,176,0,0.1)', color: '#ffb000' }}>
                ×{skill.useCount}
              </span>
            </div>
          </div>
          {skill.description && (
            <p className="text-[9px] font-mono mt-0.5 leading-relaxed" style={{ color: '#5a5a7a' }}>
              {skill.description}
            </p>
          )}
          <div className="flex items-center gap-2 mt-1">
            {skill.toolsUsed.slice(0, 3).map((t) => (
              <span key={t} className="text-[8px] font-mono px-1 py-px rounded" style={{ background: 'rgba(255,255,255,0.04)', color: '#3a3a5a' }}>
                {t}
              </span>
            ))}
            <span className="text-[8px] font-mono ml-auto" style={{ color: '#2a2a3a' }}>
              {timeAgo(skill.learnedAt)}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}
