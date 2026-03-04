// ── Auth store — manages login state, token, current user ──

import type { PublicUser } from '../types';
import * as api from '../api';
import { gateway } from '../gateway';

// Reactive state using module-level variables + subscribers pattern
// (Svelte 5 runes only work inside .svelte files, so stores use a pub/sub model)

type Subscriber = () => void;

let _user: PublicUser | null = null;
let _token: string | null = localStorage.getItem('token');
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
      // Token expired or invalid
      localStorage.removeItem('token');
      _token = null;
      _user = null;
    }
    _loading = false;
    notify();
  },

  async login(username: string, password: string): Promise<void> {
    _error = null;
    notify();
    try {
      const res = await api.login(username, password);
      _token = res.token;
      _user = res.user;
      localStorage.setItem('token', res.token);
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
      _token = res.token;
      _user = res.user;
      localStorage.setItem('token', res.token);
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
    _token = null;
    _user = null;
    localStorage.removeItem('token');
    notify();
  },

  clearError(): void {
    _error = null;
    notify();
  },
};
