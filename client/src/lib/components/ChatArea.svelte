<script lang="ts">
  import { messages } from '../stores/messages';
  import { servers } from '../stores/servers';
  import { auth } from '../stores/auth';
  import MessageItem from './MessageItem.svelte';
  import MessageInput from './MessageInput.svelte';
  import type { MessageWithAuthor } from '../types';

  let messageList: MessageWithAuthor[] = $state([]);
  let channelName: string = $state('general');
  let typingUsers: string[] = $state([]);
  let loading: boolean = $state(false);
  let scrollContainer: HTMLDivElement | undefined = $state();
  let shouldAutoScroll: boolean = $state(true);
  let prevMessageCount: number = $state(0);

  // Subscribe to message store
  $effect(() => {
    const unsub = messages.subscribe(() => {
      const newList = messages.list;
      // Auto-scroll if user was at bottom
      if (newList.length > prevMessageCount && shouldAutoScroll) {
        requestAnimationFrame(() => scrollToBottom());
      }
      prevMessageCount = newList.length;
      messageList = newList;
      typingUsers = messages.typingUsers;
      loading = messages.loading;
    });
    return unsub;
  });

  // Track current channel name
  $effect(() => {
    const unsub = servers.subscribe(() => {
      const ch = servers.currentChannel;
      if (ch) channelName = ch.name;
    });
    return unsub;
  });

  // Load messages when channel changes
  $effect(() => {
    const unsub = servers.subscribe(() => {
      const chId = servers.currentChannelId;
      if (chId && chId !== messages.channelId) {
        messages.load(chId).then(() => {
          requestAnimationFrame(() => scrollToBottom());
        });
      }
    });
    return unsub;
  });

  function scrollToBottom() {
    if (scrollContainer) {
      scrollContainer.scrollTop = scrollContainer.scrollHeight;
    }
  }

  function handleScroll() {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    // Check if near bottom
    shouldAutoScroll = scrollHeight - scrollTop - clientHeight < 50;
    // Load older messages when scrolled to top
    if (scrollTop < 100 && messages.hasMore && !messages.loading) {
      const prevHeight = scrollContainer.scrollHeight;
      messages.loadOlder().then(() => {
        // Maintain scroll position
        if (scrollContainer) {
          const newHeight = scrollContainer.scrollHeight;
          scrollContainer.scrollTop = newHeight - prevHeight;
        }
      });
    }
  }

  /** Should this message show as compact (same author within 7 minutes)? */
  function isCompact(msg: MessageWithAuthor, idx: number): boolean {
    if (idx === 0) return false;
    const prev = messageList[idx - 1];
    if (prev.author_id !== msg.author_id) return false;
    const diff = new Date(msg.created_at).getTime() - new Date(prev.created_at).getTime();
    return diff < 7 * 60 * 1000;
  }

  let typingText: string = $derived.by(() => {
    const users = typingUsers.filter(u => u !== auth.user?.username);
    if (users.length === 0) return '';
    if (users.length === 1) return `${users[0]} is typing...`;
    if (users.length === 2) return `${users[0]} and ${users[1]} are typing...`;
    return `${users[0]} and ${users.length - 1} others are typing...`;
  });
</script>

<div class="chat-area">
  <!-- Channel Header -->
  <div class="channel-header">
    <span class="channel-hash">#</span>
    <span class="channel-name">{channelName}</span>
    {#if servers.currentChannel?.topic}
      <span class="divider-vert"></span>
      <span class="channel-topic truncate">{servers.currentChannel.topic}</span>
    {/if}
  </div>

  <!-- Messages -->
  <div class="messages-scroll" bind:this={scrollContainer} onscroll={handleScroll}>
    {#if loading && messageList.length === 0}
      <div class="loading-center">
        <div class="spinner"></div>
      </div>
    {:else if messageList.length === 0}
      <div class="empty-channel">
        <div class="empty-icon">#</div>
        <h3>Welcome to #{channelName}!</h3>
        <p>This is the beginning of the #{channelName} channel.</p>
      </div>
    {:else}
      {#if loading}
        <div class="loading-top"><div class="spinner"></div></div>
      {/if}
      {#each messageList as msg, i (msg.id)}
        <MessageItem message={msg} compact={isCompact(msg, i)} />
      {/each}
    {/if}
  </div>

  <!-- Typing indicator -->
  <div class="typing-bar">
    {#if typingText}
      <span class="typing-dots">
        <span></span><span></span><span></span>
      </span>
      <span class="typing-text">{typingText}</span>
    {/if}
  </div>

  <!-- Input -->
  <MessageInput {channelName} />
</div>

<style>
  .chat-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: var(--bg-primary);
  }

  .channel-header {
    height: 48px;
    padding: 0 16px;
    display: flex;
    align-items: center;
    gap: 8px;
    border-bottom: 2px solid var(--bg-tertiary);
    flex-shrink: 0;
  }

  .channel-hash {
    color: var(--text-faint);
    font-size: 22px;
    font-weight: 500;
  }

  .channel-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-white);
  }

  .divider-vert {
    width: 1px;
    height: 24px;
    background: var(--border-subtle);
  }

  .channel-topic {
    color: var(--text-muted);
    font-size: 13px;
  }

  .messages-scroll {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding-bottom: 8px;
  }

  .loading-center {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }

  .loading-top {
    display: flex;
    justify-content: center;
    padding: 16px;
  }

  .empty-channel {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 16px;
    margin: 16px;
  }

  .empty-icon {
    width: 68px;
    height: 68px;
    border-radius: var(--radius-full);
    background: var(--bg-overlay);
    color: var(--text-muted);
    font-size: 38px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 12px;
  }

  .empty-channel h3 {
    font-size: 24px;
    margin-bottom: 8px;
  }

  .empty-channel p {
    color: var(--text-muted);
  }

  .typing-bar {
    height: 24px;
    padding: 0 16px;
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-muted);
  }

  .typing-dots {
    display: inline-flex;
    gap: 2px;
    align-items: center;
  }
  .typing-dots span {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--text-muted);
    animation: typing-bounce 1.4s ease-in-out infinite;
  }
  .typing-dots span:nth-child(2) { animation-delay: 0.2s; }
  .typing-dots span:nth-child(3) { animation-delay: 0.4s; }

  @keyframes typing-bounce {
    0%, 80%, 100% { transform: translateY(0); }
    40% { transform: translateY(-4px); }
  }

  .typing-text {
    font-weight: 500;
  }
</style>
