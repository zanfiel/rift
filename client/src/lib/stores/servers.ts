// ── Server store — manages server list, selected server/channel, members ──

import type { Server, ServerWithDetails, Channel, MemberWithUser, Role } from '../types';
import * as api from '../api';
import { gateway } from '../gateway';

type Subscriber = () => void;

let _servers: Server[] = [];
let _currentServer: ServerWithDetails | null = null;
let _currentChannelId: string | null = null;
let _members: MemberWithUser[] = [];
let _loading = false;
const subs: Set<Subscriber> = new Set();

function notify() {
  for (const fn of subs) fn();
}

export const servers = {
  subscribe(fn: Subscriber): () => void {
    subs.add(fn);
    return () => subs.delete(fn);
  },

  get list(): Server[] { return _servers; },
  get current(): ServerWithDetails | null { return _currentServer; },
  get currentChannelId(): string | null { return _currentChannelId; },
  get members(): MemberWithUser[] { return _members; },
  get loading(): boolean { return _loading; },

  get currentChannel(): Channel | null {
    if (!_currentServer || !_currentChannelId) return null;
    return _currentServer.channels.find(c => c.id === _currentChannelId) ?? null;
  },

  /** Load server list */
  async load(): Promise<void> {
    _loading = true;
    notify();
    try {
      _servers = await api.listServers();
      // Subscribe to all server events via gateway
      if (_servers.length > 0) {
        gateway.subscribe(_servers.map(s => s.id));
      }
    } catch (e) {
      console.error('Failed to load servers:', e);
    }
    _loading = false;
    notify();
  },

  /** Select a server — loads full details + members */
  async select(serverId: string): Promise<void> {
    _loading = true;
    notify();
    try {
      _currentServer = await api.getServer(serverId);
      _members = await api.listMembers(serverId);
      // Auto-select first channel if none selected
      if (_currentServer.channels.length > 0) {
        _currentChannelId = _currentServer.channels[0].id;
      } else {
        _currentChannelId = null;
      }
    } catch (e) {
      console.error('Failed to load server:', e);
    }
    _loading = false;
    notify();
  },

  /** Select a channel within the current server */
  selectChannel(channelId: string): void {
    _currentChannelId = channelId;
    notify();
  },

  /** Create a new server */
  async create(name: string, description?: string): Promise<Server> {
    const server = await api.createServer(name, description);
    _servers = [..._servers, server];
    notify();
    return server;
  },

  /** Create a new channel in the current server */
  async createChannel(name: string, topic?: string): Promise<Channel | null> {
    if (!_currentServer) return null;
    const channel = await api.createChannel(_currentServer.server.id, name, topic);
    _currentServer = {
      ..._currentServer,
      channels: [..._currentServer.channels, channel],
    };
    notify();
    return channel;
  },

  /** Delete a channel */
  async deleteChannel(channelId: string): Promise<void> {
    await api.deleteChannel(channelId);
    if (_currentServer) {
      _currentServer = {
        ..._currentServer,
        channels: _currentServer.channels.filter(c => c.id !== channelId),
      };
      if (_currentChannelId === channelId) {
        _currentChannelId = _currentServer.channels[0]?.id ?? null;
      }
    }
    notify();
  },

  /** Handle gateway events that affect server state */
  handleEvent(event: { type: string; data: any }): void {
    switch (event.type) {
      case 'ChannelCreate': {
        const d = event.data;
        if (_currentServer && d.server_id === _currentServer.server.id) {
          const newCh: Channel = {
            id: d.id,
            server_id: d.server_id,
            name: d.name,
            channel_type: d.channel_type,
            topic: null,
            position: _currentServer.channels.length,
            created_at: new Date().toISOString(),
          };
          _currentServer = {
            ..._currentServer,
            channels: [..._currentServer.channels, newCh],
          };
          notify();
        }
        break;
      }
      case 'ChannelDelete': {
        const d = event.data;
        if (_currentServer && d.server_id === _currentServer.server.id) {
          _currentServer = {
            ..._currentServer,
            channels: _currentServer.channels.filter(c => c.id !== d.id),
          };
          if (_currentChannelId === d.id) {
            _currentChannelId = _currentServer.channels[0]?.id ?? null;
          }
          notify();
        }
        break;
      }
      case 'MemberJoin': {
        const d = event.data;
        if (_currentServer && d.server_id === _currentServer.server.id) {
          // Reload members — we don't have full user info from the event
          api.listMembers(d.server_id).then(m => {
            _members = m;
            notify();
          });
        }
        break;
      }
      case 'MemberLeave': {
        const d = event.data;
        if (_currentServer && d.server_id === _currentServer.server.id) {
          _members = _members.filter(m => m.user_id !== d.user_id);
          notify();
        }
        break;
      }
      case 'RoleCreate': {
        const d = event.data;
        if (_currentServer && d.server_id === _currentServer.server.id) {
          _currentServer = {
            ..._currentServer,
            roles: [..._currentServer.roles, d.role],
          };
          notify();
        }
        break;
      }
      case 'RoleUpdate': {
        const d = event.data;
        if (_currentServer && d.server_id === _currentServer.server.id) {
          _currentServer = {
            ..._currentServer,
            roles: _currentServer.roles.map(r => r.id === d.role.id ? d.role : r),
          };
          notify();
        }
        break;
      }
      case 'RoleDelete': {
        const d = event.data;
        if (_currentServer && d.server_id === _currentServer.server.id) {
          _currentServer = {
            ..._currentServer,
            roles: _currentServer.roles.filter(r => r.id !== d.role_id),
          };
          notify();
        }
        break;
      }
    }
  },

  /** Clear state on logout */
  clear(): void {
    _servers = [];
    _currentServer = null;
    _currentChannelId = null;
    _members = [];
    notify();
  },
};
