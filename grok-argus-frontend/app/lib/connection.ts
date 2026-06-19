// Connection contract preserved exactly.
import { ClientMessage, ServerMessage } from './types';

export type MessageHandler = (msg: ServerMessage) => void;
export type StatusHandler = (connected: boolean) => void;

export interface ArgusConnection {
  send(msg: ClientMessage): void;
  close(): void;
}
