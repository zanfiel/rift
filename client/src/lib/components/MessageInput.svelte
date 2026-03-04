<script lang="ts">
  import { messages } from '../stores/messages';

  let inputValue: string = $state('');
  let inputEl: HTMLTextAreaElement | undefined = $state();

  interface Props {
    channelName?: string;
  }

  let { channelName = 'general' }: Props = $props();

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
    if (!content) return;

    inputValue = '';
    try {
      await messages.send(content);
    } catch {
      // Restore on failure
      inputValue = content;
    }

    // Refocus input
    inputEl?.focus();
  }

  // Auto-resize textarea
  function handleInput() {
    if (inputEl) {
      inputEl.style.height = 'auto';
      inputEl.style.height = Math.min(inputEl.scrollHeight, 300) + 'px';
    }
  }
</script>

<div class="message-input-wrapper">
  <div class="message-input-container">
    <textarea
      bind:this={inputEl}
      bind:value={inputValue}
      onkeydown={handleKeydown}
      oninput={handleInput}
      placeholder="Message #{channelName}"
      rows={1}
      class="message-textarea"
    ></textarea>
  </div>
</div>

<style>
  .message-input-wrapper {
    padding: 0 16px 24px;
    flex-shrink: 0;
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
    padding: 11px 16px;
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
</style>
