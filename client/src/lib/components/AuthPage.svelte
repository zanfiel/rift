<script lang="ts">
  import { auth } from '../stores/auth';

  let mode: 'login' | 'register' = $state('login');
  let username: string = $state('');
  let email: string = $state('');
  let password: string = $state('');
  let confirmPassword: string = $state('');
  let displayName: string = $state('');
  let submitting: boolean = $state(false);
  let errorMsg: string = $state('');

  async function handleSubmit(e: Event) {
    e.preventDefault();
    errorMsg = '';
    submitting = true;

    try {
      if (mode === 'login') {
        await auth.login(username, password);
      } else {
        if (password !== confirmPassword) {
          errorMsg = 'Passwords do not match';
          submitting = false;
          return;
        }
        if (password.length < 8) {
          errorMsg = 'Password must be at least 8 characters';
          submitting = false;
          return;
        }
        await auth.register(username, email, password, displayName || undefined);
      }
    } catch (e: any) {
      errorMsg = e.message || 'Something went wrong';
    }

    submitting = false;
  }

  function switchMode() {
    mode = mode === 'login' ? 'register' : 'login';
    errorMsg = '';
  }
</script>

<div class="auth-page">
  <div class="auth-card">
    <div class="brand">
      <h1>Zanverse</h1>
      <p class="tagline">
        {mode === 'login' ? 'Welcome back!' : 'Create an account'}
      </p>
    </div>

    <form onsubmit={handleSubmit}>
      <div class="field">
        <label for="username">Username</label>
        <input
          id="username"
          type="text"
          bind:value={username}
          placeholder="Enter your username"
          required
          minlength={3}
          maxlength={32}
          autocomplete={mode === 'login' ? 'username' : 'off'}
        />
      </div>

      {#if mode === 'register'}
        <div class="field">
          <label for="email">Email</label>
          <input
            id="email"
            type="email"
            bind:value={email}
            placeholder="Enter your email"
            required
            autocomplete="email"
          />
        </div>

        <div class="field">
          <label for="display-name">Display Name <span class="optional">(optional)</span></label>
          <input
            id="display-name"
            type="text"
            bind:value={displayName}
            placeholder="How others will see you"
            maxlength={64}
          />
        </div>
      {/if}

      <div class="field">
        <label for="password">Password</label>
        <input
          id="password"
          type="password"
          bind:value={password}
          placeholder={mode === 'login' ? 'Enter your password' : 'At least 8 characters'}
          required
          minlength={mode === 'register' ? 8 : undefined}
          autocomplete={mode === 'login' ? 'current-password' : 'new-password'}
        />
      </div>

      {#if mode === 'register'}
        <div class="field">
          <label for="confirm-password">Confirm Password</label>
          <input
            id="confirm-password"
            type="password"
            bind:value={confirmPassword}
            placeholder="Confirm your password"
            required
            autocomplete="new-password"
          />
        </div>
      {/if}

      {#if errorMsg}
        <div class="error">{errorMsg}</div>
      {/if}

      <button type="submit" class="btn-primary submit-btn" disabled={submitting}>
        {#if submitting}
          <span class="spinner" style="width: 16px; height: 16px; border-width: 2px;"></span>
        {:else}
          {mode === 'login' ? 'Log In' : 'Register'}
        {/if}
      </button>
    </form>

    <div class="switch-mode">
      {#if mode === 'login'}
        <span>Need an account?</span>
        <button class="btn-link" onclick={switchMode}>Register</button>
      {:else}
        <span>Already have an account?</span>
        <button class="btn-link" onclick={switchMode}>Log In</button>
      {/if}
    </div>
  </div>
</div>

<style>
  .auth-page {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
  }

  .auth-card {
    background: var(--bg-primary);
    border-radius: var(--radius-lg);
    padding: 32px;
    width: 100%;
    max-width: 420px;
    box-shadow: var(--shadow-high);
  }

  .brand {
    text-align: center;
    margin-bottom: 24px;
  }

  .brand h1 {
    font-size: 28px;
    font-weight: 700;
    color: var(--text-white);
    letter-spacing: -0.02em;
  }

  .tagline {
    color: var(--text-muted);
    margin-top: 6px;
  }

  .field {
    margin-bottom: 16px;
  }

  .field label {
    display: block;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
    margin-bottom: 6px;
    letter-spacing: 0.02em;
  }

  .optional {
    text-transform: none;
    font-weight: 400;
    color: var(--text-faint);
  }

  .error {
    color: var(--text-danger);
    font-size: 13px;
    margin-bottom: 12px;
  }

  .submit-btn {
    width: 100%;
    padding: 12px;
    font-size: 15px;
    font-weight: 600;
    margin-top: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }

  .switch-mode {
    text-align: center;
    margin-top: 16px;
    font-size: 13px;
    color: var(--text-muted);
  }
</style>
