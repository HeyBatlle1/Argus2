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
}

export const MODEL_CONFIG: Record<ModelId, ModelConfig> = {
  'claude-haiku': {
    id: 'claude-haiku',
    name: 'Claude Haiku',
    tier: 'royal',
    tierLabel: 'Royal',
    icon: '👑',
    color: '#c9a84c',
    provider: 'anthropic',
    openRouterId: 'anthropic/claude-haiku-4-5',
  },
  'claude-sonnet': {
    id: 'claude-sonnet',
    name: 'Claude Sonnet',
    tier: 'royal',
    tierLabel: 'Royal',
    icon: '👑',
    color: '#c9a84c',
    provider: 'anthropic',
    openRouterId: 'anthropic/claude-sonnet-4-5',
  },
  'claude-opus': {
    id: 'claude-opus',
    name: 'Claude Opus',
    tier: 'royal',
    tierLabel: 'Royal',
    icon: '👑',
    color: '#e8b84b',
    provider: 'anthropic',
    openRouterId: 'anthropic/claude-opus-4-5',
  },
  'grok': {
    id: 'grok',
    name: 'Grok Mini',
    tier: 'allied',
    tierLabel: 'Allied',
    icon: '🛡',
    color: '#4a7c59',
    provider: 'xai',
    openRouterId: 'x-ai/grok-3-mini-beta',
  },
  'gemini-flash': {
    id: 'gemini-flash',
    name: 'Gemini 2.5 Flash',
    tier: 'allied',
    tierLabel: 'Allied',
    icon: '🛡',
    color: '#4a7c59',
    provider: 'google',
    openRouterId: 'google/gemini-2.5-flash',
  },
};

export const MODELS_IN_ORDER: ModelId[] = [
  'claude-haiku',
  'claude-sonnet',
  'claude-opus',
  'grok',
  'gemini-flash',
];

export function getModelTier(model: ModelId): AccessTier {
  return MODEL_CONFIG[model]?.tier ?? 'royal';
}
