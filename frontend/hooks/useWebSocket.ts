/**
 * RealConnection — production WebSocket transport to the Rust backend.
 * Implements ArgusConnection. Auto-reconnects on drop.
 */

import { ArgusConnection, MessageHandler, StatusHandler } from '@/lib/connection';
import { ClientMessage, ServerMessage } from '@/lib/types';

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
      console.warn('[argus:ws] send skipped — not connected');
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
      this.ws = new WebSocket(this.url);
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
        // Normalize snake_case from Rust → camelCase expected by the store
        if ('call_id' in raw) raw.callId = raw.call_id;
        this.onMessage(raw as unknown as ServerMessage);
      } catch (e) {
        console.error('[argus:ws] parse error', e);
      }
    };

    this.ws.onerror = () => { /* onclose fires next, handles reconnect */ };

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
