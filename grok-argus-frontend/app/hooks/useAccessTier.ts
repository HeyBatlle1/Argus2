'use client';
import { useAgentStore } from './useAgentState';
import { getModelTier } from '../lib/models';

export function useAccessTier() {
  const m = useAgentStore(s => s.activeModel);
  return getModelTier(m);
}
