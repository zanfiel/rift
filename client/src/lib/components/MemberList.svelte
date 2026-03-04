<script lang="ts">
  import { servers } from '../stores/servers';
  import type { MemberWithUser } from '../types';

  let membersList: MemberWithUser[] = $state([]);

  $effect(() => {
    const unsub = servers.subscribe(() => {
      membersList = servers.members;
    });
    return unsub;
  });

  // Group by status (online first, then offline)
  let onlineMembers: MemberWithUser[] = $derived(
    membersList.filter(m => m.status === 'online' || m.status === 'idle' || m.status === 'dnd')
  );
  let offlineMembers: MemberWithUser[] = $derived(
    membersList.filter(m => m.status === 'offline' || (!m.status))
  );
</script>

<div class="member-list">
  {#if onlineMembers.length > 0}
    <div class="member-group">
      <h4 class="group-header">Online — {onlineMembers.length}</h4>
      {#each onlineMembers as member}
        <div class="member-item">
          <div class="avatar">
            {#if member.avatar_url}
              <img src={member.avatar_url} alt={member.username} />
            {:else}
              {member.username.charAt(0).toUpperCase()}
            {/if}
            <div class="status-dot online"></div>
          </div>
          <div class="member-info">
            <span class="member-name truncate">{member.display_name ?? member.username}</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if offlineMembers.length > 0}
    <div class="member-group">
      <h4 class="group-header">Offline — {offlineMembers.length}</h4>
      {#each offlineMembers as member}
        <div class="member-item offline">
          <div class="avatar">
            {#if member.avatar_url}
              <img src={member.avatar_url} alt={member.username} />
            {:else}
              {member.username.charAt(0).toUpperCase()}
            {/if}
            <div class="status-dot"></div>
          </div>
          <div class="member-info">
            <span class="member-name truncate">{member.display_name ?? member.username}</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if membersList.length === 0}
    <div class="empty">No members</div>
  {/if}
</div>

<style>
  .member-list {
    width: 240px;
    background: var(--bg-secondary);
    flex-shrink: 0;
    overflow-y: auto;
    padding: 8px 0;
  }

  .member-group {
    margin-bottom: 8px;
  }

  .group-header {
    padding: 16px 16px 4px;
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.02em;
    color: var(--text-muted);
  }

  .member-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 8px 4px 16px;
    margin: 1px 8px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--transition);
  }

  .member-item:hover {
    background: var(--bg-hover);
  }

  .member-item.offline {
    opacity: 0.5;
  }

  .avatar {
    position: relative;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-full);
    background: var(--bg-accent);
    color: var(--text-white);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 14px;
    flex-shrink: 0;
    overflow: visible;
  }
  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: var(--radius-full);
  }

  .status-dot {
    position: absolute;
    bottom: -2px;
    right: -2px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--text-faint);
    border: 3px solid var(--bg-secondary);
  }

  .status-dot.online {
    background: var(--bg-success);
  }

  .member-info {
    min-width: 0;
    flex: 1;
  }

  .member-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-muted);
  }

  .member-item:hover .member-name {
    color: var(--text-normal);
  }

  .empty {
    text-align: center;
    color: var(--text-faint);
    padding: 24px;
  }
</style>
