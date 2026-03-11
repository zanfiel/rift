<script lang="ts">
  import { auth } from '../stores/auth';
  import { ui } from '../stores/ui';
  import * as api from '../api';
  import type { PublicUser } from '../types';

  // ── Reactive user state ──
  let currentUser: PublicUser | null = $state(null);

  $effect(() => {
    const unsub = auth.subscribe(() => {
      currentUser = auth.user;
    });
    return unsub;
  });

  let activeTab: 'profile' | 'account' = $state('profile');

  // ── Profile tab state ──
  let displayName: string = $state('');
  let about: string = $state('');
  let avatarPreview: string | null = $state(null);
  let avatarFile: File | null = $state(null);
  let profileSaving: boolean = $state(false);
  let profileMsg: string = $state('');
  let profileMsgType: 'success' | 'error' = $state('success');

  // ── Account tab state ──
  let email: string = $state('');
  let emailSaving: boolean = $state(false);
  let emailMsg: string = $state('');
  let emailMsgType: 'success' | 'error' = $state('success');

  let currentPassword: string = $state('');
  let newPassword: string = $state('');
  let confirmPassword: string = $state('');
  let passwordSaving: boolean = $state(false);
  let passwordMsg: string = $state('');
  let passwordMsgType: 'success' | 'error' = $state('success');

  // ── Initialize form fields when user data loads ──
  $effect(() => {
    if (currentUser) {
      displayName = currentUser.display_name ?? '';
      about = currentUser.about ?? '';
      email = currentUser.email ?? '';
      avatarPreview = currentUser.avatar_url ?? null;
    }
  });

  // Reset messages when switching tabs
  $effect(() => {
    if (activeTab) {
      profileMsg = '';
      emailMsg = '';
      passwordMsg = '';
    }
  });

  // ── Avatar handling ──
  let fileInput: HTMLInputElement = $state(null!);

  function onAvatarClick() {
    fileInput?.click();
  }

  function onAvatarSelected(e: Event) {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    if (!file.type.startsWith('image/')) {
      profileMsg = 'File must be an image';
      profileMsgType = 'error';
      return;
    }
    if (file.size > 5 * 1024 * 1024) {
      profileMsg = 'Avatar too large (max 5MB)';
      profileMsgType = 'error';
      return;
    }

    avatarFile = file;
    avatarPreview = URL.createObjectURL(file);
    profileMsg = '';
  }

  // ── Save profile (display name + about + avatar) ──
  async function saveProfile() {
    profileSaving = true;
    profileMsg = '';
    try {
      // Upload avatar first if changed
      if (avatarFile) {
        const updated = await api.uploadAvatar(avatarFile);
        auth.updateUser(updated);
        avatarFile = null;
      }

      // Update display name and about
      const updated = await api.updateMe({
        display_name: displayName.trim() || undefined,
        about: about.trim() || undefined,
      });
      auth.updateUser(updated);

      profileMsg = 'Profile saved';
      profileMsgType = 'success';
    } catch (e: any) {
      profileMsg = e.message || 'Failed to save profile';
      profileMsgType = 'error';
    }
    profileSaving = false;
  }

  // ── Save email ──
  async function saveEmail() {
    if (!email.trim() || !email.includes('@')) {
      emailMsg = 'Enter a valid email';
      emailMsgType = 'error';
      return;
    }
    emailSaving = true;
    emailMsg = '';
    try {
      const updated = await api.updateMe({ email: email.trim() });
      auth.updateUser(updated);
      emailMsg = 'Email updated';
      emailMsgType = 'success';
    } catch (e: any) {
      emailMsg = e.message || 'Failed to update email';
      emailMsgType = 'error';
    }
    emailSaving = false;
  }

  // ── Change password ──
  async function savePassword() {
    if (newPassword.length < 8) {
      passwordMsg = 'New password must be at least 8 characters';
      passwordMsgType = 'error';
      return;
    }
    if (newPassword !== confirmPassword) {
      passwordMsg = 'Passwords do not match';
      passwordMsgType = 'error';
      return;
    }
    if (!currentPassword) {
      passwordMsg = 'Enter your current password';
      passwordMsgType = 'error';
      return;
    }
    passwordSaving = true;
    passwordMsg = '';
    try {
      await api.changePassword(currentPassword, newPassword);
      currentPassword = '';
      newPassword = '';
      confirmPassword = '';
      passwordMsg = 'Password changed';
      passwordMsgType = 'success';
    } catch (e: any) {
      passwordMsg = e.message || 'Failed to change password';
      passwordMsgType = 'error';
    }
    passwordSaving = false;
  }

  function close() {
    ui.closeModal();
  }

  function handleOverlayClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('us-overlay')) {
      close();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
  }

  function getInitial(user: PublicUser): string {
    return (user.display_name ?? user.username).charAt(0).toUpperCase();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="us-overlay modal-overlay" onclick={handleOverlayClick} onkeydown={handleKeydown} role="dialog" tabindex="-1">
  <div class="us-panel">
    <!-- Sidebar nav -->
    <div class="us-nav">
      <h3>User Settings</h3>
      <button
        class="us-nav-item"
        class:active={activeTab === 'profile'}
        onclick={() => { activeTab = 'profile'; }}
      >Profile</button>
      <button
        class="us-nav-item"
        class:active={activeTab === 'account'}
        onclick={() => { activeTab = 'account'; }}
      >Account</button>
      <div class="us-nav-divider"></div>
      <button class="us-nav-item logout" onclick={() => { auth.logout(); close(); }}>Log Out</button>
    </div>

    <!-- Content area -->
    <div class="us-content">
      <button class="us-close" onclick={close} aria-label="Close">
        <svg width="18" height="18" viewBox="0 0 18 18" fill="currentColor">
          <path d="M14.53 4.53l-1.06-1.06L9 7.94 4.53 3.47 3.47 4.53 7.94 9l-4.47 4.47 1.06 1.06L9 10.06l4.47 4.47 1.06-1.06L10.06 9z"/>
        </svg>
      </button>

      {#if activeTab === 'profile'}
        <h2>Profile</h2>
        <p class="us-subtitle">Customize how others see you.</p>

        <div class="us-profile-section">
          <!-- Avatar -->
          <div class="us-avatar-area">
            <input
              type="file"
              accept="image/*"
              bind:this={fileInput}
              onchange={onAvatarSelected}
              style="display:none;"
            />
            <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
            <div class="us-avatar" onclick={onAvatarClick} role="button" tabindex="0">
              {#if avatarPreview}
                <img src={avatarPreview} alt="Avatar" />
              {:else if currentUser}
                <span class="us-avatar-initial">{getInitial(currentUser)}</span>
              {/if}
              <div class="us-avatar-overlay">
                <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
                  <path d="M4 5a2 2 0 00-2 2v6a2 2 0 002 2h12a2 2 0 002-2V7a2 2 0 00-2-2h-1.586a1 1 0 01-.707-.293l-1.414-1.414A1 1 0 0011.586 3H8.414a1 1 0 00-.707.293L6.293 4.707A1 1 0 015.586 5H4zm6 7a3 3 0 100-6 3 3 0 000 6z"/>
                </svg>
              </div>
            </div>
            <span class="us-avatar-hint">Click to change</span>
          </div>

          <!-- Fields -->
          <div class="us-fields">
            <div class="field">
              <label for="us-displayname">Display Name</label>
              <input
                id="us-displayname"
                type="text"
                bind:value={displayName}
                placeholder={currentUser?.username ?? 'Display name'}
                maxlength={100}
              />
            </div>
            <div class="field">
              <label for="us-about">About</label>
              <textarea
                id="us-about"
                bind:value={about}
                placeholder="Tell people about yourself"
                maxlength={512}
                rows={3}
              ></textarea>
            </div>
          </div>
        </div>

        {#if profileMsg}
          <div class="us-msg" class:error={profileMsgType === 'error'} class:success={profileMsgType === 'success'}>
            {profileMsg}
          </div>
        {/if}

        <div class="us-actions">
          <button class="btn-ghost" onclick={close}>Cancel</button>
          <button class="btn-primary" onclick={saveProfile} disabled={profileSaving}>
            {profileSaving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>

      {:else if activeTab === 'account'}
        <h2>Account</h2>
        <p class="us-subtitle">Manage your email and password.</p>

        <!-- Email section -->
        <div class="us-section">
          <h3>Email</h3>
          <div class="us-inline-field">
            <input
              type="email"
              bind:value={email}
              placeholder="your@email.com"
            />
            <button class="btn-primary" onclick={saveEmail} disabled={emailSaving}>
              {emailSaving ? 'Saving...' : 'Update'}
            </button>
          </div>
          {#if emailMsg}
            <div class="us-msg" class:error={emailMsgType === 'error'} class:success={emailMsgType === 'success'}>
              {emailMsg}
            </div>
          {/if}
        </div>

        <!-- Password section -->
        <div class="us-section">
          <h3>Change Password</h3>
          <div class="field">
            <label for="us-curpw">Current Password</label>
            <input
              id="us-curpw"
              type="password"
              bind:value={currentPassword}
              placeholder="Enter current password"
            />
          </div>
          <div class="field">
            <label for="us-newpw">New Password</label>
            <input
              id="us-newpw"
              type="password"
              bind:value={newPassword}
              placeholder="Min 8 characters"
            />
          </div>
          <div class="field">
            <label for="us-confirmpw">Confirm New Password</label>
            <input
              id="us-confirmpw"
              type="password"
              bind:value={confirmPassword}
              placeholder="Repeat new password"
            />
          </div>

          {#if passwordMsg}
            <div class="us-msg" class:error={passwordMsgType === 'error'} class:success={passwordMsgType === 'success'}>
              {passwordMsg}
            </div>
          {/if}

          <div class="us-actions">
            <button class="btn-primary" onclick={savePassword} disabled={passwordSaving}>
              {passwordSaving ? 'Changing...' : 'Change Password'}
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .us-panel {
    display: flex;
    width: 820px;
    max-width: 95vw;
    height: 600px;
    max-height: 85vh;
    background: var(--bg-secondary);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-high);
    animation: scale-in 150ms ease;
    overflow: hidden;
  }

  /* ── Left nav ── */

  .us-nav {
    width: 200px;
    background: var(--bg-tertiary);
    padding: 16px 8px;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .us-nav h3 {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-faint);
    padding: 4px 10px 8px;
    font-weight: 700;
  }

  .us-nav-item {
    background: none;
    color: var(--text-muted);
    text-align: left;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    font-size: 14px;
    font-weight: 500;
    transition: background var(--transition), color var(--transition);
  }

  .us-nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-normal);
  }

  .us-nav-item.active {
    background: var(--bg-active);
    color: var(--text-white);
  }

  .us-nav-item.logout {
    color: var(--text-danger);
  }
  .us-nav-item.logout:hover {
    background: rgba(242, 63, 67, 0.1);
    color: var(--text-danger);
  }

  .us-nav-divider {
    height: 1px;
    background: var(--border-subtle);
    margin: 8px 10px;
  }

  /* ── Right content ── */

  .us-content {
    flex: 1;
    padding: 24px 32px;
    overflow-y: auto;
    position: relative;
  }

  .us-content h2 {
    font-size: 20px;
    margin-bottom: 4px;
  }

  .us-subtitle {
    color: var(--text-muted);
    font-size: 13px;
    margin-bottom: 24px;
  }

  .us-close {
    position: absolute;
    top: 16px;
    right: 16px;
    background: none;
    color: var(--text-muted);
    padding: 6px;
    border-radius: var(--radius-sm);
    transition: color var(--transition), background var(--transition);
  }
  .us-close:hover {
    color: var(--text-normal);
    background: var(--bg-hover);
  }

  /* ── Profile tab ── */

  .us-profile-section {
    display: flex;
    gap: 24px;
    margin-bottom: 20px;
  }

  .us-avatar-area {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .us-avatar {
    width: 96px;
    height: 96px;
    border-radius: 50%;
    background: var(--bg-accent);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    position: relative;
    overflow: hidden;
    transition: opacity var(--transition);
  }

  .us-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .us-avatar-initial {
    font-size: 36px;
    font-weight: 700;
    color: var(--text-white);
    user-select: none;
  }

  .us-avatar-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity var(--transition);
    color: var(--text-white);
  }

  .us-avatar:hover .us-avatar-overlay {
    opacity: 1;
  }

  .us-avatar-hint {
    font-size: 11px;
    color: var(--text-faint);
  }

  .us-fields {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .us-fields .field label {
    display: block;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
    margin-bottom: 6px;
    letter-spacing: 0.02em;
  }

  .us-fields textarea {
    resize: vertical;
    min-height: 64px;
  }

  /* ── Account tab ── */

  .us-section {
    margin-bottom: 28px;
    padding-bottom: 28px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .us-section:last-child {
    border-bottom: none;
  }

  .us-section h3 {
    font-size: 16px;
    margin-bottom: 12px;
    color: var(--text-white);
  }

  .us-section .field {
    margin-bottom: 12px;
  }

  .us-section .field label {
    display: block;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
    margin-bottom: 6px;
    letter-spacing: 0.02em;
  }

  .us-inline-field {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .us-inline-field input {
    flex: 1;
  }

  /* ── Messages ── */

  .us-msg {
    font-size: 13px;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    margin-top: 8px;
  }

  .us-msg.success {
    color: var(--text-positive);
    background: rgba(35, 165, 89, 0.1);
  }

  .us-msg.error {
    color: var(--text-danger);
    background: rgba(242, 63, 67, 0.1);
  }

  /* ── Actions ── */

  .us-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
</style>
