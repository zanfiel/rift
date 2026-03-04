// ── WebSocket gateway client ──
// Connects to the backend WS, handles identify, reconnect, and event dispatch.

import type { GatewayCommand, GatewayEvent } from './types';

export type GatewayEventHandler = (event: GatewayEvent) => void;

const RECONNECT_DELAYS = [1000, 2000, 4000, 8000, 15000];

export class GatewayClient {
  private ws: WebSocket | null = null;
  private token: string = '';
  private handlers: Set<GatewayEventHandler> = new Set();
  private reconnectAttempt = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private intentionalClose = false;
  private _connected = false;

  get connected(): boolean {
    return this._connected;
  }

  /** Register an event handler. Returns an unsubscribe function. */
  on(handler: GatewayEventHandler): () => void {
    this.handlers.add(handler);
    return () => this.handlers.delete(handler);
  }

  /** Connect to the gateway with the given JWT token. */
  connect(token: string): void {
    this.token = token;
    this.intentionalClose = false;
    this.doConnect();
  }

  /** Cleanly disconnect. */
  disconnect(): void {
    this.intentionalClose = true;
    this._connected = false;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /** Send a command to the gateway. */
  send(command: GatewayCommand): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(command));
    }
  }

  /** Subscribe to channels for a set of servers. */
  subscribe(serverIds: string[]): void {
    this.send({ type: 'Subscribe', data: { server_ids: serverIds } });
  }

  /** Send typing indicator for a channel. */
  sendTyping(channelId: string): void {
    this.send({ type: 'Typing', data: { channel_id: channelId } });
  }

  private doConnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/ws`;
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      // Send Identify immediately
      this.send({ type: 'Identify', data: { token: this.token } });
    };

    this.ws.onmessage = (evt) => {
      try {
        const event = JSON.parse(evt.data) as GatewayEvent;

        if (event.type === 'Ready') {
          this._connected = true;
          this.reconnectAttempt = 0;
        }

        for (const handler of this.handlers) {
          try {
            handler(event);
          } catch (e) {
            console.error('[Gateway] Handler error:', e);
          }
        }
      } catch (e) {
        console.error('[Gateway] Parse error:', e);
      }
    };

    this.ws.onclose = () => {
      this._connected = false;
      if (!this.intentionalClose) {
        this.scheduleReconnect();
      }
    };

    this.ws.onerror = (e) => {
      console.error('[Gateway] WebSocket error:', e);
    };
  }

  private scheduleReconnect(): void {
    const delay = RECONNECT_DELAYS[Math.min(this.reconnectAttempt, RECONNECT_DELAYS.length - 1)];
    this.reconnectAttempt++;
    console.log(`[Gateway] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempt})`);
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.doConnect();
    }, delay);
  }
}

/** Singleton gateway instance */
export const gateway = new GatewayClient();
