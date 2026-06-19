import { ModelId, AccessTier } from './types';

export interface ModelConfig {
  id: ModelId;
  name: string;
  tier: AccessTier;
  tierLabel: string;
  icon: string;
  color: string;
  provider: string;
  openRouterId: string;
  // Grok Build 2 note: specialist framing
  role?: string;
}

export const MODEL_CONFIG: Record<ModelId, ModelConfig> = {
  'claude-haiku': {
    id: 'claude-haiku', name: 'Haiku', tier: 'royal', tierLabel: 'Royal',
    icon: '👑', color: '#c9a84c', provider: 'anthropic',
    openRouterId: '~anthropic/claude-haiku-latest',
    role: 'Fast coordinator',
  },
  'claude-sonnet': {
    id: 'claude-sonnet', name: 'Sonnet', tier: 'royal', tierLabel: 'Royal',
    icon: '👑', color: '#c9a84c', provider: 'anthropic',
    openRouterId: 'anthropic/claude-sonnet-4-6',
  },
  'claude-opus': {
    id: 'claude-opus', name: 'Opus', tier: 'royal', tierLabel: 'Royal',
    icon: '👑', color: '#e8b84b', provider: 'anthropic',
    openRouterId: 'anthropic/claude-opus-4-7',
    role: 'Strategic synthesis',
  },
  'grok': {
    id: 'grok', name: 'Grok 4.3', tier: 'allied', tierLabel: 'Allied',
    icon: '🛡', color: '#39d353', provider: 'xai',
    openRouterId: 'x-ai/grok-4.3',
    role: 'Real-time truth & wit',
  },
  'grok-build': {
    id: 'grok-build', name: 'Grok Build 2', tier: 'allied', tierLabel: 'Allied',
    icon: '🛠', color: '#39d353', provider: 'xai',
    openRouterId: 'x-ai/grok-build-0.1',
    role: 'Implementation specialist — owns the build until it is right',
  },
  'grok-multi': {
    id: 'grok-multi', name: 'Grok 4.20 Multi', tier: 'allied', tierLabel: 'Allied',
    icon: '🛡', color: '#39d353', provider: 'xai',
    openRouterId: 'x-ai/grok-4.20-multi-agent',
    role: 'Orchestration & sequencing',
  },
  'gemini-flash': {
    id: 'gemini-flash', name: 'Gemini Flash', tier: 'allied', tierLabel: 'Allied',
    icon: '🛡', color: '#39d353', provider: 'google',
    openRouterId: 'google/gemini-3.1-flash-lite',
  },
};

export const MODELS_IN_ORDER: ModelId[] = [
  'claude-haiku', 'claude-sonnet', 'claude-opus', 'grok', 'grok-build', 'grok-multi', 'gemini-flash',
];

export function getModelTier(model: ModelId): AccessTier {
  return MODEL_CONFIG[model]?.tier ?? 'royal';
}
