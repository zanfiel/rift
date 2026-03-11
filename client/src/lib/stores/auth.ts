// ── Auth store — manages login state, token, current user ──

import type { PublicUser } from '../types';
import * as api from '../api';
import { gateway } from '../gateway';

// Reactive state using module-level variables + subscribers pattern
// (Svelte 5 runes only work inside .svelte files, so stores use a pub/sub model)

type Subscriber = () => void;

let _user: PublicUser | null = null;
let _token: string | null = localStorage.getItem('token');
let _refreshToken: string | null = localStorage.getItem('refresh_token');
let _loading = true;
let _error: string | null = null;
const subs: Set<Subscriber> = new Set();

function notify() {
  for (const fn of subs) fn();
}

export const auth = {
  subscribe(fn: Subscriber): () => void {
    subs.add(fn);
    return () => subs.delete(fn);
  },

  get user(): PublicUser | null { return _user; },
  get token(): string | null { return _token; },
  get loading(): boolean { return _loading; },
  get error(): string | null { return _error; },
  get loggedIn(): boolean { return _user !== null && _token !== null; },

  /** Try to restore session from stored token */
  async init(): Promise<void> {
    if (!_token) {
      _loading = false;
      notify();
      return;
    }
    try {
      _user = await api.getMe();
      gateway.connect(_token);
    } catch {
      // Access token expired — the API layer will auto-refresh if a refresh token exists.
      // If auto-refresh succeeded, the new token is already in localStorage.
      const newToken = localStorage.getItem('token');
      if (newToken && newToken !== _token) {
        // Refresh worked — retry with new token
        _token = newToken;
        _refreshToken = localStorage.getItem('refresh_token');
        try {
          _user = await api.getMe();
          gateway.connect(_token);
        } catch {
          // Still failing — clear everything
          _clearSession();
        }
      } else {
        _clearSession();
      }
    }
    _loading = false;
    notify();
  },

  async login(username: string, password: string): Promise<void> {
    _error = null;
    notify();
    try {
      const res = await api.login(username, password);
      _setSession(res.token, res.refresh_token, res.user);
      gateway.connect(res.token);
      notify();
    } catch (e: any) {
      _error = e.message || 'Login failed';
      notify();
      throw e;
    }
  },

  async register(username: string, email: string, password: string, displayName?: string): Promise<void> {
    _error = null;
    notify();
    try {
      const res = await api.register(username, email, password, displayName);
      _setSession(res.token, res.refresh_token, res.user);
      gateway.connect(res.token);
      notify();
    } catch (e: any) {
      _error = e.message || 'Registration failed';
      notify();
      throw e;
    }
  },

  async logout(): Promise<void> {
    try {
      await api.logout();
    } catch { /* ignore */ }
    gateway.disconnect();
    _clearSession();
    notify();
  },

  clearError(): void {
    _error = null;
    notify();
  },

  /** Update the cached user object (e.g. after profile edit) */
  updateUser(updates: Partial<PublicUser>): void {
    if (_user) {
      _user = { ..._user, ...updates };
      notify();
    }
  },
};

function _setSession(token: string, refreshToken: string, user: PublicUser) {
  _token = token;
  _refreshToken = refreshToken;
  _user = user;
  localStorage.setItem('token', token);
  localStorage.setItem('refresh_token', refreshToken);
}

function _clearSession() {
  _token = null;
  _refreshToken = null;
  _user = null;
  localStorage.removeItem('token');
  localStorage.removeItem('refresh_token');
}
