<script lang="ts">
  import { servers } from '../stores/servers';
  import { ui } from '../stores/ui';
  import { auth } from '../stores/auth';

  let serverList: typeof servers.list = $state([]);

  $effect(() => {
    const unsub = servers.subscribe(() => {
      serverList = servers.list;
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
</style>
