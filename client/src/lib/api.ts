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
  Role,
  Server,
  ServerWithDetails,
  UploadedFile,
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

/** In-flight refresh promise to prevent concurrent refresh attempts */
let refreshInFlight: Promise<AuthResponse> | null = null;

/**
 * Attempt to refresh the access token using the stored refresh token.
 * Returns the new AuthResponse or throws on failure.
 * Deduplicates concurrent calls.
 */
async function tryRefresh(): Promise<AuthResponse> {
  if (refreshInFlight) return refreshInFlight;

  const rt = localStorage.getItem('refresh_token');
  if (!rt) throw new Error('No refresh token');

  refreshInFlight = (async () => {
    const res = await fetch('/api/auth/refresh', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: rt }),
    });
    if (!res.ok) {
      // Refresh failed — clear everything
      localStorage.removeItem('token');
      localStorage.removeItem('refresh_token');
      throw new ApiError(res.status, 'Session expired');
    }
    const data: AuthResponse = await res.json();
    localStorage.setItem('token', data.token);
    localStorage.setItem('refresh_token', data.refresh_token);
    return data;
  })();

  try {
    return await refreshInFlight;
  } finally {
    refreshInFlight = null;
  }
}

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  _isRetry = false,
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

  // Auto-refresh on 401 (but not for auth endpoints themselves, and not on retry)
  if (res.status === 401 && !_isRetry && !path.includes('/auth/')) {
    try {
      await tryRefresh();
      // Retry the original request with the new token
      return request<T>(method, path, body, true);
    } catch {
      // Refresh also failed — propagate 401
    }
  }

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

export function updateMe(data: { display_name?: string; about?: string; email?: string }) {
  return request<PublicUser>('PATCH', '/api/users/@me', data);
}

export async function uploadAvatar(file: File): Promise<PublicUser> {
  const formData = new FormData();
  formData.append('file', file, file.name);

  const token = getToken();
  const headers: Record<string, string> = {};
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch('/api/users/@me/avatar', {
    method: 'POST',
    headers,
    body: formData,
  });

  if (res.status === 401) {
    try {
      await tryRefresh();
      const newToken = getToken();
      const retryHeaders: Record<string, string> = {};
      if (newToken) retryHeaders['Authorization'] = `Bearer ${newToken}`;
      const retry = await fetch('/api/users/@me/avatar', {
        method: 'POST',
        headers: retryHeaders,
        body: formData,
      });
      if (!retry.ok) throw new ApiError(retry.status, 'Avatar upload failed');
      return retry.json();
    } catch {
      throw new ApiError(401, 'Session expired');
    }
  }

  if (!res.ok) {
    let msg = res.statusText;
    try { const err = await res.json(); msg = err.error || msg; } catch {}
    throw new ApiError(res.status, msg);
  }

  return res.json();
}

export function changePassword(current_password: string, new_password: string) {
  return request<{ ok: boolean }>('POST', '/api/users/@me/password', { current_password, new_password });
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

export function sendMessage(channelId: string, content?: string, attachment_ids?: string[]) {
  return request<MessageWithAuthor>('POST', `/api/channels/${channelId}/messages`, { content, attachment_ids });
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

// ── Uploads ──

export async function uploadFiles(files: File[]): Promise<UploadedFile[]> {
  const formData = new FormData();
  for (const file of files) {
    formData.append('file', file, file.name);
  }

  const token = getToken();
  const headers: Record<string, string> = {};
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch('/api/upload', {
    method: 'POST',
    headers,
    body: formData,
  });

  if (res.status === 401) {
    try {
      await tryRefresh();
      // Retry with new token
      const newToken = getToken();
      const retryHeaders: Record<string, string> = {};
      if (newToken) retryHeaders['Authorization'] = `Bearer ${newToken}`;
      const retry = await fetch('/api/upload', {
        method: 'POST',
        headers: retryHeaders,
        body: formData,
      });
      if (!retry.ok) throw new ApiError(retry.status, 'Upload failed');
      return retry.json();
    } catch {
      throw new ApiError(401, 'Session expired');
    }
  }

  if (!res.ok) {
    let msg = res.statusText;
    try { const err = await res.json(); msg = err.error || msg; } catch {}
    throw new ApiError(res.status, msg);
  }

  return res.json();
}

// ── Roles ──

export function listRoles(serverId: string) {
  return request<Role[]>('GET', `/api/servers/${serverId}/roles`);
}

export function createRole(serverId: string, data: { name: string; color?: number; permissions?: number }) {
  return request<Role>('POST', `/api/servers/${serverId}/roles`, data);
}

export function updateRole(serverId: string, roleId: string, data: { name?: string; color?: number; permissions?: number; position?: number }) {
  return request<Role>('PATCH', `/api/servers/${serverId}/roles/${roleId}`, data);
}

export function deleteRole(serverId: string, roleId: string) {
  return request<void>('DELETE', `/api/servers/${serverId}/roles/${roleId}`);
}

export function getMemberRoles(serverId: string, userId: string) {
  return request<string[]>('GET', `/api/servers/${serverId}/members/${userId}/roles`);
}

export function assignRole(serverId: string, userId: string, roleId: string) {
  return request<void>('PUT', `/api/servers/${serverId}/members/${userId}/roles/${roleId}`);
}

export function removeRole(serverId: string, userId: string, roleId: string) {
  return request<void>('DELETE', `/api/servers/${serverId}/members/${userId}/roles/${roleId}`);
}
