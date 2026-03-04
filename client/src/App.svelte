<script lang="ts">
  import { auth } from './lib/stores/auth';
  import { servers } from './lib/stores/servers';
  import { messages } from './lib/stores/messages';
  import { ui } from './lib/stores/ui';
  import { gateway } from './lib/gateway';
  import type { GatewayEvent } from './lib/types';

  import AuthPage from './lib/components/AuthPage.svelte';
  import ServerSidebar from './lib/components/ServerSidebar.svelte';
  import ChannelSidebar from './lib/components/ChannelSidebar.svelte';
  import ChatArea from './lib/components/ChatArea.svelte';
  import MemberList from './lib/components/MemberList.svelte';
  import DmView from './lib/components/DmView.svelte';
  import Modals from './lib/components/Modals.svelte';

  let loggedIn: boolean = $state(false);
  let appLoading: boolean = $state(true);
  let currentView: typeof ui.view = $state('servers');
  let memberListVisible: boolean = $state(true);

  // Init auth on mount
  $effect(() => {
    auth.init();
  });

  // Subscribe to auth state
  $effect(() => {
    const unsub = auth.subscribe(() => {
      loggedIn = auth.loggedIn;
      appLoading = auth.loading;

      // Load servers when logged in
      if (auth.loggedIn) {
        servers.load();
      } else {
        servers.clear();
        messages.clear();
      }
    });
    return unsub;
  });

  // Subscribe to UI state
  $effect(() => {
    const unsub = ui.subscribe(() => {
      currentView = ui.view;
      memberListVisible = ui.memberListVisible;
    });
    return unsub;
  });

  // Gateway event dispatch — route events to the right stores
  $effect(() => {
    const unsub = gateway.on((event: GatewayEvent) => {
      const t: string = event.type;

      // Message-related events
      if (t === 'MessageCreate' || t === 'MessageUpdate' ||
          t === 'MessageDelete' || t === 'TypingStart') {
        messages.handleEvent(event);
      }

      // Server-related events
      if (t === 'ChannelCreate' || t === 'ChannelDelete' ||
          t === 'MemberJoin' || t === 'MemberLeave' ||
          t === 'RoleCreate' || t === 'RoleUpdate' ||
          t === 'RoleDelete') {
        servers.handleEvent({ type: t, data: (event as any).data });
      }
    });
    return unsub;
  });
</script>

{#if appLoading}
  <div class="loading-screen">
    <div class="spinner"></div>
  </div>
{:else if !loggedIn}
  <AuthPage />
{:else}
  <div class="app-layout">
    <ServerSidebar />

    {#if currentView === 'dms'}
      <DmView />
    {:else}
      <ChannelSidebar />
      <ChatArea />
      {#if memberListVisible}
        <MemberList />
      {/if}
    {/if}
  </div>

  <Modals />
{/if}

<style>
  .loading-screen {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
  }

  .app-layout {
    height: 100%;
    display: flex;
    overflow: hidden;
  }
</style>
