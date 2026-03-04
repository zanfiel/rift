<script lang="ts">
  import * as api from '../api';
  import { auth } from '../stores/auth';
  import type { DmChannel, DmMessage } from '../types';

  let dmChannels: DmChannel[] = $state([]);
  let selectedDm: DmChannel | null = $state(null);
  let dmMessages: DmMessage[] = $state([]);
  let loading: boolean = $state(false);
  let inputValue: string = $state('');
  let scrollContainer: HTMLDivElement | undefined = $state();

  // Load DM channels on mount
  $effect(() => {
    loadDms();
  });

  async function loadDms() {
    try {
      dmChannels = await api.listDms();
    } catch (e) {
      console.error('Failed to load DMs:', e);
    }
  }

  async function selectDm(dm: DmChannel) {
    selectedDm = dm;
    loading = true;
    try {
      const msgs = await api.listDmMessages(dm.id, { limit: 50 });
      dmMessages = msgs.reverse();
    } catch (e) {
      console.error('Failed to load DM messages:', e);
      dmMessages = [];
    }
    loading = false;
    requestAnimationFrame(() => {
      if (scrollContainer) scrollContainer.scrollTop = scrollContainer.scrollHeight;
    });
  }

  async function sendDm() {
    if (!selectedDm || !inputValue.trim()) return;
    const content = inputValue.trim();
    inputValue = '';
    try {
      const msg = await api.sendDmMessage(selectedDm.id, content);
      dmMessages = [...dmMessages, msg];
      requestAnimationFrame(() => {
        if (scrollContainer) scrollContainer.scrollTop = scrollContainer.scrollHeight;
      });
    } catch {
      inputValue = content;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendDm();
    }
  }

  function formatTime(iso: string): string {
    const d = new Date(iso);
    const now = new Date();
    if (d.getDate() === now.getDate()) {
      return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }
    return d.toLocaleDateString([], { month: 'short', day: 'numeric' }) +
      ' ' + d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }
</script>

<div class="dm-layout">
  <!-- DM Channel List -->
  <div class="dm-sidebar">
    <div class="dm-header">
      <h3>Direct Messages</h3>
    </div>
    <div class="dm-list">
      {#each dmChannels as dm}
        <button
          class="dm-item"
          class:active={selectedDm?.id === dm.id}
          onclick={() => selectDm(dm)}
        >
          <div class="avatar sm">
            {#if dm.other_avatar_url}
              <img src={dm.other_avatar_url} alt={dm.other_username} />
            {:else}
              {dm.other_username.charAt(0).toUpperCase()}
            {/if}
          </div>
          <span class="truncate">{dm.other_display_name ?? dm.other_username}</span>
        </button>
      {/each}
      {#if dmChannels.length === 0}
        <div class="dm-empty">No conversations yet</div>
      {/if}
    </div>
  </div>

  <!-- DM Chat Area -->
  <div class="dm-chat">
    {#if selectedDm}
      <div class="dm-chat-header">
        <span class="at-symbol">@</span>
        <span class="dm-chat-name">{selectedDm.other_display_name ?? selectedDm.other_username}</span>
      </div>

      <div class="dm-messages" bind:this={scrollContainer}>
        {#if loading}
          <div class="loading-center"><div class="spinner"></div></div>
        {:else if dmMessages.length === 0}
          <div class="dm-start">
            <div class="avatar xl">
              {selectedDm.other_username.charAt(0).toUpperCase()}
            </div>
            <h3>{selectedDm.other_display_name ?? selectedDm.other_username}</h3>
            <p>This is the beginning of your direct message history.</p>
          </div>
        {:else}
          {#each dmMessages as msg (msg.id)}
            <div class="dm-msg" class:own={msg.sender_id === auth.user?.id}>
              <div class="dm-msg-content">{msg.content}</div>
              <span class="dm-msg-time">{formatTime(msg.created_at)}</span>
            </div>
          {/each}
        {/if}
      </div>

      <div class="dm-input-wrapper">
        <textarea
          bind:value={inputValue}
          onkeydown={handleKeydown}
          placeholder="Message @{selectedDm.other_display_name ?? selectedDm.other_username}"
          rows={1}
          class="dm-textarea"
        ></textarea>
      </div>
    {:else}
      <div class="dm-placeholder">
        <h2>Select a conversation</h2>
        <p>Or start a new one from a server member list</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .dm-layout {
    flex: 1;
    display: flex;
    min-width: 0;
  }

  .dm-sidebar {
    width: 240px;
    background: var(--bg-secondary);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .dm-header {
    height: 48px;
    padding: 0 16px;
    display: flex;
    align-items: center;
    border-bottom: 2px solid var(--bg-tertiary);
  }

  .dm-header h3 {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-muted);
  }

  .dm-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .dm-item {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 8px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-size: 14px;
    cursor: pointer;
    text-align: left;
  }

  .dm-item:hover {
    background: var(--bg-hover);
    color: var(--text-normal);
  }

  .dm-item.active {
    background: var(--bg-active);
    color: var(--text-white);
  }

  .dm-empty {
    text-align: center;
    color: var(--text-faint);
    padding: 24px;
    font-size: 13px;
  }

  .dm-chat {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: var(--bg-primary);
  }

  .dm-chat-header {
    height: 48px;
    padding: 0 16px;
    display: flex;
    align-items: center;
    gap: 6px;
    border-bottom: 2px solid var(--bg-tertiary);
    flex-shrink: 0;
  }

  .at-symbol {
    color: var(--text-faint);
    font-size: 18px;
    font-weight: 600;
  }

  .dm-chat-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-white);
  }

  .dm-messages {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .loading-center {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }

  .dm-start {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 24px;
    text-align: center;
  }

  .dm-start h3 {
    margin-top: 12px;
    font-size: 20px;
  }

  .dm-start p {
    color: var(--text-muted);
    margin-top: 4px;
  }

  .dm-msg {
    margin-bottom: 8px;
    max-width: 70%;
  }

  .dm-msg.own {
    margin-left: auto;
  }

  .dm-msg-content {
    background: var(--bg-secondary);
    padding: 8px 12px;
    border-radius: var(--radius-md);
    color: var(--text-normal);
    line-height: 1.375;
    word-wrap: break-word;
    white-space: pre-wrap;
  }

  .dm-msg.own .dm-msg-content {
    background: var(--bg-accent);
    color: var(--text-white);
  }

  .dm-msg-time {
    font-size: 10px;
    color: var(--text-faint);
    margin-top: 2px;
    display: block;
  }

  .dm-msg.own .dm-msg-time {
    text-align: right;
  }

  .dm-placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-faint);
  }

  .dm-placeholder h2 {
    color: var(--text-muted);
  }

  .dm-input-wrapper {
    padding: 0 16px 24px;
    flex-shrink: 0;
  }

  .dm-textarea {
    width: 100%;
    background: var(--bg-input);
    border: none;
    border-radius: var(--radius-md);
    color: var(--text-normal);
    font-family: var(--font-sans);
    font-size: 14px;
    line-height: 1.375;
    padding: 11px 16px;
    resize: none;
    outline: none;
  }
  .dm-textarea:focus {
    box-shadow: none;
  }
  .dm-textarea::placeholder {
    color: var(--text-faint);
  }
</style>
