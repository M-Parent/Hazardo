<script lang="ts">
  import { users, selectedUser, createNewUser, loadUsers } from '../../stores/userStore';
  import Icon from '../atoms/Icon.svelte';
  import Modal from '../organisms/Modal.svelte';
  import FormLabel from '../atoms/FormLabel.svelte';
  import FormInput from '../atoms/FormInput.svelte';

  let dropdownOpen = false;
  let showCreateModal = false;
  let newUserName = '';

  function selectUser(u: { user_id: number; user_name: string }) {
    selectedUser.set(u);
    dropdownOpen = false;
  }

  function openCreateModal() {
    dropdownOpen = false;
    newUserName = '';
    showCreateModal = true;
  }

  async function handleCreate() {
    if (!newUserName.trim()) return;
    const created = await createNewUser(newUserName.trim());
    if (created) {
      showCreateModal = false;
    }
  }
</script>

<div class="relative">
  <button
    class="border rounded px-3 py-1 text-sm font-semibold border-hazardo-lightGray flex items-center gap-1 bg-hazardo-surface"
    on:click={() => dropdownOpen = !dropdownOpen}
  >
    {$selectedUser?.user_name ?? 'Username'}
    <Icon name="chevron-down" size={16} />
  </button>

  {#if dropdownOpen}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="fixed inset-0 z-10" on:click={() => dropdownOpen = false}></div>
    <div class="absolute right-0 mt-1 w-44 bg-hazardo-surface border border-hazardo-lightGray rounded shadow-lg z-20">
      {#each $users as u}
        <button
          class="w-full text-left px-3 py-2 text-sm hover:bg-hazardo-background {$selectedUser?.user_id === u.user_id ? 'text-hazardo-primary font-semibold' : ''}"
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

<Modal bind:show={showCreateModal} title="Create User">
  <form on:submit|preventDefault={handleCreate} class="flex flex-col gap-4">
    <div class="flex flex-col">
      <FormLabel label="Username:" htmlFor="newUserName" />
      <FormInput id="newUserName" name="newUserName" bind:value={newUserName} placeholder="Enter Username..." />
    </div>
    <div class="flex justify-end gap-2">
      <button type="button" class="px-4 py-1 rounded border border-hazardo-lightGray text-sm" on:click={() => showCreateModal = false}>Cancel</button>
      <button type="submit" class="px-4 py-1 rounded bg-hazardo-primary text-white text-sm">Create</button>
    </div>
  </form>
</Modal>
