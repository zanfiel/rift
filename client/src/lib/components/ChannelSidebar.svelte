<script lang="ts">
  import { servers } from '../stores/servers';
  import { ui } from '../stores/ui';
  import { auth } from '../stores/auth';
  import type { Channel, ServerWithDetails } from '../types';

  let currentServer: ServerWithDetails | null = $state(null);
  let currentChannelId: string | null = $state(null);

  $effect(() => {
    const unsub = servers.subscribe(() => {
      currentServer = servers.current;
      currentChannelId = servers.currentChannelId;
    });
    return unsub;
  });

  function selectChannel(id: string) {
    servers.selectChannel(id);
  }

  function handleCreateChannel() {
    ui.openModal('create-channel');
  }

  function handleServerSettings() {
    ui.openModal('settings');
  }

  function handleInvite() {
    ui.openModal('invite');
  }

  function handleLogout() {
    auth.logout();
  }

  let textChannels: Channel[] = $derived.by(() => {
    const cs = currentServer;
    if (!cs) return [];
    return cs.channels.filter((c: Channel) => c.channel_type === 'text').sort((a: Channel, b: Channel) => a.position - b.position);
  });
</script>

<div class="channel-sidebar">
  {#if currentServer}
    <!-- Server Header -->
    <div class="server-header">
      <button class="server-name" onclick={handleServerSettings}>
        <span class="truncate">{currentServer.server.name}</span>
        <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor" style="flex-shrink: 0; margin-left: 4px;">
          <path d="M3 4.5l3 3 3-3" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <!-- Channel List -->
    <div class="channel-list">
      <div class="category">
        <div class="category-header">
          <span class="category-name">Text Channels</span>
          <button class="category-action" title="Create Channel" onclick={handleCreateChannel}>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 3a.5.5 0 01.5.5V7h3.5a.5.5 0 010 1H8.5v3.5a.5.5 0 01-1 0V8H4a.5.5 0 010-1h3.5V3.5A.5.5 0 018 3z"/>
            </svg>
          </button>
        </div>

        {#each textChannels as channel}
          <button
            class="channel-item"
            class:active={channel.id === currentChannelId}
            onclick={() => selectChannel(channel.id)}
          >
            <span class="channel-hash">#</span>
            <span class="truncate">{channel.name}</span>
          </button>
        {/each}
      </div>
    </div>

    <!-- Actions bar at bottom -->
    <div class="sidebar-actions">
      <button class="action-btn" title="Create Invite" onclick={handleInvite}>
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 1a3 3 0 00-3 3v2H4a1 1 0 00-1 1v7a1 1 0 001 1h8a1 1 0 001-1V7a1 1 0 00-1-1h-1V4a3 3 0 00-3-3zm0 1.5A1.5 1.5 0 019.5 4v2h-3V4A1.5 1.5 0 018 2.5z"/>
        </svg>
        <span>Invite</span>
      </button>
    </div>
  {:else}
    <div class="empty-sidebar">
      <p>Select a server</p>
    </div>
  {/if}

  <!-- User bar -->
  <div class="user-bar">
    <div class="user-info">
      <div class="avatar sm">
        {auth.user?.username?.charAt(0)?.toUpperCase() ?? '?'}
      </div>
      <div class="user-text">
        <span class="user-name truncate">{auth.user?.display_name ?? auth.user?.username ?? 'User'}</span>
        <span class="user-status">Online</span>
      </div>
    </div>
    <button class="icon-btn" title="Log Out" onclick={handleLogout}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M6 2a1 1 0 00-1 1v2a.5.5 0 001 0V3h6v10H6v-2a.5.5 0 00-1 0v2a1 1 0 001 1h6a1 1 0 001-1V3a1 1 0 00-1-1H6z"/>
        <path d="M1.646 7.646a.5.5 0 000 .708l2.5 2.5a.5.5 0 00.708-.708L3.207 8.5H10a.5.5 0 000-1H3.207l1.647-1.646a.5.5 0 10-.708-.708l-2.5 2.5z"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .channel-sidebar {
    width: 240px;
    background: var(--bg-secondary);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .server-header {
    height: 48px;
    padding: 0 12px;
    display: flex;
    align-items: center;
    border-bottom: 2px solid var(--bg-tertiary);
    flex-shrink: 0;
  }

  .server-name {
    background: none;
    border: none;
    color: var(--text-white);
    font-size: 15px;
    font-weight: 600;
    padding: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    width: 100%;
    overflow: hidden;
  }
  .server-name:hover {
    color: var(--text-normal);
  }

  .channel-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .category {
    margin-bottom: 8px;
  }

  .category-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px 0 16px;
    margin-bottom: 2px;
  }

  .category-name {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.02em;
    color: var(--text-muted);
  }

  .category-action {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px;
    display: flex;
    border-radius: var(--radius-sm);
  }
  .category-action:hover {
    color: var(--text-normal);
  }

  .channel-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: calc(100% - 16px);
    margin: 1px 8px;
    padding: 6px 8px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
    text-align: left;
    transition: background var(--transition), color var(--transition);
  }

  .channel-item:hover {
    background: var(--bg-hover);
    color: var(--text-normal);
  }

  .channel-item.active {
    background: var(--bg-active);
    color: var(--text-white);
  }

  .channel-hash {
    color: var(--text-faint);
    font-size: 18px;
    font-weight: 500;
    width: 20px;
    text-align: center;
    flex-shrink: 0;
  }

  .sidebar-actions {
    padding: 8px;
    border-top: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 8px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-size: 13px;
    cursor: pointer;
  }
  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-normal);
  }

  .empty-sidebar {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-faint);
  }

  /* User Bar */
  .user-bar {
    height: 52px;
    padding: 0 8px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--bg-tertiary);
    flex-shrink: 0;
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
  }

  .user-text {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .user-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-white);
  }

  .user-status {
    font-size: 11px;
    color: var(--text-muted);
  }

  .icon-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 6px;
    border-radius: var(--radius-sm);
    display: flex;
  }
  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-normal);
  }
</style>
