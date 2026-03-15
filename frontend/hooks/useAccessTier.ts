'use client';

import { useAgentStore } from './useAgentState';
import { getModelTier } from '@/lib/models';
import { AccessTier } from '@/lib/types';

export function useAccessTier(): AccessTier {
  const activeModel = useAgentStore((s) => s.activeModel);
  return getModelTier(activeModel);
}
