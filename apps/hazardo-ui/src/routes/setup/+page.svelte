<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import LogoTxt from '../../components/atoms/LogoTxt.svelte';
  import PrimaryButton from '../../components/atoms/PrimaryButton.svelte';
  import Title from '../../components/atoms/Title.svelte';
  import FormLabel from '../../components/atoms/FormLabel.svelte';
  import FormInput from '../../components/atoms/FormInput.svelte';

  import { invoke } from '@tauri-apps/api/core';
  import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
  import { loadUsers } from '../../stores/userStore';
  import { t } from '../../stores/i18nStore';

  let deviceName = '';
  let username = '';
  let locationEnabled = false;
  let locationStatus: 'idle' | 'requesting' | 'granted' | 'denied' = 'idle';

  onMount(async () => {
    try {
      const exists = await invoke<boolean>('has_device');
      if (exists) {
        goto('/');
      }
    } catch (err) {
      console.error('device check failed', err);
    }
  });

  function requestLocationPermission() {
    if (!navigator.geolocation) {
      ipFallbackSetup();
      return;
    }
    locationStatus = 'requesting';
    navigator.geolocation.getCurrentPosition(
      () => { locationStatus = 'granted'; locationEnabled = true; },
      () => { ipFallbackSetup(); },
      { enableHighAccuracy: true, timeout: 15000 }
    );
  }

  async function ipFallbackSetup() {
    try {
      const resp = await tauriFetch('https://ipwho.is/', { method: 'GET' });
      if (resp.ok) {
        const data = await resp.json();
        if (data.success !== false && data.latitude && data.longitude) {
          locationStatus = 'granted';
          locationEnabled = true;
          return;
        }
      }
    } catch (_) {}
    locationStatus = 'denied';
    locationEnabled = false;
  }

  async function save() {
    if (!deviceName || deviceName.trim() === '') {
      alert($t('setup.alert_device_name'));
      return;
    }

    try {
      await invoke('create_device', {
        deviceName: deviceName.trim(),
        userName: username && username.trim() !== '' ? username.trim() : null,
      });
      await loadUsers();
      // Save location preference for the newly created user
      const users = await invoke<{user_id: number}[]>('get_users');
      if (users.length > 0 && locationEnabled) {
        await invoke('set_setting', { userId: users[0].user_id, key: 'location_enabled', value: 'true' });
      }
      goto('/');
    } catch (err) {
      console.error('create_device failed', err);
      alert('Failed to save device: ' + JSON.stringify(err));
    }
  }
</script>

<main class="max-w-lg mx-auto px-4">
  <LogoTxt />
  <div class="flex flex-col items-center">
    <div class="mt-14 mb-10 bg-hazardo-accent px-2 rounded">
      <Title title={$t('setup.welcome')} />
    </div>
    <p class="text-lg mx-auto max-w-78 mb-6">{$t('setup.device_desc')}</p>
    <div>
      <form on:submit|preventDefault={save} class="max-w-78">
        <div class="flex flex-col mb-6">
          <FormLabel label={$t('setup.device_name')} htmlFor="deviceName" />
          <FormInput id="deviceName" name="deviceName" bind:value={deviceName} placeholder={$t('setup.device_name_ph')} />
        </div>
        <p class="text-lg mx-auto max-w-78 mb-6">{$t('setup.user_desc')}</p>
        <div class="flex flex-col mb-10">
          <FormLabel label={$t('setup.username')} htmlFor="username" />
          <FormInput id="username" name="username" bind:value={username} placeholder={$t('setup.username_ph')} />
        </div>
        <div class="flex flex-col mb-10">
          <p class="text-sm text-hazardo-lightGray mb-3">{$t('setup.location_desc')}</p>
          {#if locationStatus === 'idle'}
            <button type="button" class="flex items-center justify-center gap-2 px-4 py-3 rounded-lg bg-hazardo-accent text-white text-sm font-medium" on:click={requestLocationPermission}>
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/><circle cx="12" cy="10" r="3"/></svg>
              {$t('location.allow')}
            </button>
          {:else if locationStatus === 'requesting'}
            <div class="flex items-center gap-2 text-sm text-hazardo-lightGray">
              <div class="w-4 h-4 border-2 border-hazardo-accent border-t-transparent rounded-full animate-spin"></div>
              {$t('location.requesting')}
            </div>
          {:else if locationStatus === 'granted'}
            <div class="flex items-center gap-2 text-sm text-green-600 font-medium">
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
              {$t('setup.location_granted')}
            </div>
          {:else if locationStatus === 'denied'}
            <div class="flex flex-col gap-2">
              <div class="flex items-center gap-2 text-sm text-red-500 font-medium">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/></svg>
                {$t('setup.location_denied')}
              </div>
              <button type="button" class="text-xs text-hazardo-accent underline text-left" on:click={requestLocationPermission}>{$t('setup.location_try_again')}</button>
            </div>
          {/if}
        </div>
        <div class="flex justify-center">
          <PrimaryButton />
        </div>
      </form>
    </div>
  </div>
</main>

