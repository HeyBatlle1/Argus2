'use client';
import { Skill } from '../../lib/types';

export function SkillsList({ skills }: { skills: Skill[] }) {
  if (!skills.length) return <div className="text-[10px] text-[#b8b5ac] py-2">No skills learned yet.<div className="text-[9px] mt-1 text-[#3a3a5a]">Auto-populated after repeated tool use.</div></div>;
  return <div className="space-y-1">{skills.slice(0, 6).map(s => <div key={s.id} className="p-1.5 rounded text-[10px]" style={{ background: '#111120', border: '1px solid #1a1a2e' }}><div className="flex justify-between"><span>{s.name}</span><span className="text-[#ffb000] text-[9px]">×{s.useCount}</span></div></div>)}</div>;
}
