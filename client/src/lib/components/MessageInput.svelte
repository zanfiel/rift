<script lang="ts">
  import { messages } from '../stores/messages';

  let inputValue: string = $state('');
  let inputEl: HTMLTextAreaElement | undefined = $state();
  let fileInputEl: HTMLInputElement | undefined = $state();
  let stagedFiles: File[] = $state([]);
  let sending: boolean = $state(false);
  let dragOver: boolean = $state(false);

  interface Props {
    channelName?: string;
  }

  let { channelName = 'general' }: Props = $props();

  // Preview URLs for staged images
  let previews: { file: File; url: string; isImage: boolean }[] = $derived.by(() => {
    return stagedFiles.map(f => ({
      file: f,
      url: f.type.startsWith('image/') ? URL.createObjectURL(f) : '',
      isImage: f.type.startsWith('image/'),
    }));
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
    // Typing indicator
    messages.sendTyping();
  }

  async function sendMessage() {
    const content = inputValue.trim();
    if (!content && stagedFiles.length === 0) return;
    if (sending) return;

    const filesToSend = stagedFiles.length > 0 ? [...stagedFiles] : undefined;
    const textToSend = content;

    inputValue = '';
    stagedFiles = [];
    sending = true;

    try {
      await messages.send(textToSend, filesToSend);
    } catch {
      // Restore on failure
      inputValue = textToSend;
      if (filesToSend) stagedFiles = filesToSend;
    } finally {
      sending = false;
    }

    // Reset textarea height
    if (inputEl) {
      inputEl.style.height = 'auto';
    }
    inputEl?.focus();
  }

  // Auto-resize textarea
  function handleInput() {
    if (inputEl) {
      inputEl.style.height = 'auto';
      inputEl.style.height = Math.min(inputEl.scrollHeight, 300) + 'px';
    }
  }

  function handlePaste(e: ClipboardEvent) {
    const items = e.clipboardData?.items;
    if (!items) return;

    const files: File[] = [];
    for (const item of items) {
      if (item.kind === 'file') {
        const f = item.getAsFile();
        if (f) files.push(f);
      }
    }
    if (files.length > 0) {
      e.preventDefault();
      addFiles(files);
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    dragOver = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    const files: File[] = [];
    if (e.dataTransfer?.files) {
      for (const f of e.dataTransfer.files) {
        files.push(f);
      }
    }
    if (files.length > 0) {
      addFiles(files);
    }
  }

  function openFilePicker() {
    fileInputEl?.click();
  }

  function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      addFiles(Array.from(input.files));
      input.value = ''; // Reset so same file can be re-selected
    }
  }

  function addFiles(files: File[]) {
    // Max 10 files at once, 25MB each
    const MAX_SIZE = 25 * 1024 * 1024;
    const MAX_FILES = 10;
    const remaining = MAX_FILES - stagedFiles.length;
    const toAdd = files.slice(0, remaining).filter(f => f.size <= MAX_SIZE);
    if (toAdd.length > 0) {
      stagedFiles = [...stagedFiles, ...toAdd];
    }
  }

  function removeFile(index: number) {
    stagedFiles = stagedFiles.filter((_, i) => i !== index);
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  }
</script>

<div
  class="message-input-wrapper"
  class:drag-over={dragOver}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  role="region"
>
  {#if stagedFiles.length > 0}
    <div class="staged-files">
      {#each previews as preview, i (preview.file.name + i)}
        <div class="staged-file">
          {#if preview.isImage}
            <img src={preview.url} alt={preview.file.name} class="staged-preview" />
          {:else}
            <div class="staged-icon">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8l-6-6zm4 18H6V4h7v5h5v11z"/>
              </svg>
            </div>
          {/if}
          <span class="staged-name" title={preview.file.name}>
            {preview.file.name}
          </span>
          <span class="staged-size">{formatSize(preview.file.size)}</span>
          <button class="staged-remove" onclick={() => removeFile(i)} title="Remove">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
              <path d="M3.5 3.5l7 7m0-7l-7 7" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round"/>
            </svg>
          </button>
        </div>
      {/each}
    </div>
  {/if}

  <div class="message-input-container">
    <button class="attach-btn" onclick={openFilePicker} title="Attach files" disabled={sending}>
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21.44 11.05l-9.19 9.19a6 6 0 01-8.49-8.49l9.19-9.19a4 4 0 015.66 5.66l-9.2 9.19a2 2 0 01-2.83-2.83l8.49-8.48"/>
      </svg>
    </button>

    <textarea
      bind:this={inputEl}
      bind:value={inputValue}
      onkeydown={handleKeydown}
      oninput={handleInput}
      onpaste={handlePaste}
      placeholder="Message #{channelName}"
      rows={1}
      class="message-textarea"
      disabled={sending}
    ></textarea>

    {#if stagedFiles.length > 0 || inputValue.trim()}
      <button class="send-btn" onclick={sendMessage} title="Send" disabled={sending}>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
        </svg>
      </button>
    {/if}
  </div>

  <input
    bind:this={fileInputEl}
    type="file"
    multiple
    class="hidden-file-input"
    onchange={handleFileSelect}
  />

  {#if dragOver}
    <div class="drop-overlay">
      <div class="drop-label">Drop files to upload</div>
    </div>
  {/if}
</div>

<style>
  .message-input-wrapper {
    padding: 0 16px 24px;
    flex-shrink: 0;
    position: relative;
  }

  .message-input-container {
    background: var(--bg-input);
    border-radius: var(--radius-md);
    display: flex;
    align-items: flex-end;
  }

  .message-textarea {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-normal);
    font-family: var(--font-sans);
    font-size: 14px;
    line-height: 1.375;
    padding: 11px 4px 11px 0;
    resize: none;
    max-height: 300px;
    overflow-y: auto;
  }
  .message-textarea:focus {
    box-shadow: none;
  }
  .message-textarea::placeholder {
    color: var(--text-faint);
  }
  .message-textarea:disabled {
    opacity: 0.6;
  }

  .attach-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    padding: 10px 8px 10px 12px;
    cursor: pointer;
    display: flex;
    align-items: center;
    flex-shrink: 0;
    border-radius: 0;
  }
  .attach-btn:hover {
    color: var(--text-normal);
  }
  .attach-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .send-btn {
    background: none;
    border: none;
    color: var(--bg-accent);
    padding: 10px 12px 10px 8px;
    cursor: pointer;
    display: flex;
    align-items: center;
    flex-shrink: 0;
    border-radius: 0;
  }
  .send-btn:hover {
    color: var(--bg-accent-hover);
  }
  .send-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .hidden-file-input {
    display: none;
  }

  /* Staged files preview area */
  .staged-files {
    background: var(--bg-input);
    border-radius: var(--radius-md) var(--radius-md) 0 0;
    padding: 12px 12px 8px;
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    border-bottom: 1px solid var(--border-subtle);
  }

  .staged-files + .message-input-container {
    border-radius: 0 0 var(--radius-md) var(--radius-md);
  }

  .staged-file {
    position: relative;
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 120px;
    gap: 4px;
  }

  .staged-preview {
    width: 100%;
    height: 80px;
    object-fit: cover;
    border-radius: 4px;
  }

  .staged-icon {
    width: 100%;
    height: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
  }

  .staged-name {
    font-size: 11px;
    color: var(--text-normal);
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .staged-size {
    font-size: 10px;
    color: var(--text-faint);
  }

  .staged-remove {
    position: absolute;
    top: 4px;
    right: 4px;
    background: var(--bg-overlay);
    border: none;
    color: var(--text-muted);
    width: 20px;
    height: 20px;
    border-radius: var(--radius-full);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }
  .staged-remove:hover {
    color: var(--text-danger);
    background: var(--bg-secondary);
  }

  /* Drag-drop overlay */
  .drag-over {
    position: relative;
  }

  .drop-overlay {
    position: absolute;
    inset: 0;
    background: rgba(88, 101, 242, 0.12);
    border: 2px dashed var(--bg-accent);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
    pointer-events: none;
  }

  .drop-label {
    font-size: 16px;
    font-weight: 600;
    color: var(--bg-accent);
  }
</style>
