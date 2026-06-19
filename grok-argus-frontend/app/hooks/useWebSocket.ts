/**
 * Real WS connection — identical behavior and normalization as the original Argus1.
 * Auto-reconnect. Snake_case → camelCase for callId.
 */
'use client';
import { ArgusConnection, MessageHandler, StatusHandler } from '../lib/connection';
import { ClientMessage, ServerMessage } from '../lib/types';

export class RealConnection implements ArgusConnection {
  private ws: WebSocket | null = null;
  private readonly url: string;
  private readonly onMessage: MessageHandler;
  private readonly onStatus: StatusHandler;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private closing = false;

  constructor(url: string, onMessage: MessageHandler, onStatus: StatusHandler) {
    this.url = url;
    this.onMessage = onMessage;
    this.onStatus = onStatus;
    this._connect();
  }

  send(msg: ClientMessage) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    } else {
      console.warn('[grok-argus:ws] send skipped — not connected');
    }
  }

  close() {
    this.closing = true;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.ws?.close();
  }

  private _connect() {
    this.closing = false;
    try {
      const token = process.env.NEXT_PUBLIC_WS_TOKEN;
      const url = token ? `${this.url}?token=${encodeURIComponent(token)}` : this.url;
      this.ws = new WebSocket(url);
    } catch {
      this._scheduleReconnect();
      return;
    }

    this.ws.onopen = () => {
      if (this.reconnectTimer) { clearTimeout(this.reconnectTimer); this.reconnectTimer = null; }
      this.onStatus(true);
    };

    this.ws.onmessage = (event) => {
      try {
        const raw = JSON.parse(event.data as string) as Record<string, unknown>;
        if ('call_id' in raw) (raw as any).callId = raw.call_id;
        this.onMessage(raw as unknown as ServerMessage);
      } catch (e) {
        console.error('[grok-argus:ws] parse error', e);
      }
    };

    this.ws.onerror = () => {};
    this.ws.onclose = () => {
      this.onStatus(false);
      if (!this.closing) this._scheduleReconnect();
    };
  }

  private _scheduleReconnect() {
    if (this.reconnectTimer || this.closing) return;
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this._connect();
    }, 3000);
  }
}
