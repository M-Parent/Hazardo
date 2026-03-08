<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import LogoTxt from '../../components/atoms/LogoTxt.svelte';
  import PrimaryButton from '../../components/atoms/PrimaryButton.svelte';
  import Title from '../../components/atoms/Title.svelte';
  import FormLabel from '../../components/atoms/FormLabel.svelte';
  import FormInput from '../../components/atoms/FormInput.svelte';

  import { invoke } from '@tauri-apps/api/core';

  let deviceName = '';
  let username = '';

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

  async function save() {
    if (!deviceName || deviceName.trim() === '') {
      alert('Please enter a device name');
      return;
    }

    try {
      await invoke('create_device', {
        deviceName: deviceName.trim(),
        userName: username && username.trim() !== '' ? username.trim() : null,
      });
      goto('/');
    } catch (err) {
      console.error('create_device failed', err);
      alert('Failed to save device: ' + JSON.stringify(err));
    }
  }
</script>

<main class="max-w-lg">
  <LogoTxt />
  <div class="flex flex-col items-center">
    <div class="mt-14 mb-10 bg-hazardo-accent px-2 rounded">
      <Title title="Welcome to Hazardo!" />
    </div>
    <p class="text-lg mx-auto max-w-78 mb-6">Identify this device to enable seamless syncing across all your devices.</p>
    <div>
      <form on:submit|preventDefault={save} class="min-w-78">
        <div class="flex flex-col mb-6">
          <FormLabel label="Device Name:" htmlFor="deviceName" />
          <FormInput id="deviceName" name="deviceName" bind:value={deviceName} placeholder="Enter Device name..." />
        </div>
        <p class="text-lg mx-auto max-w-78 mb-6">Create a username to get started. You can host multiple users on this device, each with their own unique categories and lists.</p>
        <div class="flex flex-col mb-10">
          <FormLabel label="Username:" htmlFor="username" />
          <FormInput id="username" name="username" bind:value={username} placeholder="Enter Username..." />
        </div>
        <div class="flex justify-center">
          <PrimaryButton />
        </div>
      </form>
    </div>
  </div>
</main>

