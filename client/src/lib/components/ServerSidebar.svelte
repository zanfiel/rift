<script lang="ts">
  import { servers } from '../stores/servers';
  import { ui } from '../stores/ui';
  import { auth } from '../stores/auth';
  import type { PublicUser } from '../types';

  let serverList: typeof servers.list = $state([]);
  let currentUser: PublicUser | null = $state(null);

  $effect(() => {
    const unsub = servers.subscribe(() => {
      serverList = servers.list;
    });
    return unsub;
  });

  $effect(() => {
    const unsub = auth.subscribe(() => {
      currentUser = auth.user;
    });
    return unsub;
  });

  function selectServer(id: string) {
    servers.select(id);
  }

  function handleCreateServer() {
    ui.openModal('create-server');
  }

  function handleJoinServer() {
    ui.openModal('join-server');
  }

  function switchToDms() {
    ui.setView('dms');
  }

  function openUserSettings() {
    ui.openModal('user-settings');
  }

  function getInitial(user: PublicUser): string {
    return (user.display_name ?? user.username).charAt(0).toUpperCase();
  }
</script>

<div class="server-sidebar">
  <!-- DM Button -->
  <div class="sidebar-item" title="Direct Messages">
    <button class="server-icon dm-icon" onclick={switchToDms} aria-label="Direct Messages">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 2C6.477 2 2 6.477 2 12c0 1.89.525 3.66 1.438 5.168L2.546 20.2A1.5 1.5 0 003.8 21.454l3.032-.892A9.96 9.96 0 0012 22c5.523 0 10-4.477 10-10S17.523 2 12 2zM8 13a1 1 0 110-2 1 1 0 010 2zm4 0a1 1 0 110-2 1 1 0 010 2zm4 0a1 1 0 110-2 1 1 0 010 2z"/>
      </svg>
    </button>
  </div>

  <div class="divider"></div>

  <!-- Server List -->
  {#each serverList as server}
    <div class="sidebar-item" title={server.name}>
      <button
        class="server-icon"
        class:active={servers.current?.server.id === server.id}
        onclick={() => selectServer(server.id)}
      >
        {#if server.icon_url}
          <img src={server.icon_url} alt={server.name} />
        {:else}
          <span class="server-initial">{server.name.charAt(0).toUpperCase()}</span>
        {/if}
      </button>
      <div class="pill" class:active={servers.current?.server.id === server.id}></div>
    </div>
  {/each}

  <div class="divider"></div>

  <!-- Add Server -->
  <div class="sidebar-item" title="Create Server">
    <button class="server-icon add-server" onclick={handleCreateServer} aria-label="Create Server">
      <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
        <path d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z"/>
      </svg>
    </button>
  </div>

  <!-- Join Server -->
  <div class="sidebar-item" title="Join Server">
    <button class="server-icon join-server" onclick={handleJoinServer} aria-label="Join Server">
      <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
        <path d="M10 2a8 8 0 100 16 8 8 0 000-16zM6.39 9.967l2.59-2.59a.75.75 0 011.06 0l2.59 2.59a.75.75 0 01-1.06 1.06L10 9.458l-1.57 1.57a.75.75 0 01-1.06-1.06z"/>
      </svg>
    </button>
  </div>

  <!-- Spacer pushes user bar to bottom -->
  <div class="spacer"></div>

  <!-- User bar -->
  {#if currentUser}
    <div class="user-bar" title="{currentUser.display_name ?? currentUser.username}">
      <div class="user-avatar">
        {#if currentUser.avatar_url}
          <img src={currentUser.avatar_url} alt="Avatar" />
        {:else}
          <span>{getInitial(currentUser)}</span>
        {/if}
      </div>
      <button class="settings-btn" onclick={openUserSettings} aria-label="User Settings" title="User Settings">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 10a2 2 0 100-4 2 2 0 000 4z"/>
          <path fill-rule="evenodd" d="M6.143 1.31a.75.75 0 01.736-.61h2.242a.75.75 0 01.736.61l.27 1.355a5.484 5.484 0 011.072.62l1.31-.44a.75.75 0 01.874.344l1.122 1.942a.75.75 0 01-.14.953l-1.04.916a5.56 5.56 0 010 1.24l1.04.916a.75.75 0 01.14.953l-1.122 1.942a.75.75 0 01-.874.344l-1.31-.44a5.484 5.484 0 01-1.072.62l-.27 1.355a.75.75 0 01-.736.61H6.879a.75.75 0 01-.736-.61l-.27-1.355a5.484 5.484 0 01-1.072-.62l-1.31.44a.75.75 0 01-.874-.344L1.495 11.9a.75.75 0 01.14-.953l1.04-.916a5.56 5.56 0 010-1.24l-1.04-.916a.75.75 0 01-.14-.953l1.122-1.942a.75.75 0 01.874-.344l1.31.44a5.484 5.484 0 011.072-.62l.27-1.355zM8 11a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd"/>
        </svg>
      </button>
    </div>
  {/if}
</div>

<style>
  .server-sidebar {
    width: 72px;
    background: var(--bg-tertiary);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px 0;
    gap: 4px;
    overflow-y: auto;
    flex-shrink: 0;
  }

  .sidebar-item {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .server-icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--bg-primary);
    color: var(--text-muted);
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    font-weight: 600;
    transition: border-radius var(--transition), background var(--transition), color var(--transition);
    overflow: hidden;
  }

  .server-icon:hover, .server-icon.active {
    border-radius: 16px;
    background: var(--bg-accent);
    color: var(--text-white);
  }

  .server-icon img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .server-initial {
    user-select: none;
  }

  .dm-icon:hover {
    background: var(--bg-accent);
    color: var(--text-white);
  }

  .add-server {
    color: var(--bg-success);
  }
  .add-server:hover {
    background: var(--bg-success);
    color: var(--text-white);
  }

  .join-server {
    color: var(--bg-success);
  }
  .join-server:hover {
    background: var(--bg-success);
    color: var(--text-white);
  }

  .pill {
    position: absolute;
    left: 0;
    width: 4px;
    height: 0;
    background: var(--text-white);
    border-radius: 0 4px 4px 0;
    transition: height var(--transition);
  }
  .pill.active {
    height: 40px;
  }
  .sidebar-item:hover .pill:not(.active) {
    height: 20px;
  }

  .divider {
    width: 32px;
    height: 2px;
    background: var(--border-subtle);
    border-radius: 1px;
    margin: 4px 0;
  }

  .spacer {
    flex: 1;
  }

  /* ── User bar ── */

  .user-bar {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding-top: 4px;
    border-top: 2px solid var(--border-subtle);
    width: 48px;
  }

  .user-avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: var(--bg-accent);
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-white);
  }

  .user-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .settings-btn {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: none;
    color: var(--text-faint);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition: color var(--transition), background var(--transition);
  }

  .settings-btn:hover {
    color: var(--text-normal);
    background: var(--bg-hover);
  }
</style>
