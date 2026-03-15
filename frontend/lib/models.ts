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
  'claude-opus': {
    id: 'claude-opus',
    name: 'Claude Opus',
    tier: 'royal',
    tierLabel: 'Royal',
    icon: '👑',
    color: '#c9a84c',
    provider: 'anthropic',
    openRouterId: 'anthropic/claude-opus-4-6',
  },
  'grok': {
    id: 'grok',
    name: 'Grok 4.1 Fast',
    tier: 'allied',
    tierLabel: 'Allied',
    icon: '🛡',
    color: '#4a7c59',
    provider: 'xai',
    openRouterId: 'x-ai/grok-4.1-fast',
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
  'claude-opus',
  'grok',
  'gemini-flash',
];

export function getModelTier(model: ModelId): AccessTier {
  return MODEL_CONFIG[model].tier;
}
