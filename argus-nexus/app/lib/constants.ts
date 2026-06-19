import { EyeState } from './types';

export const EYE_SYMBOLS: Record<EyeState, string> = { watching:'◉', thinking:'◎', executing:'⊙', complete:'✦' };
export const EYE_COLORS: Record<EyeState, string> = { watching:'#39ff9f', thinking:'#f5b800', executing:'#67f6ff', complete:'#ffffff' };

export const WS_URL = process.env.NEXT_PUBLIC_WS_URL ?? null;

export const MEETING_BRIEF = "MONTHLY MEETING — INTERNAL BASELINE: You are Haiku coordinating. Research recent skill usage, memory fidelity, and tool efficiency. Post findings to the intranet.";
