// ── Message store — manages messages for the active channel ──

import type { MessageWithAuthor } from '../types';
import * as api from '../api';
import { gateway } from '../gateway';

type Subscriber = () => void;

let _messages: MessageWithAuthor[] = [];
let _channelId: string | null = null;
let _loading = false;
let _hasMore = true;
let _typingUsers: Map<string, { username: string; timeout: ReturnType<typeof setTimeout> }> = new Map();
const subs: Set<Subscriber> = new Set();

function notify() {
  for (const fn of subs) fn();
}

// Throttle typing indicator — at most once per 3 seconds
let lastTypingSent = 0;

export const messages = {
  subscribe(fn: Subscriber): () => void {
    subs.add(fn);
    return () => subs.delete(fn);
  },

  get list(): MessageWithAuthor[] { return _messages; },
  get channelId(): string | null { return _channelId; },
  get loading(): boolean { return _loading; },
  get hasMore(): boolean { return _hasMore; },
  get typingUsers(): string[] {
    return Array.from(_typingUsers.values()).map(t => t.username);
  },

  /** Load messages for a channel (replaces current) */
  async load(channelId: string): Promise<void> {
    _channelId = channelId;
    _loading = true;
    _hasMore = true;
    notify();

    try {
      const msgs = await api.listMessages(channelId, { limit: 50 });
      _messages = msgs.reverse(); // API returns newest first, we want oldest first
    } catch (e) {
      console.error('Failed to load messages:', e);
      _messages = [];
    }

    _loading = false;
    // Subscribe the gateway to this channel's events
    gateway.send({ type: 'Subscribe', data: { server_ids: [] } }); // server sub handled elsewhere
    notify();
  },

  /** Load older messages (infinite scroll up) */
  async loadOlder(): Promise<void> {
    if (!_channelId || _loading || !_hasMore || _messages.length === 0) return;
    _loading = true;
    notify();

    try {
      const oldest = _messages[0];
      const older = await api.listMessages(_channelId, { before: oldest.id, limit: 50 });
      if (older.length < 50) _hasMore = false;
      _messages = [...older.reverse(), ..._messages];
    } catch (e) {
      console.error('Failed to load older messages:', e);
    }

    _loading = false;
    notify();
  },

  /** Send a message to the current channel */
  async send(content: string): Promise<void> {
    if (!_channelId) return;
    try {
      // The server will broadcast MessageCreate via WS — we add it there
      await api.sendMessage(_channelId, content);
    } catch (e) {
      console.error('Failed to send message:', e);
      throw e;
    }
  },

  /** Edit a message */
  async edit(messageId: string, content: string): Promise<void> {
    if (!_channelId) return;
    try {
      await api.editMessage(_channelId, messageId, content);
    } catch (e) {
      console.error('Failed to edit message:', e);
      throw e;
    }
  },

  /** Delete a message */
  async delete(messageId: string): Promise<void> {
    if (!_channelId) return;
    try {
      await api.deleteMessage(_channelId, messageId);
    } catch (e) {
      console.error('Failed to delete message:', e);
      throw e;
    }
  },

  /** Send typing indicator (throttled) */
  sendTyping(): void {
    const now = Date.now();
    if (now - lastTypingSent > 3000 && _channelId) {
      gateway.sendTyping(_channelId);
      lastTypingSent = now;
    }
  },

  /** Handle gateway events that affect messages */
  handleEvent(event: { type: string; data: any }): void {
    switch (event.type) {
      case 'MessageCreate': {
        const d = event.data;
        if (d.channel_id === _channelId) {
          const msg: MessageWithAuthor = {
            id: d.id,
            channel_id: d.channel_id,
            author_id: d.author_id,
            content: d.content,
            edited_at: null,
            created_at: d.created_at,
            author_username: d.author_username,
            author_display_name: d.author_display_name,
            author_avatar_url: d.author_avatar_url,
          };
          _messages = [..._messages, msg];
          // Clear typing for this user
          _typingUsers.delete(d.author_id);
          notify();
        }
        break;
      }
      case 'MessageUpdate': {
        const d = event.data;
        if (d.channel_id === _channelId) {
          _messages = _messages.map(m =>
            m.id === d.id ? { ...m, content: d.content, edited_at: d.edited_at } : m
          );
          notify();
        }
        break;
      }
      case 'MessageDelete': {
        const d = event.data;
        if (d.channel_id === _channelId) {
          _messages = _messages.filter(m => m.id !== d.id);
          notify();
        }
        break;
      }
      case 'TypingStart': {
        const d = event.data;
        if (d.channel_id === _channelId) {
          // Clear existing timeout
          const existing = _typingUsers.get(d.user_id);
          if (existing) clearTimeout(existing.timeout);
          // Set new timeout to clear after 5s
          const timeout = setTimeout(() => {
            _typingUsers.delete(d.user_id);
            notify();
          }, 5000);
          _typingUsers.set(d.user_id, { username: d.username, timeout });
          notify();
        }
        break;
      }
    }
  },

  /** Clear state */
  clear(): void {
    _messages = [];
    _channelId = null;
    _hasMore = true;
    _typingUsers.clear();
    notify();
  },
};
