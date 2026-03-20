<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { invoke } from '@tauri-apps/api/core';
  import { loadUsers, selectedUser } from '../stores/userStore';
  import { loadThemeSetting } from '../stores/themeStore';
  import { loadLanguageSetting } from '../stores/i18nStore';
  import AppHeader from '../components/organisms/AppHeader.svelte';
  import BottomNav from '../components/organisms/BottomNav.svelte';
  import Toast from '../components/atoms/Toast.svelte';

  let ready = false;

  $: isSetup = $page.url.pathname.startsWith('/setup');

  // Load theme and language when user changes
  $: if ($selectedUser) {
    loadThemeSetting($selectedUser.user_id);
    loadLanguageSetting($selectedUser.user_id);
  }

  onMount(async () => {
    try {
      const exists = await invoke<boolean>('has_device');
      const path = location.pathname;
      if (!exists && !path.startsWith('/setup')) {
        await goto('/setup');
      } else if (exists && path.startsWith('/setup')) {
        await goto('/');
      }
      if (exists) {
        await loadUsers();
      }
    } catch (e) {
      console.warn('device check failed', e);
    }
    ready = true;
  });
</script>

{#if ready}
  <Toast />
  {#if !isSetup}
    <div class="min-h-screen flex flex-col pb-16">
      <AppHeader />
      <div class="flex-1">
        <slot />
      </div>
      <BottomNav />
    </div>
  {:else}
    <slot />
  {/if}
{:else}
  <div class="fixed inset-0 flex items-center justify-center bg-hazardo-background">
    <img src="/logo.png" alt="Hazardo" width="80" class="animate-pulse" />
  </div>
{/if}
