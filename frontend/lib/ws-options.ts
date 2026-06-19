import { ModelId } from './types';

export type WsSurface = 'web' | 'council';

export interface WsConnectOptions {
  model?: ModelId;
  surface?: WsSurface;
}

/** Append model + surface query params to an authenticated WS URL. */
export function buildWsUrl(baseUrl: string, token: string | undefined, opts?: WsConnectOptions): string {
  const params = new URLSearchParams();
  if (token) params.set('token', token);
  if (opts?.model) params.set('model', opts.model);
  if (opts?.surface) params.set('surface', opts.surface);
  const qs = params.toString();
  return qs ? `${baseUrl}?${qs}` : baseUrl;
}