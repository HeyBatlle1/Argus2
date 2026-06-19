import { ModelId } from './types';

/** Grok Build is the primary coding agent — full atmosphere + tool powers. */
export const PRIMARY_CODER: ModelId = 'grok-build';

export function isBuilderModel(model: ModelId): boolean {
  return model === PRIMARY_CODER;
}

export const BUILDER_THEME = {
  accent: '#4ade80',
  accentDim: '#22c55e',
  glow: 'rgba(74, 222, 128, 0.35)',
  surface: 'rgba(74, 222, 128, 0.06)',
  border: 'rgba(74, 222, 128, 0.35)',
} as const;

/** One-click coding briefs — Grok Build's native language. */
export const BUILDER_QUICK_PROMPTS: { id: string; label: string; prompt: string }[] = [
  {
    id: 'ship',
    label: 'Ship',
    prompt:
      'Read the relevant files first, then implement the change. Run what you need to verify. Commit with a clear message when done.',
  },
  {
    id: 'debug',
    label: 'Debug',
    prompt:
      'Something is broken. Trace the failure from symptom to root cause — read logs and code, form a hypothesis, test it, fix it.',
  },
  {
    id: 'review',
    label: 'Review',
    prompt:
      'Review the current diff and recent changes. Flag bugs, security issues, and anything that will bite us later. Be direct.',
  },
  {
    id: 'architect',
    label: 'Design',
    prompt:
      'Propose a clean architecture for what we are building. Name files, data flow, and tradeoffs. Then implement the first slice.',
  },
];