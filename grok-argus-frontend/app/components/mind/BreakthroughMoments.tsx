'use client';
import { motion } from 'framer-motion';
import { Breakthrough } from '../../lib/types';

export function BreakthroughMoments({ breakthroughs }: { breakthroughs: Breakthrough[] }) {
  if (!breakthroughs.length) return <div className="text-[10px] text-[#b8b5ac]">No milestones yet.</div>;
  return <div className="space-y-1.5">{breakthroughs.map(b => <motion.div key={b.id} whileHover={{ scale: 1.01 }} className="p-2 rounded text-xs" style={{ background: '#16162a', border: '1px solid #32325a', boxShadow: `0 0 ${b.emotionalWeight}px rgba(201,168,76,${b.emotionalWeight / 90})` }}><div className="text-[#f5b800]">✦ {b.title}</div><div className="text-[#b8b5ac] mt-0.5">{b.description}</div></motion.div>)}</div>;
}
