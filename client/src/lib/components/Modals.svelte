<script lang="ts">
  import { ui } from '../stores/ui';
  import { servers } from '../stores/servers';
  import * as api from '../api';
  import type { Invite, Role } from '../types';
  import { Permissions, hasPermission } from '../types';
  import UserSettings from './UserSettings.svelte';

  let modal: typeof ui.modal = $state(null);

  $effect(() => {
    const unsub = ui.subscribe(() => {
      modal = ui.modal;
    });
    return unsub;
  });

  // ── Create Server ──
  let serverName: string = $state('');
  let serverDesc: string = $state('');
  let serverSubmitting: boolean = $state(false);

  async function createServer() {
    if (!serverName.trim()) return;
    serverSubmitting = true;
    try {
      const s = await servers.create(serverName.trim(), serverDesc.trim() || undefined);
      await servers.select(s.id);
      ui.closeModal();
      serverName = '';
      serverDesc = '';
    } catch (e: any) {
      alert(e.message || 'Failed to create server');
    }
    serverSubmitting = false;
  }

  // ── Create Channel ──
  let channelName: string = $state('');
  let channelTopic: string = $state('');
  let channelSubmitting: boolean = $state(false);

  async function createChannel() {
    if (!channelName.trim()) return;
    channelSubmitting = true;
    try {
      const ch = await servers.createChannel(channelName.trim().toLowerCase().replace(/\s+/g, '-'), channelTopic.trim() || undefined);
      if (ch) servers.selectChannel(ch.id);
      ui.closeModal();
      channelName = '';
      channelTopic = '';
    } catch (e: any) {
      alert(e.message || 'Failed to create channel');
    }
    channelSubmitting = false;
  }

  // ── Join Server ──
  let inviteCode: string = $state('');
  let joinSubmitting: boolean = $state(false);

  async function joinServer() {
    if (!inviteCode.trim()) return;
    joinSubmitting = true;
    try {
      const s = await api.joinInvite(inviteCode.trim());
      await servers.load();
      await servers.select(s.id);
      ui.closeModal();
      inviteCode = '';
    } catch (e: any) {
      alert(e.message || 'Invalid invite code');
    }
    joinSubmitting = false;
  }

  // ── Invite ──
  let invites: Invite[] = $state([]);
  let inviteLoading: boolean = $state(false);
  let newInviteCode: string = $state('');

  async function loadInvites() {
    if (!servers.current) return;
    inviteLoading = true;
    try {
      invites = await api.listInvites(servers.current.server.id);
    } catch { /* ignore */ }
    inviteLoading = false;
  }

  async function generateInvite() {
    if (!servers.current) return;
    try {
      const inv = await api.createInvite(servers.current.server.id, undefined, 24);
      newInviteCode = inv.code;
      invites = [...invites, inv];
    } catch (e: any) {
      alert(e.message || 'Failed to create invite');
    }
  }

  async function deleteInvite(code: string) {
    if (!servers.current) return;
    try {
      await api.deleteInvite(servers.current.server.id, code);
      invites = invites.filter(i => i.code !== code);
      if (newInviteCode === code) newInviteCode = '';
    } catch { /* ignore */ }
  }

  function copyInvite(code: string) {
    navigator.clipboard.writeText(code);
  }

  // Load invites when invite modal opens
  $effect(() => {
    if (modal === 'invite') {
      loadInvites();
      newInviteCode = '';
    }
  });

  // ── Settings ──
  let settingsTab: 'general' | 'roles' = $state('general');
  let settingsName: string = $state('');
  let settingsDesc: string = $state('');
  let settingsSubmitting: boolean = $state(false);

  $effect(() => {
    if (modal === 'settings' && servers.current) {
      settingsName = servers.current.server.name;
      settingsDesc = servers.current.server.description ?? '';
      settingsTab = 'general';
      // Reset role editing state
      editingRole = null;
      creatingRole = false;
      newRoleName = '';
      newRoleColor = '#5865f2';
      newRolePermissions = Permissions.VIEW_CHANNELS | Permissions.SEND_MESSAGES | Permissions.READ_HISTORY;
    }
  });

  async function saveSettings() {
    if (!servers.current || !settingsName.trim()) return;
    settingsSubmitting = true;
    try {
      await api.updateServer(servers.current.server.id, {
        name: settingsName.trim(),
        description: settingsDesc.trim() || undefined,
      });
      await servers.select(servers.current.server.id);
      ui.closeModal();
    } catch (e: any) {
      alert(e.message || 'Failed to update server');
    }
    settingsSubmitting = false;
  }

  async function deleteServer() {
    if (!servers.current) return;
    if (!confirm(`Delete "${servers.current.server.name}"? This cannot be undone.`)) return;
    try {
      await api.deleteServer(servers.current.server.id);
      await servers.load();
      ui.closeModal();
    } catch (e: any) {
      alert(e.message || 'Failed to delete server');
    }
  }

  // ── Role Management ──
  let roles: Role[] = $state([]);
  let editingRole: Role | null = $state(null);
  let creatingRole: boolean = $state(false);
  let roleSubmitting: boolean = $state(false);

  // New role form
  let newRoleName: string = $state('');
  let newRoleColor: string = $state('#5865f2');
  let newRolePermissions: number = $state(Permissions.VIEW_CHANNELS | Permissions.SEND_MESSAGES | Permissions.READ_HISTORY);

  // Edit role form
  let editRoleName: string = $state('');
  let editRoleColor: string = $state('#5865f2');
  let editRolePermissions: number = $state(0);

  // Subscribe to server store for live role updates
  $effect(() => {
    const unsub = servers.subscribe(() => {
      if (servers.current) {
        roles = servers.current.roles;
      }
    });
    return unsub;
  });

  // Permission definitions for the UI
  const permissionDefs = [
    { key: Permissions.VIEW_CHANNELS, label: 'View Channels' },
    { key: Permissions.SEND_MESSAGES, label: 'Send Messages' },
    { key: Permissions.READ_HISTORY, label: 'Read Message History' },
    { key: Permissions.MANAGE_MESSAGES, label: 'Manage Messages' },
    { key: Permissions.ATTACH_FILES, label: 'Attach Files' },
    { key: Permissions.MANAGE_CHANNELS, label: 'Manage Channels' },
    { key: Permissions.MANAGE_SERVER, label: 'Manage Server' },
    { key: Permissions.MANAGE_ROLES, label: 'Manage Roles' },
    { key: Permissions.KICK_MEMBERS, label: 'Kick Members' },
    { key: Permissions.BAN_MEMBERS, label: 'Ban Members' },
    { key: Permissions.CREATE_INVITES, label: 'Create Invites' },
    { key: Permissions.MANAGE_INVITES, label: 'Manage Invites' },
    { key: Permissions.MENTION_EVERYONE, label: 'Mention Everyone' },
    { key: Permissions.ADMINISTRATOR, label: 'Administrator' },
  ];

  function intToHex(color: number): string {
    return '#' + (color & 0xFFFFFF).toString(16).padStart(6, '0');
  }

  function hexToInt(hex: string): number {
    return parseInt(hex.replace('#', ''), 16);
  }

  function togglePermission(current: number, perm: number): number {
    return current ^ perm;
  }

  function startCreateRole() {
    creatingRole = true;
    editingRole = null;
    newRoleName = '';
    newRoleColor = '#5865f2';
    newRolePermissions = Permissions.VIEW_CHANNELS | Permissions.SEND_MESSAGES | Permissions.READ_HISTORY;
  }

  function startEditRole(role: Role) {
    if (role.is_default) return; // Can't edit @everyone permissions display (backend prevents name/delete)
    editingRole = role;
    creatingRole = false;
    editRoleName = role.name;
    editRoleColor = intToHex(role.color);
    editRolePermissions = role.permissions;
  }

  function cancelRoleEdit() {
    editingRole = null;
    creatingRole = false;
  }

  async function createRole() {
    if (!servers.current || !newRoleName.trim()) return;
    roleSubmitting = true;
    try {
      await api.createRole(servers.current.server.id, {
        name: newRoleName.trim(),
        color: hexToInt(newRoleColor),
        permissions: newRolePermissions,
      });
      creatingRole = false;
      newRoleName = '';
    } catch (e: any) {
      alert(e.message || 'Failed to create role');
    }
    roleSubmitting = false;
  }

  async function saveRole() {
    if (!servers.current || !editingRole || !editRoleName.trim()) return;
    roleSubmitting = true;
    try {
      await api.updateRole(servers.current.server.id, editingRole.id, {
        name: editRoleName.trim(),
        color: hexToInt(editRoleColor),
        permissions: editRolePermissions,
      });
      editingRole = null;
    } catch (e: any) {
      alert(e.message || 'Failed to update role');
    }
    roleSubmitting = false;
  }

  async function deleteRole(role: Role) {
    if (!servers.current || role.is_default) return;
    if (!confirm(`Delete role "${role.name}"?`)) return;
    try {
      await api.deleteRole(servers.current.server.id, role.id);
      if (editingRole?.id === role.id) editingRole = null;
    } catch (e: any) {
      alert(e.message || 'Failed to delete role');
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('modal-overlay')) {
      ui.closeModal();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') ui.closeModal();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if modal}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-overlay" onclick={handleOverlayClick} onkeydown={handleKeydown} role="dialog" tabindex="-1">
    {#if modal === 'create-server'}
      <div class="modal">
        <h2>Create a Server</h2>
        <div class="field">
          <label for="cs-name">Server Name</label>
          <input id="cs-name" type="text" bind:value={serverName} placeholder="My awesome server" maxlength={100} />
        </div>
        <div class="field">
          <label for="cs-desc">Description <span style="text-transform:none;font-weight:400;color:var(--text-faint)">(optional)</span></label>
          <input id="cs-desc" type="text" bind:value={serverDesc} placeholder="What's this server about?" maxlength={256} />
        </div>
        <div class="actions">
          <button class="btn-ghost" onclick={() => ui.closeModal()}>Cancel</button>
          <button class="btn-primary" onclick={createServer} disabled={serverSubmitting || !serverName.trim()}>
            {serverSubmitting ? 'Creating...' : 'Create'}
          </button>
        </div>
      </div>
    {/if}

    {#if modal === 'create-channel'}
      <div class="modal">
        <h2>Create Channel</h2>
        <div class="field">
          <label for="cc-name">Channel Name</label>
          <input id="cc-name" type="text" bind:value={channelName} placeholder="new-channel" maxlength={100} />
        </div>
        <div class="field">
          <label for="cc-topic">Topic <span style="text-transform:none;font-weight:400;color:var(--text-faint)">(optional)</span></label>
          <input id="cc-topic" type="text" bind:value={channelTopic} placeholder="What's this channel about?" maxlength={256} />
        </div>
        <div class="actions">
          <button class="btn-ghost" onclick={() => ui.closeModal()}>Cancel</button>
          <button class="btn-primary" onclick={createChannel} disabled={channelSubmitting || !channelName.trim()}>
            {channelSubmitting ? 'Creating...' : 'Create'}
          </button>
        </div>
      </div>
    {/if}

    {#if modal === 'join-server'}
      <div class="modal">
        <h2>Join a Server</h2>
        <p style="color: var(--text-muted); margin-bottom: 16px;">Enter an invite code to join an existing server.</p>
        <div class="field">
          <label for="js-code">Invite Code</label>
          <input id="js-code" type="text" bind:value={inviteCode} placeholder="Enter invite code" />
        </div>
        <div class="actions">
          <button class="btn-ghost" onclick={() => ui.closeModal()}>Cancel</button>
          <button class="btn-primary" onclick={joinServer} disabled={joinSubmitting || !inviteCode.trim()}>
            {joinSubmitting ? 'Joining...' : 'Join Server'}
          </button>
        </div>
      </div>
    {/if}

    {#if modal === 'invite'}
      <div class="modal">
        <h2>Invite People</h2>
        {#if newInviteCode}
          <div class="invite-display">
            <code>{newInviteCode}</code>
            <button class="btn-primary" onclick={() => copyInvite(newInviteCode)}>Copy</button>
          </div>
        {:else}
          <button class="btn-primary" style="width:100%;" onclick={generateInvite}>
            Generate Invite Link
          </button>
        {/if}

        {#if invites.length > 0}
          <div class="invite-list">
            <h4 style="color:var(--text-muted);font-size:12px;text-transform:uppercase;margin:16px 0 8px;">Active Invites</h4>
            {#each invites as inv}
              <div class="invite-row">
                <code class="truncate">{inv.code}</code>
                <span class="invite-uses">{inv.uses}{inv.max_uses ? `/${inv.max_uses}` : ''} uses</span>
                <button class="btn-ghost" style="padding:4px 8px;" onclick={() => deleteInvite(inv.code)}>Delete</button>
              </div>
            {/each}
          </div>
        {/if}

        <div class="actions">
          <button class="btn-ghost" onclick={() => ui.closeModal()}>Done</button>
        </div>
      </div>
    {/if}

    {#if modal === 'settings'}
      <div class="modal settings-modal">
        <!-- Tab bar -->
        <div class="settings-tabs">
          <button
            class="settings-tab"
            class:active={settingsTab === 'general'}
            onclick={() => { settingsTab = 'general'; cancelRoleEdit(); }}
          >General</button>
          <button
            class="settings-tab"
            class:active={settingsTab === 'roles'}
            onclick={() => { settingsTab = 'roles'; }}
          >Roles</button>
        </div>

        {#if settingsTab === 'general'}
          <h2>Server Settings</h2>
          <div class="field">
            <label for="ss-name">Server Name</label>
            <input id="ss-name" type="text" bind:value={settingsName} maxlength={100} />
          </div>
          <div class="field">
            <label for="ss-desc">Description</label>
            <input id="ss-desc" type="text" bind:value={settingsDesc} maxlength={256} />
          </div>
          <div class="actions" style="justify-content: space-between;">
            <button class="btn-danger" onclick={deleteServer}>Delete Server</button>
            <div style="display:flex;gap:8px;">
              <button class="btn-ghost" onclick={() => ui.closeModal()}>Cancel</button>
              <button class="btn-primary" onclick={saveSettings} disabled={settingsSubmitting || !settingsName.trim()}>
                {settingsSubmitting ? 'Saving...' : 'Save'}
              </button>
            </div>
          </div>
        {:else if settingsTab === 'roles'}
          <div class="roles-panel">
            <div class="roles-header">
              <h2>Roles</h2>
              <button class="btn-primary" style="padding:6px 12px;font-size:13px;" onclick={startCreateRole}>
                + New Role
              </button>
            </div>

            {#if creatingRole}
              <div class="role-editor">
                <h3>Create Role</h3>
                <div class="field">
                  <label for="nr-name">Name</label>
                  <input id="nr-name" type="text" bind:value={newRoleName} placeholder="Role name" maxlength={100} />
                </div>
                <div class="field">
                  <label for="nr-color">Color</label>
                  <div class="color-row">
                    <input id="nr-color" type="color" bind:value={newRoleColor} class="color-picker" />
                    <span class="color-hex">{newRoleColor}</span>
                    <span class="color-preview" style="background:{newRoleColor};"></span>
                  </div>
                </div>
                <div class="field">
                  <label>Permissions</label>
                  <div class="permissions-grid">
                    {#each permissionDefs as perm}
                      <label class="perm-checkbox">
                        <input
                          type="checkbox"
                          checked={(newRolePermissions & perm.key) !== 0}
                          onchange={() => { newRolePermissions = togglePermission(newRolePermissions, perm.key); }}
                        />
                        <span>{perm.label}</span>
                      </label>
                    {/each}
                  </div>
                </div>
                <div class="actions">
                  <button class="btn-ghost" onclick={cancelRoleEdit}>Cancel</button>
                  <button class="btn-primary" onclick={createRole} disabled={roleSubmitting || !newRoleName.trim()}>
                    {roleSubmitting ? 'Creating...' : 'Create Role'}
                  </button>
                </div>
              </div>
            {:else if editingRole}
              <div class="role-editor">
                <h3>Edit Role</h3>
                <div class="field">
                  <label for="er-name">Name</label>
                  <input id="er-name" type="text" bind:value={editRoleName} maxlength={100} />
                </div>
                <div class="field">
                  <label for="er-color">Color</label>
                  <div class="color-row">
                    <input id="er-color" type="color" bind:value={editRoleColor} class="color-picker" />
                    <span class="color-hex">{editRoleColor}</span>
                    <span class="color-preview" style="background:{editRoleColor};"></span>
                  </div>
                </div>
                <div class="field">
                  <label>Permissions</label>
                  <div class="permissions-grid">
                    {#each permissionDefs as perm}
                      <label class="perm-checkbox">
                        <input
                          type="checkbox"
                          checked={(editRolePermissions & perm.key) !== 0}
                          onchange={() => { editRolePermissions = togglePermission(editRolePermissions, perm.key); }}
                        />
                        <span>{perm.label}</span>
                      </label>
                    {/each}
                  </div>
                </div>
                <div class="actions" style="justify-content: space-between;">
                  <button class="btn-danger" onclick={() => deleteRole(editingRole!)}>Delete Role</button>
                  <div style="display:flex;gap:8px;">
                    <button class="btn-ghost" onclick={cancelRoleEdit}>Cancel</button>
                    <button class="btn-primary" onclick={saveRole} disabled={roleSubmitting || !editRoleName.trim()}>
                      {roleSubmitting ? 'Saving...' : 'Save'}
                    </button>
                  </div>
                </div>
              </div>
            {:else}
              <!-- Role list -->
              <div class="role-list">
                {#each roles.sort((a, b) => b.position - a.position) as role}
                  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
                  <div class="role-item" class:default={role.is_default} onclick={() => startEditRole(role)} role="button" tabindex="0">
                    <span class="role-color-dot" style="background:{intToHex(role.color)};"></span>
                    <span class="role-name">{role.name}</span>
                    {#if role.is_default}
                      <span class="role-badge">default</span>
                    {/if}
                    {#if hasPermission(role.permissions, Permissions.ADMINISTRATOR)}
                      <span class="role-badge admin">admin</span>
                    {/if}
                  </div>
                {/each}
                {#if roles.length === 0}
                  <div class="empty-roles">No roles yet</div>
                {/if}
              </div>
            {/if}

            {#if !creatingRole && !editingRole}
              <div class="actions">
                <button class="btn-ghost" onclick={() => ui.closeModal()}>Done</button>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}

{#if modal === 'user-settings'}
  <UserSettings />
{/if}

<style>
  .invite-display {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--bg-input);
    padding: 8px 12px;
    border-radius: var(--radius-md);
    margin-bottom: 8px;
  }

  .invite-display code {
    flex: 1;
    font-size: 14px;
    background: none;
    padding: 0;
    color: var(--text-white);
    user-select: all;
  }

  .invite-list {
    max-height: 200px;
    overflow-y: auto;
  }

  .invite-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    border-bottom: 1px solid var(--border-subtle);
  }

  .invite-row code {
    flex: 1;
    font-size: 13px;
  }

  .invite-uses {
    font-size: 12px;
    color: var(--text-faint);
    white-space: nowrap;
  }

  /* ── Settings tabs ── */

  .settings-modal {
    max-width: 600px;
  }

  .settings-tabs {
    display: flex;
    gap: 4px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 0;
  }

  .settings-tab {
    background: none;
    color: var(--text-muted);
    padding: 8px 16px;
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    font-size: 14px;
    font-weight: 500;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: color var(--transition), border-color var(--transition);
  }

  .settings-tab:hover {
    color: var(--text-normal);
  }

  .settings-tab.active {
    color: var(--text-white);
    border-bottom-color: var(--bg-accent);
  }

  /* ── Roles panel ── */

  .roles-panel h2 {
    margin-bottom: 0;
  }

  .roles-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .role-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 300px;
    overflow-y: auto;
  }

  .role-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--transition);
  }

  .role-item:hover {
    background: var(--bg-hover);
  }

  .role-item.default {
    cursor: default;
    opacity: 0.7;
  }

  .role-color-dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    flex-shrink: 0;
    border: 2px solid var(--border-subtle);
  }

  .role-name {
    flex: 1;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-normal);
  }

  .role-badge {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.02em;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-overlay);
    color: var(--text-faint);
  }

  .role-badge.admin {
    background: rgba(218, 55, 60, 0.2);
    color: var(--text-danger);
  }

  .empty-roles {
    text-align: center;
    color: var(--text-faint);
    padding: 24px;
    font-size: 14px;
  }

  /* ── Role editor ── */

  .role-editor {
    border-top: 1px solid var(--border-subtle);
    padding-top: 16px;
  }

  .role-editor h3 {
    font-size: 16px;
    margin-bottom: 12px;
  }

  .color-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .color-picker {
    width: 40px;
    height: 36px;
    padding: 2px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    background: var(--bg-input);
    cursor: pointer;
  }

  .color-hex {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text-muted);
  }

  .color-preview {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 2px solid var(--border-subtle);
  }

  /* ── Permissions grid ── */

  .permissions-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px 16px;
    max-height: 220px;
    overflow-y: auto;
    padding: 8px;
    background: var(--bg-input);
    border-radius: var(--radius-md);
  }

  .perm-checkbox {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-normal);
    padding: 4px 0;
  }

  .perm-checkbox input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--bg-accent);
    cursor: pointer;
    flex-shrink: 0;
  }

  .perm-checkbox span {
    user-select: none;
  }
</style>
