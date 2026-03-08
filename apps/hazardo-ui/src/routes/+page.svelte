<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import LogoTxt from '../components/atoms/LogoTxt.svelte';
  import Title from '../components/atoms/Title.svelte';
  import FormLabel from '../components/atoms/FormLabel.svelte';
  import FormInput from '../components/atoms/FormInput.svelte';
  import Icon from '../components/atoms/Icon.svelte';

  interface User { user_id: number; user_name: string; }

  const timeOptions = ['AM', 'PM', 'Night', 'Mixed'];
  const vibeOptions = ['Friend', 'Date', 'Family', 'Mixed'];

  let selectedTime = 'Mixed';
  let selectedVibe = 'Mixed';
  let selectedList = 'All Lists';
  let selectedDate = new Date().toISOString().split('T')[0];
  let advanceRoll = false;

  let users: User[] = [];
  let selectedUser: User | null = null;
  let dropdownOpen = false;

  let showModal = false;
  let newUserName = '';

  onMount(async () => {
    await loadUsers();
  });

  async function loadUsers() {
    try {
      users = await invoke<User[]>('get_users');
      if (users.length > 0 && !selectedUser) {
        selectedUser = users[0];
      }
    } catch (e) {
      console.error('get_users failed', e);
    }
  }

  function selectUser(u: User) {
    selectedUser = u;
    dropdownOpen = false;
  }

  function openCreateModal() {
    dropdownOpen = false;
    newUserName = '';
    showModal = true;
  }

  async function createUser() {
    if (!newUserName.trim()) return;
    try {
      const created = await invoke<User>('create_user', { userName: newUserName.trim() });
      users = [...users, created];
      selectedUser = created;
      showModal = false;
    } catch (e) {
      console.error('create_user failed', e);
      alert('Failed to create user: ' + JSON.stringify(e));
    }
  }
</script>

<main class="max-w-lg pb-24">
  <div class="flex items-center justify-between">
    <LogoTxt />
    <!-- User dropdown -->
    <div class="relative mt-8 me-3">
      <button
        class="border rounded px-3 py-1 text-sm border-hazardo-lightGray flex items-center gap-1"
        on:click={() => dropdownOpen = !dropdownOpen}
      >
        {selectedUser?.user_name ?? 'Username'} <span>▾</span>
      </button>
      {#if dropdownOpen}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div class="fixed inset-0 z-10" on:click={() => dropdownOpen = false}></div>
        <div class="absolute right-0 mt-1 w-44 bg-white border border-hazardo-lightGray rounded shadow-lg z-20">
          {#each users as u}
            <button
              class="w-full text-left px-3 py-2 text-sm hover:bg-hazardo-background {selectedUser?.user_id === u.user_id ? 'text-hazardo-primary font-semibold' : ''}"
              on:click={() => selectUser(u)}
            >{u.user_name}</button>
          {/each}
          <hr class="border-hazardo-lightGray" />
          <button
            class="w-full text-left px-3 py-2 text-sm text-hazardo-accent hover:bg-hazardo-background"
            on:click={openCreateModal}
          >+ Create a user</button>
        </div>
      {/if}
    </div>
  </div>

  <div class="flex flex-col items-center">
    <div class="mt-14 mb-10 bg-hazardo-accent px-2 rounded">
      <Title title="Select Criteria" />
    </div>

    <div class="w-full max-w-78 flex flex-col gap-6">
      <!-- Time Preference -->
      <div>
        <FormLabel label="Time Preference:" />
        <div class="flex gap-0 mt-2 border rounded border-hazardo-lightGray overflow-hidden">
          {#each timeOptions as opt}
            <button
              class="flex-1 py-2 text-sm transition-colors {selectedTime === opt ? 'bg-hazardo-accent text-white' : ''}"
              on:click={() => selectedTime = opt}
            >{opt}</button>
          {/each}
        </div>
      </div>

      <!-- Vibe Preference -->
      <div>
        <FormLabel label="Vibe Preference:" />
        <div class="flex gap-0 mt-2 border rounded border-hazardo-lightGray overflow-hidden">
          {#each vibeOptions as opt}
            <button
              class="flex-1 py-2 text-sm transition-colors {selectedVibe === opt ? 'bg-hazardo-accent text-white' : ''}"
              on:click={() => selectedVibe = opt}
            >{opt}</button>
          {/each}
        </div>
      </div>

      <!-- List Preference -->
      <div>
        <FormLabel label="List Preference:" />
        <div class="relative mt-2">
          <select
            bind:value={selectedList}
            class="w-full appearance-none border rounded p-2 pr-8 border-hazardo-lightGray focus:outline-hazardo-accent bg-white"
          >
            <option>All Lists</option>
          </select>
          <span class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-hazardo-lightGray">▾</span>
        </div>
      </div>

      <!-- Date Preference -->
      <div>
        <FormLabel label="Date Preference:" />
        <div class="relative mt-2">
          <input
            type="date"
            bind:value={selectedDate}
            class="w-full border rounded p-2 pr-8 border-hazardo-lightGray focus:outline-hazardo-accent bg-white"
          />
        </div>
      </div>

      <!-- Advance Roll -->
      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={advanceRoll} class="w-4 h-4 accent-hazardo-accent" />
        Advance Roll (need AI setting to work)
      </label>

      <!-- Roll Dice -->
      <div class="flex justify-center mt-4">
        <button class="bg-hazardo-primary text-white title-font shadow-purple-700 py-2 px-8 rounded">
          Roll Dice
        </button>
      </div>
    </div>
  </div>
</main>

<!-- Bottom Nav -->
<nav class="fixed bottom-0 left-0 right-0 bg-white border-t border-hazardo-lightGray flex justify-around py-2">
  <a href="/" class="flex flex-col items-center text-xs text-hazardo-primary gap-1">
    <Icon name="home" size={22} /> Home
  </a>
  <a href="/" class="flex flex-col items-center text-xs text-hazardo-text gap-1">
    <Icon name="vault" size={22} /> Vault
  </a>
  <a href="/" class="flex flex-col items-center text-xs text-hazardo-text gap-1">
    <Icon name="picked" size={22} /> Picked
  </a>
  <a href="/" class="flex flex-col items-center text-xs text-hazardo-text gap-1">
    <Icon name="chatbot" size={22} /> ChatBot
  </a>
  <a href="/" class="flex flex-col items-center text-xs text-hazardo-text gap-1">
    <Icon name="setting" size={22} /> Setting
  </a>
</nav>

<!-- Create User Modal -->
{#if showModal}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40" on:click={() => showModal = false}>
    <div class="bg-white rounded-lg p-6 w-80 shadow-xl" on:click|stopPropagation>
      <h2 class="title-font text-lg mb-4">Create User</h2>
      <form on:submit|preventDefault={createUser} class="flex flex-col gap-4">
        <div class="flex flex-col">
          <FormLabel label="Username:" htmlFor="newUserName" />
          <FormInput id="newUserName" name="newUserName" bind:value={newUserName} placeholder="Enter Username..." />
        </div>
        <div class="flex justify-end gap-2">
          <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm" on:click={() => showModal = false}>Cancel</button>
          <button type="submit" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm">Create</button>
        </div>
      </form>
    </div>
  </div>
{/if}

