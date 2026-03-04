<script lang="ts">
  import type { MessageWithAuthor, Attachment } from '../types';
  import { auth } from '../stores/auth';
  import { messages } from '../stores/messages';

  interface Props {
    message: MessageWithAuthor;
    compact?: boolean;
  }

  let { message, compact = false }: Props = $props();

  let editing: boolean = $state(false);
  let editContent: string = $state('');
  let showActions: boolean = $state(false);

  const isOwn = $derived(message.author_id === auth.user?.id);
  const displayName = $derived(message.author_display_name ?? message.author_username);
  const initial = $derived(message.author_username.charAt(0).toUpperCase());

  let imageAttachments: Attachment[] = $derived.by(() => {
    return (message.attachments ?? []).filter(a => a.content_type?.startsWith('image/'));
  });

  let fileAttachments: Attachment[] = $derived.by(() => {
    return (message.attachments ?? []).filter(a => !a.content_type?.startsWith('image/'));
  });

  function formatTime(iso: string): string {
    const d = new Date(iso);
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    const hours = diff / 3600000;

    if (hours < 24 && d.getDate() === now.getDate()) {
      return `Today at ${d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
    }
    const yesterday = new Date(now);
    yesterday.setDate(yesterday.getDate() - 1);
    if (hours < 48 && d.getDate() === yesterday.getDate()) {
      return `Yesterday at ${d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
    }
    return d.toLocaleDateString([], { month: '2-digit', day: '2-digit', year: 'numeric' }) +
      ' ' + d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  function formatSize(bytes: number | null): string {
    if (bytes == null) return '';
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  }

  function startEdit() {
    editContent = message.content;
    editing = true;
  }

  async function saveEdit() {
    if (editContent.trim() && editContent !== message.content) {
      try {
        await messages.edit(message.id, editContent.trim());
      } catch { /* error handled in store */ }
    }
    editing = false;
  }

  function cancelEdit() {
    editing = false;
  }

  async function handleDelete() {
    if (confirm('Delete this message?')) {
      await messages.delete(message.id);
    }
  }

  function handleEditKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      saveEdit();
    }
    if (e.key === 'Escape') {
      cancelEdit();
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="message"
  class:compact
  class:own={isOwn}
  onmouseenter={() => showActions = true}
  onmouseleave={() => showActions = false}
>
  {#if !compact}
    <div class="avatar">
      {#if message.author_avatar_url}
        <img src={message.author_avatar_url} alt={message.author_username} />
      {:else}
        {initial}
      {/if}
    </div>
  {:else}
    <span class="compact-time" title={formatTime(message.created_at)}>
      {new Date(message.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
    </span>
  {/if}

  <div class="message-body">
    {#if !compact}
      <div class="message-header">
        <span class="author-name">{displayName}</span>
        <span class="timestamp">{formatTime(message.created_at)}</span>
        {#if message.edited_at}
          <span class="edited" title={formatTime(message.edited_at)}>(edited)</span>
        {/if}
      </div>
    {/if}

    {#if editing}
      <textarea
        class="edit-input"
        bind:value={editContent}
        onkeydown={handleEditKeydown}
      ></textarea>
      <div class="edit-hint">
        escape to <button class="btn-link" onclick={cancelEdit}>cancel</button> &middot; enter to <button class="btn-link" onclick={saveEdit}>save</button>
      </div>
    {:else}
      {#if message.content}
        <div class="content">{message.content}</div>
      {/if}
      {#if compact && message.edited_at}
        <span class="edited" title={formatTime(message.edited_at)}>(edited)</span>
      {/if}

      {#if imageAttachments.length > 0}
        <div class="attachments-images">
          {#each imageAttachments as att (att.id)}
            <a href={att.url} target="_blank" rel="noopener" class="image-attachment">
              <img src={att.url} alt={att.filename} loading="lazy" />
            </a>
          {/each}
        </div>
      {/if}

      {#if fileAttachments.length > 0}
        <div class="attachments-files">
          {#each fileAttachments as att (att.id)}
            <a href={att.url} download={att.filename} class="file-attachment">
              <div class="file-icon">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8l-6-6zm4 18H6V4h7v5h5v11z"/>
                </svg>
              </div>
              <div class="file-info">
                <span class="file-name">{att.filename}</span>
                <span class="file-size">{formatSize(att.size_bytes)}</span>
              </div>
              <div class="file-download">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
                  <polyline points="7 10 12 15 17 10"/>
                  <line x1="12" y1="15" x2="12" y2="3"/>
                </svg>
              </div>
            </a>
          {/each}
        </div>
      {/if}
    {/if}
  </div>

  {#if showActions && !editing}
    <div class="actions">
      {#if isOwn}
        <button class="action-btn" title="Edit" onclick={startEdit}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M12.146 1.146a.5.5 0 01.708 0l2 2a.5.5 0 010 .708l-9.5 9.5a.5.5 0 01-.168.11l-4 1.5a.5.5 0 01-.65-.65l1.5-4a.5.5 0 01.11-.168l9.5-9.5z"/>
          </svg>
        </button>
        <button class="action-btn danger" title="Delete" onclick={handleDelete}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <path d="M5.5 5.5a.5.5 0 01.5.5v6a.5.5 0 01-1 0V6a.5.5 0 01.5-.5zm2.5.5a.5.5 0 011 0v6a.5.5 0 01-1 0V6zm3.5-.5a.5.5 0 01.5.5v6a.5.5 0 01-1 0V6a.5.5 0 01.5-.5z"/>
            <path d="M14.5 3a1 1 0 01-1 1H13v9a2 2 0 01-2 2H5a2 2 0 01-2-2V4h-.5a1 1 0 010-2h3a1 1 0 011-1h3a1 1 0 011 1h3a1 1 0 011 1zM4.118 4L4 4.059V13a1 1 0 001 1h6a1 1 0 001-1V4.059L11.882 4H4.118z"/>
          </svg>
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .message {
    position: relative;
    display: flex;
    gap: 16px;
    padding: 2px 16px 2px 72px;
    min-height: 2.75rem;
    align-items: flex-start;
  }

  .message:not(.compact) {
    margin-top: 16px;
    padding-top: 2px;
  }

  .message:hover {
    background: var(--bg-hover);
  }

  .avatar {
    position: absolute;
    left: 16px;
    top: 2px;
    width: 40px;
    height: 40px;
    border-radius: var(--radius-full);
    background: var(--bg-accent);
    color: var(--text-white);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 16px;
    flex-shrink: 0;
    overflow: hidden;
    cursor: pointer;
  }
  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .compact-time {
    position: absolute;
    left: 16px;
    width: 40px;
    font-size: 10px;
    color: transparent;
    text-align: right;
    user-select: none;
  }
  .message.compact:hover .compact-time {
    color: var(--text-faint);
  }

  .message-body {
    flex: 1;
    min-width: 0;
  }

  .message-header {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .author-name {
    font-weight: 600;
    color: var(--text-white);
    font-size: 14px;
    cursor: pointer;
  }
  .author-name:hover {
    text-decoration: underline;
  }

  .timestamp {
    font-size: 11px;
    color: var(--text-faint);
  }

  .edited {
    font-size: 10px;
    color: var(--text-faint);
  }

  .content {
    color: var(--text-normal);
    line-height: 1.375;
    word-wrap: break-word;
    white-space: pre-wrap;
  }

  .edit-input {
    width: 100%;
    min-height: 40px;
    max-height: 200px;
    resize: none;
    font-size: 14px;
    line-height: 1.375;
    margin-top: 4px;
  }

  .edit-hint {
    font-size: 11px;
    color: var(--text-faint);
    margin-top: 4px;
  }

  .actions {
    position: absolute;
    top: -16px;
    right: 16px;
    display: flex;
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    overflow: hidden;
    box-shadow: var(--shadow-low);
  }

  .action-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    padding: 6px 8px;
    cursor: pointer;
    display: flex;
    border-radius: 0;
  }
  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-normal);
  }
  .action-btn.danger:hover {
    color: var(--text-danger);
  }

  /* Attachment styles */
  .attachments-images {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 4px;
  }

  .image-attachment {
    display: block;
    max-width: 400px;
    border-radius: var(--radius-md);
    overflow: hidden;
    cursor: pointer;
  }

  .image-attachment img {
    display: block;
    max-width: 100%;
    max-height: 350px;
    object-fit: contain;
    border-radius: var(--radius-md);
  }

  .image-attachment:hover img {
    opacity: 0.9;
  }

  .attachments-files {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 4px;
  }

  .file-attachment {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 8px 12px;
    max-width: 400px;
    text-decoration: none;
    color: inherit;
    cursor: pointer;
    transition: background 0.15s;
  }

  .file-attachment:hover {
    background: var(--bg-hover);
  }

  .file-icon {
    color: var(--text-muted);
    flex-shrink: 0;
    display: flex;
  }

  .file-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .file-name {
    font-size: 13px;
    color: var(--text-link);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-size {
    font-size: 11px;
    color: var(--text-faint);
  }

  .file-download {
    color: var(--text-muted);
    flex-shrink: 0;
    display: flex;
  }

  .file-attachment:hover .file-download {
    color: var(--text-normal);
  }
</style>
