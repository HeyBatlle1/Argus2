/**
 * ArgusConnection — common interface for both real (WebSocket) and dev (mock) transports.
 * The Zustand store only talks to this interface and never imports from lib/dev/.
 */

import { ClientMessage, ServerMessage } from './types';

export type MessageHandler = (msg: ServerMessage) => void;
export type StatusHandler = (connected: boolean) => void;

export interface ArgusConnection {
  send(msg: ClientMessage): void;
  close(): void;
}
