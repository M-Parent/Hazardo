<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { handleMarkdownClick, renderMarkdown } from '$lib/markdown';
  import { selectedUser } from '../../stores/userStore';
  import { t, currentLang, formatDateLocalized } from '../../stores/i18nStore';
  import { MAX_IMAGE_SIZE } from '$lib/constants';
  import { showToast } from '../../stores/toastStore';
  import Title from '../../components/atoms/Title.svelte';
  import Icon from '../../components/atoms/Icon.svelte';
  import FormLabel from '../../components/atoms/FormLabel.svelte';
  import SearchBar from '../../components/molecules/SearchBar.svelte';
  import SelectDropdown from '../../components/molecules/SelectDropdown.svelte';
  import Modal from '../../components/organisms/Modal.svelte';
  import ScrollToTop from '../../components/atoms/ScrollToTop.svelte';
  import type { Category, Pick as PickType } from '$lib/types';

  // marked is configured globally by $lib/markdown

  let categories: Category[] = [];
  let picks: PickType[] = [];
  let selectedCategoryId = '';
  let searchQuery = '';
  let listContainer: HTMLElement;

  // Edit pick modal
  let showEditPick = false;
  let editingPick: PickType | null = null;
  let editPickNotes = '';
  let showImageOptions = false;
  let fileInput: HTMLInputElement;
  let cameraInput: HTMLInputElement;
  let showFullscreenImage = false;
  let showConfirmDeleteImage = false;
  let showAiRecommendation = false;



  $: if ($selectedUser) {
    loadCategories($selectedUser.user_id);
  }

  $: if ($selectedUser) {
    loadPicks($selectedUser.user_id, selectedCategoryId ? Number(selectedCategoryId) : null);
  }

  $: filteredPicks = searchQuery
    ? picks.filter(p => p.item_name.toLowerCase().includes(searchQuery.toLowerCase()) || p.category_name.toLowerCase().includes(searchQuery.toLowerCase()))
    : picks;

  $: categoryOptions = [
    { value: '', label: $t('picked.all_categories'), icon: '' },
    ...categories.map(c => ({ value: String(c.category_id), label: c.category_name, icon: c.category_icon }))
  ];

  async function loadCategories(userId: number) {
    try {
      categories = await invoke<Category[]>('get_categories', { userId });
    } catch (e) {
      console.error('get_categories failed', e);
    }
  }

  async function loadPicks(userId: number, categoryId: number | null) {
    try {
      picks = await invoke<PickType[]>('get_picks', { userId, categoryId });
    } catch (e) {
      console.error('get_picks failed', e);
    }
  }

  function openEditPick(pick: PickType) {
    editingPick = pick;
    editPickNotes = pick.notes || '';
    showAiRecommendation = false;
    showEditPick = true;
  }

  async function handleSavePick() {
    if (!editingPick || !$selectedUser) return;
    try {
      await invoke('update_pick', {
        pickId: editingPick.pick_id,
        notes: editPickNotes || null,
        imagePath: editingPick.image_path || null,
      });
      await loadPicks($selectedUser.user_id, selectedCategoryId ? Number(selectedCategoryId) : null);
      showEditPick = false;
    } catch (e) {
      console.error('update_pick failed', e);
    }
  }

  function handleImageFile(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file || !editingPick) return;
    if (file.size > MAX_IMAGE_SIZE) {
      showToast($t('picked.image_too_large') || 'Image must be under 5 MB', 'error');
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      if (editingPick) {
        editingPick.image_path = reader.result as string;
        editingPick = editingPick;
      }
      showImageOptions = false;
    };
    reader.readAsDataURL(file);
  }

  function removeImage() {
    showConfirmDeleteImage = false;
    if (editingPick) {
      editingPick.image_path = null;
      editingPick = editingPick;
    }
  }

  async function downloadImage() {
    if (!editingPick?.image_path) return;
    const filename = `hazardo_${editingPick.item_name.replace(/\s+/g, '_')}_${Date.now()}`;
    try {
      const result = await invoke<string>('save_to_gallery', {
        base64Data: editingPick.image_path,
        filename
      });
      if (result === 'ok') {
        showToast($t('picked.image_saved'), 'success');
      } else {
        showToast($t('picked.image_save_failed'), 'error');
      }
    } catch (e: any) {
      console.error('[Download] Error:', e);
      showToast(e?.message || $t('picked.image_save_failed'), 'error');
    }
  }

  async function handleDeletePick(pickId: number) {
    if (!$selectedUser) return;
    try {
      await invoke('delete_pick', { pickId });
      await loadPicks($selectedUser.user_id, selectedCategoryId ? Number(selectedCategoryId) : null);
    } catch (e) {
      console.error('delete_pick failed', e);
    }
  }

  function formatDate(dateStr: string): string {
    return formatDateLocalized(dateStr, $currentLang);
  }
</script>

<main class="max-w-lg mx-auto px-4 flex flex-col h-[calc(100vh-8rem)]">
  <div class="flex flex-col items-center">
    <div class="mt-6 mb-6 bg-hazardo-accent px-2 rounded">
      <Title title={$t('picked.title')} />
    </div>
  </div>

  <div class="mb-3">
    <FormLabel label={$t('picked.filter')} />
    <div class="mt-1">
      <SelectDropdown options={categoryOptions} bind:selected={selectedCategoryId} placeholder="" />
    </div>
  </div>

  <div class="mb-3">
    <SearchBar bind:value={searchQuery} />
  </div>

  <h3 class="title-font text-base mb-2">{$t('picked.list_title')}</h3>

  <div class="flex-1 overflow-y-auto" bind:this={listContainer}>
    {#if filteredPicks.length === 0}
      <p class="text-center text-hazardo-lightGray py-8 text-sm">{$t('picked.no_picks')}</p>
    {:else}
      {#each filteredPicks as pick}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div class="flex items-center justify-between py-3 border-b border-hazardo-lightGray/30 cursor-pointer hover:bg-hazardo-background/50 transition-colors" on:click={() => openEditPick(pick)}>
          <div class="flex items-center gap-2 text-sm min-w-0">
            <span class="text-hazardo-text font-medium whitespace-nowrap">{formatDate(pick.pick_date)}</span>
            <span class="text-hazardo-lightGray">-</span>
            <Icon name={pick.category_icon} size={16} />
            <span class="truncate">{pick.item_name}</span>
          </div>
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <!-- svelte-ignore a11y-no-static-element-interactions -->
          <div class="flex items-center gap-2 shrink-0 ml-2" on:click|stopPropagation>
            <button class="p-2 text-hazardo-lightGray hover:text-hazardo-primary rounded-lg hover:bg-hazardo-background transition-colors" on:click={() => openEditPick(pick)}>
              <Icon name="edit" size={18} />
            </button>
            <button class="p-2 text-hazardo-lightGray hover:text-red-500 rounded-lg hover:bg-red-50 transition-colors" on:click={() => handleDeletePick(pick.pick_id)}>
              <Icon name="trash" size={18} />
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <ScrollToTop container={listContainer} />
</main>

<!-- Edit Pick Modal -->
<Modal bind:show={showEditPick} title={$t('picked.title')} width="w-96">
  {#if editingPick}
    <div class="flex flex-col gap-4">
      <div class="flex items-center justify-between">
        <p class="text-sm font-semibold text-hazardo-text">Date: {formatDate(editingPick.pick_date)}</p>
        <button class="p-1 text-hazardo-lightGray hover:text-red-500" on:click={() => { if (editingPick) { handleDeletePick(editingPick.pick_id); showEditPick = false; } }}>
          <Icon name="trash" size={18} />
        </button>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div class="border border-hazardo-lightGray rounded-lg p-3">
          <div class="flex items-center gap-1 text-xs text-hazardo-text mb-1">
            <Icon name="clock" size={14} /> {$t('home.time')}
          </div>
          <p class="font-semibold text-center">{editingPick.time_pref}</p>
        </div>
        <div class="border border-hazardo-lightGray rounded-lg p-3">
          <div class="flex items-center gap-1 text-xs text-hazardo-text mb-1">
            <Icon name="users" size={14} /> {$t('home.vibe')}
          </div>
          <p class="font-semibold text-center">{editingPick.vibe_pref}</p>
        </div>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div class="border border-hazardo-lightGray rounded-lg p-3">
          <div class="flex items-center gap-1 text-xs text-hazardo-text mb-1">
            <Icon name={editingPick.category_icon} size={14} /> {editingPick.category_name}
          </div>
          <p class="font-semibold text-center">{editingPick.item_name}</p>
        </div>
        <div class="border border-hazardo-lightGray rounded-lg p-3">
          <div class="flex items-center gap-1 text-xs text-hazardo-text mb-1">
            <Icon name="thermometer" size={14} /> {$t('home.location')}
          </div>
          <p class="font-semibold text-center text-sm">{editingPick.location || 'N/A'}</p>
        </div>
      </div>

      <!-- AI Recommendation -->
      {#if editingPick.ai_recommendation}
        <div class="border border-hazardo-lightGray rounded-lg p-3">
          <button class="flex items-center gap-1 text-xs text-hazardo-text w-full" on:click={() => showAiRecommendation = !showAiRecommendation}>
            <Icon name="chatbot" size={14} /> {$t('home.ai_recommendation')}
            <span class="ml-auto text-[10px] text-hazardo-lightGray">{showAiRecommendation ? '▲' : '▼'}</span>
          </button>
          {#if showAiRecommendation}
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div class="text-sm mt-2 max-h-48 overflow-y-auto markdown-content" on:click={handleMarkdownClick}>
              {@html renderMarkdown(editingPick.ai_recommendation)}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Notes -->
      <div class="border border-hazardo-lightGray rounded-lg p-3">
        <div class="flex items-center gap-1 text-xs text-hazardo-text mb-2">
          <Icon name="edit" size={14} /> {$t('picked.notes')}
        </div>
        <textarea
          bind:value={editPickNotes}
          placeholder={$t('picked.add_notes')}
          class="w-full text-sm resize-none focus:outline-none bg-transparent"
          rows="3"
        ></textarea>
      </div>

      <!-- Image -->
      <div class="border border-hazardo-lightGray rounded-lg p-3">
        <div class="flex items-center gap-1 text-xs text-hazardo-text mb-2">
          <Icon name="image" size={14} /> {$t('picked.image')}
        </div>
        {#if editingPick.image_path}
          <div class="relative">
            <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <img src={editingPick.image_path} alt="Event" class="w-full h-32 object-cover rounded cursor-pointer" on:click={() => showFullscreenImage = true} />
            <button class="absolute top-1 left-1 w-6 h-6 rounded-full bg-hazardo-accent text-white flex items-center justify-center hover:bg-hazardo-primary transition-colors" title={$t('picked.download_image')} on:click={downloadImage}>
              <Icon name="download" size={14} />
            </button>
            <button class="absolute top-1 right-1 w-6 h-6 rounded-full bg-red-500 text-white flex items-center justify-center text-xs hover:bg-red-600 transition-colors" on:click={() => showConfirmDeleteImage = true}>✕</button>
          </div>
        {:else if showImageOptions}
          <div class="flex gap-2">
            <button class="flex-1 flex flex-col items-center gap-1 py-3 rounded-lg bg-hazardo-accent/10 text-hazardo-accent hover:bg-hazardo-accent/20 transition-colors text-xs font-medium" on:click={() => cameraInput?.click()}>
              <Icon name="camera" size={20} />
              {$t('picked.take_photo')}
            </button>
            <button class="flex-1 flex flex-col items-center gap-1 py-3 rounded-lg bg-hazardo-accent/10 text-hazardo-accent hover:bg-hazardo-accent/20 transition-colors text-xs font-medium" on:click={() => fileInput?.click()}>
              <Icon name="upload" size={20} />
              {$t('picked.upload')}
            </button>
            <button class="px-2 py-3 rounded-lg text-hazardo-lightGray hover:text-hazardo-text transition-colors text-xs" on:click={() => showImageOptions = false}>
              {$t('picked.cancel')}
            </button>
          </div>
        {:else}
          <button class="w-10 h-10 rounded-lg bg-hazardo-accent/10 text-hazardo-accent flex items-center justify-center hover:bg-hazardo-accent/20 transition-colors" on:click={() => showImageOptions = true}>
            <Icon name="plus" size={20} />
          </button>
        {/if}
        <input type="file" accept="image/*" capture="environment" class="hidden" bind:this={cameraInput} on:change={handleImageFile} />
        <input type="file" accept="image/*" class="hidden" bind:this={fileInput} on:change={handleImageFile} />
      </div>

      <div class="flex items-center justify-center gap-3 mt-2">
        <button class="px-4 py-2 text-sm text-hazardo-text hover:text-hazardo-primary" on:click={() => showEditPick = false}>{$t('picked.cancel')}</button>
        <button class="px-6 py-2 rounded bg-hazardo-primary text-white text-sm font-medium" on:click={handleSavePick}>{$t('picked.save')}</button>
      </div>
    </div>
  {/if}
</Modal>

<!-- Fullscreen Image Overlay -->
{#if showFullscreenImage && editingPick?.image_path}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="fixed inset-0 z-70 bg-black flex items-center justify-center" on:click={() => showFullscreenImage = false}>
    <button class="absolute top-4 right-4 w-10 h-10 rounded-full bg-white/20 text-white flex items-center justify-center text-xl hover:bg-white/30 transition-colors z-71" on:click={() => showFullscreenImage = false}>✕</button>
    <button class="absolute top-4 left-4 w-10 h-10 rounded-full bg-white/20 text-white flex items-center justify-center hover:bg-white/30 transition-colors z-71" on:click|stopPropagation={downloadImage} title={$t('picked.download_image')}>
      <Icon name="download" size={20} />
    </button>
    <img src={editingPick.image_path} alt="Event fullscreen" class="max-w-full max-h-full object-contain" />
  </div>
{/if}

<!-- Confirm Delete Image -->
{#if showConfirmDeleteImage}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="fixed inset-0 z-70 bg-black/50 flex items-center justify-center" on:click={() => showConfirmDeleteImage = false}>
    <div class="bg-hazardo-surface rounded-lg p-6 w-72 shadow-xl" on:click|stopPropagation>
      <p class="text-sm font-semibold text-hazardo-text mb-2">{$t('picked.delete_image_title')}</p>
      <p class="text-xs text-hazardo-lightGray mb-4">{$t('picked.delete_image_msg')}</p>
      <div class="flex justify-end gap-2">
        <button class="px-4 py-1 rounded border border-hazardo-lightGray text-sm" on:click={() => showConfirmDeleteImage = false}>{$t('picked.cancel')}</button>
        <button class="px-4 py-1 rounded bg-red-500 text-white text-sm" on:click={removeImage}>{$t('picked.delete')}</button>
      </div>
    </div>
  </div>
{/if}
