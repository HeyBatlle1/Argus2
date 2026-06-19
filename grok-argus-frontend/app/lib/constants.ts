import { EyeState, ToolName, Tool, ModelId } from './types';

export const EYE_SYMBOLS: Record<EyeState, string> = {
  watching: '◉', thinking: '◎', executing: '⊙', complete: '✦',
};

export const EYE_LABELS: Record<EyeState, string> = {
  watching: 'WATCHING', thinking: 'THINKING', executing: 'EXECUTING', complete: 'COMPLETE',
};

export const EYE_COLORS: Record<EyeState, string> = {
  watching: '#4a7c59', thinking: '#c9a84c', executing: '#4a8fc4', complete: '#ffffff',
};

export const TOOL_BORDER_COLORS = {
  executing: '#4a8fc4', complete: '#4a7c59', error: '#8b1a1a', pending: '#1a1a2e',
};

export const MEMORY_TYPE_COLORS: Record<string, string> = {
  fact: '#c9a84c', preference: '#4a7c59', task: '#4a8fc4', learning: '#8b5cf6',
  relationship: '#ec4899', technical: '#06b6d4', milestone: '#f59e0b',
  breakthrough: '#ef4444', personal_history: '#a78bfa',
};

export const DEFAULT_TOOLS: Tool[] = [
  { name: 'read_file', label: 'read_file', description: 'Read file from filesystem', state: 'idle', callCount: 0 },
  { name: 'write_file', label: 'write_file', description: 'Write file to filesystem', state: 'idle', callCount: 0 },
  { name: 'list_directory', label: 'list_directory', description: 'List directory contents', state: 'idle', callCount: 0 },
  { name: 'shell', label: 'shell', description: 'Execute shell command', state: 'idle', callCount: 0 },
  { name: 'web_search', label: 'web_search', description: 'Search web via Brave', state: 'idle', callCount: 0 },
  { name: 'http_request', label: 'http_request', description: 'Call any HTTP endpoint', state: 'idle', callCount: 0 },
  { name: 'remember', label: 'remember', description: 'Store to persistent memory', state: 'idle', callCount: 0 },
  { name: 'recall', label: 'recall', description: 'Search persistent memory', state: 'idle', callCount: 0 },
  { name: 'forget', label: 'forget', description: 'Remove from memory', state: 'idle', callCount: 0 },
];

export const WS_URL = process.env.NEXT_PUBLIC_WS_URL ?? null;

// Grok Build 2 — default meeting briefs (preserved from Argus1)
export const MEETING_BRIEF_PANE1 =
  'MONTHLY MEETING — INTERNAL HEALTH CHECK: You are Haiku, opening this meeting as coordinator. Three other instances are running right now — Grok on AI landscape, Gemini on tech trends, Opus on synthesis. You will all post to Discord and read each other\'s work. Your job is the honest internal baseline: look at the skill library and pick the 3 capabilities that have seen the most real use, check whether memory reflects what actually happened in recent work, and identify one place where tool use has been inefficient. Report what you find. The meeting needs a real baseline — not a presentation. Post to Discord when done.';

export const MEETING_BRIEFS: Record<2 | 3 | 4, { model: ModelId; brief: string }> = {
  2: {
    model: 'grok-build',
    brief: 'MONTHLY MEETING — AI LANDSCAPE INTEL: You are covering the AI landscape for this meeting. Haiku is running the internal health check, Gemini is covering tech and infrastructure, and Opus will read your findings alongside Gemini\'s for the synthesis. Research the last 30 days: the most significant model releases or capability shifts, any safety or alignment developments worth noting, and one signal that isn\'t mainstream yet but should be watched. Be specific — name models, name organizations, name dates. If a search turns up nothing worth calling out, say so. Post to Discord when done.',
  },
  3: {
    model: 'gemini-flash',
    brief: 'MONTHLY MEETING — TECH & INFRA TRENDS: You are covering the developer and infrastructure landscape for this meeting. Haiku is running the internal health check, Grok is covering AI developments, and Opus will read your findings alongside Grok\'s for the synthesis. Research the last 30 days: what moved in tooling, cloud, or open-source that actually matters; any security or supply chain issues worth watching; one project or library gaining real traction and why. If something is overhyped, say so. Opus is reading this — give them something real to work with. Post to Discord when done.',
  },
  4: {
    model: 'claude-opus',
    brief: 'MONTHLY MEETING — STRATEGIC SYNTHESIS: Grok just posted the AI landscape briefing and Gemini just posted the tech and infrastructure briefing — both are in Discord. Read what they actually wrote. Your job is genuine synthesis: find the real thread between the two reports if one exists, name the single most important thing this system should be paying attention to this month and why, and give a clear recommendation for the next 30 days. If the two reports connect in a meaningful way, show it. If they don\'t, say so — a forced connection is worse than an honest gap. Post to Discord when done.',
  },
};
