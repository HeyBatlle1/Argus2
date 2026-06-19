'use client';
import { ArgusConnection } from '../lib/connection';
import { ServerMessage } from '../lib/types';

export class RealConnection implements ArgusConnection {
  private ws: WebSocket | null = null;
  constructor(private url: string, private onMsg: (m: ServerMessage)=>void, private onStatus: (c:boolean)=>void) { this._connect(); }
  send(msg: any) { if (this.ws?.readyState===1) this.ws.send(JSON.stringify(msg)); }
  close() { this.ws?.close(); }
  private _connect() {
    this.ws = new WebSocket(this.url);
    this.ws.onopen = () => this.onStatus(true);
    this.ws.onmessage = (e) => { try { const raw=JSON.parse(e.data); if (raw.call_id) raw.callId=raw.call_id; this.onMsg(raw); } catch{} };
    this.ws.onclose = () => this.onStatus(false);
  }
}
