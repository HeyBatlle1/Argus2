'use client';
import { PartnershipDynamic } from '../../lib/types';

export function PartnershipDynamics({ dynamics }: { dynamics: PartnershipDynamic[] }) {
  if (!dynamics.length) return <div className="text-[10px] text-[#b8b5ac]">No patterns.</div>;
  return <div className="space-y-1.5">{dynamics.map(d => <div key={d.id} className="p-2 rounded text-xs" style={{ background: '#16162a', border: '1px solid #32325a' }}><div className="font-medium">{d.patternName}</div><div className="text-[#b8b5ac] mt-0.5 text-[10px]">{d.description}</div></div>)}</div>;
}
