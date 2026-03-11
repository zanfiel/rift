// ── UI store — manages global UI state (modals, views, etc.) ──

type Subscriber = () => void;

export type View = 'servers' | 'dms';
export type Modal = 'create-server' | 'create-channel' | 'invite' | 'settings' | 'join-server' | 'user-settings' | null;

let _view: View = 'servers';
let _modal: Modal = null;
let _sidebarCollapsed = false;
let _memberListVisible = true;
const subs: Set<Subscriber> = new Set();

function notify() {
  for (const fn of subs) fn();
}

export const ui = {
  subscribe(fn: Subscriber): () => void {
    subs.add(fn);
    return () => subs.delete(fn);
  },

  get view(): View { return _view; },
  get modal(): Modal { return _modal; },
  get sidebarCollapsed(): boolean { return _sidebarCollapsed; },
  get memberListVisible(): boolean { return _memberListVisible; },

  setView(view: View): void {
    _view = view;
    notify();
  },

  openModal(modal: Modal): void {
    _modal = modal;
    notify();
  },

  closeModal(): void {
    _modal = null;
    notify();
  },

  toggleSidebar(): void {
    _sidebarCollapsed = !_sidebarCollapsed;
    notify();
  },

  toggleMemberList(): void {
    _memberListVisible = !_memberListVisible;
    notify();
  },
};
