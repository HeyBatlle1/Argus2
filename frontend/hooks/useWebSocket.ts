/**
 * RealConnection — production WebSocket transport to the Rust backend.
 * Token is fetched at runtime from /api/ws-token (not baked into the bundle).
 */

import { ArgusConnection, MessageHandler, StatusHandler } from '@/lib/connection';
import { ClientMessage, ServerMessage } from '@/lib/types';
import { WsConnectOptions, buildWsUrl } from '@/lib/ws-options';

let cachedWsToken: string | null = null;

async function resolveWsToken(): Promise<string | undefined> {
  if (cachedWsToken) return cachedWsToken;

  // Dev fallback — .env.local may still set NEXT_PUBLIC_WS_TOKEN
  const envToken = process.env.NEXT_PUBLIC_WS_TOKEN;
  if (envToken) {
    cachedWsToken = envToken;
    return envToken;
  }

  try {
    const res = await fetch('/api/ws-token', { cache: 'no-store' });
    if (res.ok) {
      const data = (await res.json()) as { token?: string };
      if (data.token) {
        cachedWsToken = data.token;
        return data.token;
      }
    }
  } catch {
    // fall through
  }

  return undefined;
}

export class RealConnection implements ArgusConnection {
  private ws: WebSocket | null = null;
  private readonly url: string;
  private readonly onMessage: MessageHandler;
  private readonly onStatus: StatusHandler;
  private readonly connectOptions?: WsConnectOptions;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private closing = false;

  constructor(
    url: string,
    onMessage: MessageHandler,
    onStatus: StatusHandler,
    connectOptions?: WsConnectOptions,
  ) {
    this.url = url;
    this.onMessage = onMessage;
    this.onStatus = onStatus;
    this.connectOptions = connectOptions;
    void this._connect();
  }

  send(msg: ClientMessage) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    } else {
      console.warn('[argus:ws] send skipped — not connected');
    }
  }

  close() {
    this.closing = true;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.ws?.close();
  }

  private async _connect() {
    if (this.closing) return;

    const token = await resolveWsToken();
    if (!token) {
      console.warn('[argus:ws] no auth token — retrying in 3s');
      this._scheduleReconnect();
      return;
    }

    try {
      const url = buildWsUrl(this.url, token, this.connectOptions);
      this.ws = new WebSocket(url);
    } catch {
      this._scheduleReconnect();
      return;
    }

    this.ws.onopen = () => {
      if (this.reconnectTimer) {
        clearTimeout(this.reconnectTimer);
        this.reconnectTimer = null;
      }
      this.onStatus(true);
    };

    this.ws.onmessage = (event) => {
      try {
        const raw = JSON.parse(event.data as string) as Record<string, unknown>;
        if ('call_id' in raw) raw.callId = raw.call_id;
        this.onMessage(raw as unknown as ServerMessage);
      } catch (e) {
        console.error('[argus:ws] parse error', e);
      }
    };

    this.ws.onerror = () => { /* onclose fires next */ };

    this.ws.onclose = () => {
      this.onStatus(false);
      if (!this.closing) this._scheduleReconnect();
    };
  }

  private _scheduleReconnect() {
    if (this.reconnectTimer || this.closing) return;
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      void this._connect();
    }, 3000);
  }
}

/** Clear cached token after vault reload so the next connect fetches fresh. */
export function clearWsTokenCache() {
  cachedWsToken = null;
}