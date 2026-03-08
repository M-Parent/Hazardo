<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';

  let ready = false;

  onMount(async () => {
    try {
      const exists = await invoke<boolean>('has_device');
      const path = location.pathname;
      if (!exists && !path.startsWith('/setup')) {
        await goto('/setup');
      } else if (exists && path.startsWith('/setup')) {
        await goto('/');
      }
    } catch (e) {
      console.warn('device check failed', e);
    }
    ready = true;
  });
</script>

{#if ready}
  <slot />
{:else}
  <div class="fixed inset-0 flex items-center justify-center bg-hazardo-background">
    <img src="/logo.png" alt="Hazardo" width="80" class="animate-pulse" />
  </div>
{/if}
