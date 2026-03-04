<script lang="ts">
  import { ui } from '../stores/ui';
  import { servers } from '../stores/servers';
  import * as api from '../api';
  import type { Invite } from '../types';

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
  let settingsName: string = $state('');
  let settingsDesc: string = $state('');
  let settingsSubmitting: boolean = $state(false);

  $effect(() => {
    if (modal === 'settings' && servers.current) {
      settingsName = servers.current.server.name;
      settingsDesc = servers.current.server.description ?? '';
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
      <div class="modal">
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
      </div>
    {/if}
  </div>
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
</style>
