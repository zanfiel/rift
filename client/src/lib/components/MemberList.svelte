<script lang="ts">
  import { servers } from '../stores/servers';
  import * as api from '../api';
  import type { MemberWithUser, Role } from '../types';

  let membersList: MemberWithUser[] = $state([]);
  let roles: Role[] = $state([]);

  $effect(() => {
    const unsub = servers.subscribe(() => {
      membersList = servers.members;
      if (servers.current) {
        roles = servers.current.roles;
      }
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

  // ── Role popup state ──
  let popupMember: MemberWithUser | null = $state(null);
  let popupRoleIds: Set<string> = $state(new Set());
  let popupLoading: boolean = $state(false);
  let popupX: number = $state(0);
  let popupY: number = $state(0);
  let togglingRoleId: string | null = $state(null);

  // Non-default roles for the popup
  let assignableRoles: Role[] = $derived(
    roles.filter(r => !r.is_default).sort((a, b) => b.position - a.position)
  );

  function intToHex(color: number): string {
    return '#' + (color & 0xFFFFFF).toString(16).padStart(6, '0');
  }

  async function openRolePopup(member: MemberWithUser, event: MouseEvent) {
    if (!servers.current) return;
    // Position popup to the left of the click
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    popupX = rect.left - 220;
    popupY = rect.top;
    popupMember = member;
    popupLoading = true;
    popupRoleIds = new Set();

    try {
      const ids = await api.getMemberRoles(servers.current.server.id, member.user_id);
      popupRoleIds = new Set(ids);
    } catch (e) {
      console.error('Failed to load member roles:', e);
    }
    popupLoading = false;
  }

  function closePopup() {
    popupMember = null;
    togglingRoleId = null;
  }

  async function toggleRole(roleId: string) {
    if (!servers.current || !popupMember || togglingRoleId) return;
    togglingRoleId = roleId;

    try {
      if (popupRoleIds.has(roleId)) {
        await api.removeRole(servers.current.server.id, popupMember.user_id, roleId);
        const next = new Set(popupRoleIds);
        next.delete(roleId);
        popupRoleIds = next;
      } else {
        await api.assignRole(servers.current.server.id, popupMember.user_id, roleId);
        const next = new Set(popupRoleIds);
        next.add(roleId);
        popupRoleIds = next;
      }
    } catch (e: any) {
      alert(e.message || 'Failed to update role');
    }
    togglingRoleId = null;
  }

  function handleWindowClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.role-popup') && !target.closest('.member-item')) {
      closePopup();
    }
  }
</script>

<svelte:window onclick={handleWindowClick} />

<div class="member-list">
  {#if onlineMembers.length > 0}
    <div class="member-group">
      <h4 class="group-header">Online — {onlineMembers.length}</h4>
      {#each onlineMembers as member}
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
        <div class="member-item" onclick={(e) => openRolePopup(member, e)} role="button" tabindex="0">
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
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
        <div class="member-item offline" onclick={(e) => openRolePopup(member, e)} role="button" tabindex="0">
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

<!-- Role assignment popup -->
{#if popupMember}
  <div class="role-popup" style="top:{popupY}px;left:{popupX}px;">
    <div class="popup-header">
      <span class="popup-name truncate">{popupMember.display_name ?? popupMember.username}</span>
      <button class="popup-close" onclick={closePopup}>x</button>
    </div>
    {#if popupLoading}
      <div class="popup-loading"><div class="spinner"></div></div>
    {:else if assignableRoles.length === 0}
      <div class="popup-empty">No roles to assign</div>
    {:else}
      <div class="popup-roles">
        {#each assignableRoles as role}
          <label class="popup-role-row" class:toggling={togglingRoleId === role.id}>
            <input
              type="checkbox"
              checked={popupRoleIds.has(role.id)}
              disabled={togglingRoleId !== null}
              onchange={() => toggleRole(role.id)}
            />
            <span class="popup-role-dot" style="background:{intToHex(role.color)};"></span>
            <span class="popup-role-name">{role.name}</span>
          </label>
        {/each}
      </div>
    {/if}
  </div>
{/if}

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

  /* ── Role popup ── */

  .role-popup {
    position: fixed;
    z-index: 2000;
    width: 210px;
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-high);
    animation: scale-in 100ms ease;
    max-height: 300px;
    display: flex;
    flex-direction: column;
  }

  .popup-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .popup-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-white);
  }

  .popup-close {
    background: none;
    border: none;
    color: var(--text-faint);
    font-size: 14px;
    padding: 0 4px;
    cursor: pointer;
    line-height: 1;
  }

  .popup-close:hover {
    color: var(--text-normal);
  }

  .popup-loading {
    display: flex;
    justify-content: center;
    padding: 16px;
  }

  .popup-empty {
    padding: 12px;
    text-align: center;
    font-size: 13px;
    color: var(--text-faint);
  }

  .popup-roles {
    overflow-y: auto;
    padding: 6px 0;
  }

  .popup-role-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 10px;
    cursor: pointer;
    transition: background var(--transition);
    font-size: 13px;
  }

  .popup-role-row:hover {
    background: var(--bg-hover);
  }

  .popup-role-row.toggling {
    opacity: 0.5;
  }

  .popup-role-row input[type="checkbox"] {
    width: 14px;
    height: 14px;
    accent-color: var(--bg-accent);
    cursor: pointer;
    flex-shrink: 0;
  }

  .popup-role-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .popup-role-name {
    color: var(--text-normal);
    user-select: none;
  }

  @keyframes scale-in {
    from { opacity: 0; transform: scale(0.95); }
    to { opacity: 1; transform: scale(1); }
  }
</style>
