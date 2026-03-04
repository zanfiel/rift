// ── Typed API client for all REST endpoints ──

import type {
  AuthResponse,
  Channel,
  DmChannel,
  DmMessage,
  Invite,
  MemberWithUser,
  MessageWithAuthor,
  PublicUser,
  Server,
  ServerWithDetails,
} from './types';

class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
    this.name = 'ApiError';
  }
}

function getToken(): string | null {
  return localStorage.getItem('token');
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const headers: Record<string, string> = {};
  const token = getToken();
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  if (body !== undefined) {
    headers['Content-Type'] = 'application/json';
  }

  const res = await fetch(path, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });

  if (!res.ok) {
    let msg = res.statusText;
    try {
      const err = await res.json();
      msg = err.error || err.message || msg;
    } catch { /* ignore */ }
    throw new ApiError(res.status, msg);
  }

  // 204 No Content
  if (res.status === 204) return undefined as T;

  return res.json() as Promise<T>;
}

// ── Auth ──

export function register(username: string, email: string, password: string, display_name?: string) {
  return request<AuthResponse>('POST', '/api/auth/register', { username, email, password, display_name });
}

export function login(username: string, password: string) {
  return request<AuthResponse>('POST', '/api/auth/login', { username, password });
}

export function refresh(refresh_token: string) {
  return request<AuthResponse>('POST', '/api/auth/refresh', { refresh_token });
}

export function logout() {
  return request<{ ok: boolean }>('POST', '/api/auth/logout');
}

// ── Users ──

export function getMe() {
  return request<PublicUser>('GET', '/api/users/@me');
}

export function updateMe(data: { display_name?: string; about?: string }) {
  return request<PublicUser>('PATCH', '/api/users/@me', data);
}

export function getUser(userId: string) {
  return request<PublicUser>('GET', `/api/users/${userId}`);
}

// ── Servers ──

export function listServers() {
  return request<Server[]>('GET', '/api/servers');
}

export function createServer(name: string, description?: string) {
  return request<Server>('POST', '/api/servers', { name, description });
}

export function getServer(serverId: string) {
  return request<ServerWithDetails>('GET', `/api/servers/${serverId}`);
}

export function updateServer(serverId: string, data: { name?: string; description?: string }) {
  return request<Server>('PATCH', `/api/servers/${serverId}`, data);
}

export function deleteServer(serverId: string) {
  return request<void>('DELETE', `/api/servers/${serverId}`);
}

// ── Members ──

export function listMembers(serverId: string) {
  return request<MemberWithUser[]>('GET', `/api/servers/${serverId}/members`);
}

export function kickMember(serverId: string, userId: string) {
  return request<void>('DELETE', `/api/servers/${serverId}/members/${userId}`);
}

// ── Channels ──

export function listChannels(serverId: string) {
  return request<Channel[]>('GET', `/api/servers/${serverId}/channels`);
}

export function createChannel(serverId: string, name: string, topic?: string, channel_type?: string) {
  return request<Channel>('POST', `/api/servers/${serverId}/channels`, { name, topic, channel_type });
}

export function updateChannel(channelId: string, data: { name?: string; topic?: string; position?: number }) {
  return request<Channel>('PATCH', `/api/channels/${channelId}`, data);
}

export function deleteChannel(channelId: string) {
  return request<void>('DELETE', `/api/channels/${channelId}`);
}

// ── Messages ──

export function listMessages(channelId: string, opts?: { before?: string; after?: string; limit?: number }) {
  const params = new URLSearchParams();
  if (opts?.before) params.set('before', opts.before);
  if (opts?.after) params.set('after', opts.after);
  if (opts?.limit) params.set('limit', String(opts.limit));
  const qs = params.toString();
  return request<MessageWithAuthor[]>('GET', `/api/channels/${channelId}/messages${qs ? '?' + qs : ''}`);
}

export function sendMessage(channelId: string, content: string) {
  return request<MessageWithAuthor>('POST', `/api/channels/${channelId}/messages`, { content });
}

export function editMessage(channelId: string, messageId: string, content: string) {
  return request<MessageWithAuthor>('PATCH', `/api/channels/${channelId}/messages/${messageId}`, { content });
}

export function deleteMessage(channelId: string, messageId: string) {
  return request<void>('DELETE', `/api/channels/${channelId}/messages/${messageId}`);
}

// ── Invites ──

export function listInvites(serverId: string) {
  return request<Invite[]>('GET', `/api/servers/${serverId}/invites`);
}

export function createInvite(serverId: string, max_uses?: number, expires_in_hours?: number) {
  return request<Invite>('POST', `/api/servers/${serverId}/invites`, { max_uses, expires_in_hours });
}

export function deleteInvite(serverId: string, code: string) {
  return request<void>('DELETE', `/api/servers/${serverId}/invites/${code}`);
}

export function joinInvite(code: string) {
  return request<Server>('POST', `/api/invites/${code}/join`);
}

// ── DMs ──

export function listDms() {
  return request<DmChannel[]>('GET', '/api/users/@me/dms');
}

export function createDm(recipientId: string) {
  return request<DmChannel>('POST', '/api/users/@me/dms', { recipient_id: recipientId });
}

export function listDmMessages(dmId: string, opts?: { before?: string; limit?: number }) {
  const params = new URLSearchParams();
  if (opts?.before) params.set('before', opts.before);
  if (opts?.limit) params.set('limit', String(opts.limit));
  const qs = params.toString();
  return request<DmMessage[]>('GET', `/api/dms/${dmId}/messages${qs ? '?' + qs : ''}`);
}

export function sendDmMessage(dmId: string, content: string) {
  return request<DmMessage>('POST', `/api/dms/${dmId}/messages`, { content });
}
