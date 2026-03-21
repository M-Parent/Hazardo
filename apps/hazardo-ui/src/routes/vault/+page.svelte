<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { selectedUser } from '../../stores/userStore';
  import { t } from '../../stores/i18nStore';
  import Title from '../../components/atoms/Title.svelte';
  import Icon from '../../components/atoms/Icon.svelte';
  import FormLabel from '../../components/atoms/FormLabel.svelte';
  import FormInput from '../../components/atoms/FormInput.svelte';
  import SearchBar from '../../components/molecules/SearchBar.svelte';
  import SelectDropdown from '../../components/molecules/SelectDropdown.svelte';
  import OptionToggle from '../../components/molecules/OptionToggle.svelte';
  import Modal from '../../components/organisms/Modal.svelte';
  import ScrollToTop from '../../components/atoms/ScrollToTop.svelte';
  import type { Category, Item } from '$lib/types';
  import { DEFAULT_CATEGORIES } from '$lib/defaultCategories';
  import { TIME_OPTIONS, VIBE_OPTIONS } from '$lib/constants';
  import { showToast } from '../../stores/toastStore';

  let categories: Category[] = [];
  let items: Item[] = [];
  let selectedCategoryId = '';
  let searchQuery = '';
  let listContainer: HTMLElement;

  // Add/Edit Category modal
  let showCategoryModal = false;
  let editingCategory: Category | null = null;
  let catName = '';
  let catIcon = 'misc';
  const iconOptions = DEFAULT_CATEGORIES.map(c => ({ value: c.icon, label: c.name, icon: c.icon }));

  // Add/Edit Item modal
  let showItemModal = false;
  let editingItem: Item | null = null;
  let itemName = '';
  let itemCategoryId = '';
  let itemTimePref = 'Mixed';
  let itemVibePref = 'Mixed';
  let itemNotes = '';
  let itemIsPicked = 0;

  const timeOptions = [...TIME_OPTIONS];
  const vibeOptions = [...VIBE_OPTIONS];

  $: if ($selectedUser) {
    loadCategories($selectedUser.user_id);
  }

  $: if ($selectedUser) {
    loadItems($selectedUser.user_id, selectedCategoryId ? Number(selectedCategoryId) : null);
  }

  $: filteredItems = searchQuery
    ? items.filter(i => i.item_name.toLowerCase().includes(searchQuery.toLowerCase()))
    : items;

  $: categoryOptions = [
    { value: '', label: $t('vault.all_lists'), icon: '' },
    ...categories.map(c => ({ value: String(c.category_id), label: c.category_name, icon: c.category_icon }))
  ];

  $: itemCategoryOptions = categories.map(c => ({ value: String(c.category_id), label: c.category_name, icon: c.category_icon }));

  async function loadCategories(userId: number) {
    try {
      categories = await invoke<Category[]>('get_categories', { userId });
    } catch (e) {
      console.error('get_categories failed', e);
    }
  }

  async function loadItems(userId: number, categoryId: number | null) {
    try {
      items = await invoke<Item[]>('get_items', { userId, categoryId });
    } catch (e) {
      console.error('get_items failed', e);
    }
  }

  // Category modal
  function openAddCategory() {
    editingCategory = null;
    catName = '';
    catIcon = 'misc';
    showCategoryModal = true;
  }

  function openEditCategory() {
    const cat = categories.find(c => String(c.category_id) === selectedCategoryId);
    if (!cat) return;
    editingCategory = cat;
    catName = cat.category_name;
    catIcon = cat.category_icon;
    showCategoryModal = true;
  }

  async function handleSaveCategory(keepOpen: boolean) {
    if (!catName.trim() || !$selectedUser) return;
    try {
      if (editingCategory) {
        await invoke('update_category', {
          categoryId: editingCategory.category_id,
          categoryName: catName.trim(),
          categoryIcon: catIcon,
        });
      } else {
        await invoke('create_category', {
          userId: $selectedUser.user_id,
          categoryName: catName.trim(),
          categoryIcon: catIcon,
        });
      }
      await loadCategories($selectedUser.user_id);
      if (keepOpen && !editingCategory) {
        catName = '';
        catIcon = 'misc';
      } else {
        showCategoryModal = false;
      }
    } catch (e) {
      console.error('save category failed', e);
    }
  }

  // Item modal
  function openAddItem() {
    editingItem = null;
    itemName = '';
    itemCategoryId = selectedCategoryId || (categories.length > 0 ? String(categories[0].category_id) : '');
    itemTimePref = 'Mixed';
    itemVibePref = 'Mixed';
    itemNotes = '';
    itemIsPicked = 0;
    showItemModal = true;
  }

  function openEditItem(item: Item) {
    editingItem = item;
    itemName = item.item_name;
    itemCategoryId = String(item.category_id);
    itemTimePref = item.time_pref;
    itemVibePref = item.vibe_pref;
    itemNotes = item.notes || '';
    itemIsPicked = item.is_picked;
    showItemModal = true;
  }

  async function handleSaveItem(keepOpen: boolean) {
    if (!itemName.trim() || !itemCategoryId || !$selectedUser) return;
    try {
      if (editingItem) {
        await invoke('update_item', {
          itemId: editingItem.item_id,
          itemName: itemName.trim(),
          categoryId: Number(itemCategoryId),
          timePref: itemTimePref,
          vibePref: itemVibePref,
          location: null,
          description: null,
          notes: itemNotes || null,
          isPicked: itemIsPicked,
        });
      } else {
        await invoke('create_item', {
          userId: $selectedUser.user_id,
          categoryId: Number(itemCategoryId),
          itemName: itemName.trim(),
          timePref: itemTimePref,
          vibePref: itemVibePref,
          location: null,
          description: null,
          notes: itemNotes || null,
        });
      }
      await loadItems($selectedUser.user_id, selectedCategoryId ? Number(selectedCategoryId) : null);
      if (keepOpen && !editingItem) {
        itemName = '';
        itemNotes = '';
      } else {
        showItemModal = false;
      }
    } catch (e) {
      console.error('save item failed', e);
    }
  }

  async function handleDeleteItem(itemId: number) {
    if (!$selectedUser) return;
    try {
      await invoke('delete_item', { itemId });
      await loadItems($selectedUser.user_id, selectedCategoryId ? Number(selectedCategoryId) : null);
      showToast($t('vault.item_deleted'), 'success');
    } catch (e) {
      console.error('delete_item failed', e);
    }
  }

  async function handleDeleteCategory() {
    if (!editingCategory || !$selectedUser) return;
    try {
      await invoke('delete_category', { categoryId: editingCategory.category_id });
      showCategoryModal = false;
      selectedCategoryId = '';
      await loadCategories($selectedUser.user_id);
      await loadItems($selectedUser.user_id, null);
      showToast($t('vault.category_deleted'), 'success');
    } catch (e) {
      console.error('delete_category failed', e);
    }
  }

  function getCategoryIcon(catId: number): string {
    return categories.find(c => c.category_id === catId)?.category_icon || 'misc';
  }
</script>

<main class="max-w-lg mx-auto px-4 flex flex-col h-[calc(100vh-8rem)]">
  <!-- Static header section -->
  <div class="flex flex-col items-center">
    <div class="mt-6 mb-6 bg-hazardo-accent px-2 rounded">
      <Title title="{$t('vault.title_of')} {$selectedUser?.user_name ?? 'User'}" />
    </div>
  </div>

  <div class="mb-3">
    <FormLabel label={$t('vault.select_categories')} />
    <div class="mt-1 flex gap-2">
      <div class="flex-1">
        <SelectDropdown options={categoryOptions} bind:selected={selectedCategoryId} placeholder="" />
      </div>
      {#if selectedCategoryId}
        <button
          class="bg-hazardo-surface border border-hazardo-lightGray text-hazardo-primary rounded p-2 hover:bg-hazardo-background transition-colors"
          on:click={openEditCategory}
          title={$t('vault.edit_category')}
        >
          <Icon name="edit" size={18} />
        </button>
      {/if}
      <button
        class="bg-hazardo-accent text-white rounded p-2 hover:bg-hazardo-primary transition-colors"
        on:click={openAddCategory}
        title={$t('vault.add_category')}
      >
        <Icon name="plus" size={18} />
      </button>
    </div>
  </div>

  <div class="mb-3">
    <SearchBar bind:value={searchQuery} />
  </div>

  <!-- Scrollable item list -->
  <div class="flex items-center justify-between mb-2">
    <h3 class="title-font text-base">{$t('vault.item_list')}</h3>
    <button
      class="bg-hazardo-accent text-white rounded-full w-9 h-9 flex items-center justify-center hover:bg-hazardo-primary transition-colors"
      on:click={openAddItem}
      title={$t('vault.add_item')}
    >
      <Icon name="plus" size={18} />
    </button>
  </div>

  <div class="flex-1 overflow-y-auto" bind:this={listContainer}>
    {#if filteredItems.length === 0}
      <p class="text-center text-hazardo-lightGray py-8 text-sm">{$t('vault.no_items')}</p>
    {:else}
      {#each filteredItems as item, i}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div class="flex items-center justify-between py-3 border-b border-hazardo-lightGray/30 cursor-pointer hover:bg-hazardo-background/50 transition-colors" on:click={() => openEditItem(item)}>
          <div class="flex items-center gap-2">
            <Icon name={getCategoryIcon(item.category_id)} size={16} />
            <span class="text-sm {item.is_picked ? 'line-through text-hazardo-lightGray' : ''}">{item.item_name}</span>
          </div>
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <!-- svelte-ignore a11y-no-static-element-interactions -->
          <div class="flex items-center gap-2" on:click|stopPropagation>
            <button class="p-2 text-hazardo-lightGray hover:text-hazardo-primary rounded-lg hover:bg-hazardo-background transition-colors" on:click={() => openEditItem(item)}>
              <Icon name="edit" size={18} />
            </button>
            <button class="p-2 text-hazardo-lightGray hover:text-red-500 rounded-lg hover:bg-red-50 transition-colors" on:click={() => handleDeleteItem(item.item_id)}>
              <Icon name="trash" size={18} />
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <ScrollToTop container={listContainer} />
</main>

<!-- Add/Edit Category Modal -->
<Modal bind:show={showCategoryModal} title={editingCategory ? $t('vault.edit_category') : $t('vault.add_category')} width="w-96">
  <div class="flex flex-col gap-4 pb-48">
    <div class="flex flex-col">
      <FormLabel label={$t('vault.category_name')} />
      <FormInput bind:value={catName} placeholder={$t('vault.category_name_ph')} />
    </div>
    <div class="flex flex-col">
      <FormLabel label={$t('vault.icon')} />
      <SelectDropdown options={iconOptions} bind:selected={catIcon} placeholder="" />
    </div>
    <div class="flex justify-end gap-2">
      {#if !editingCategory}
        <button type="button" class="px-4 py-2 rounded border border-hazardo-accent text-hazardo-accent text-sm font-medium hover:bg-hazardo-background transition-colors" on:click={() => handleSaveCategory(true)}>{$t('vault.add')}</button>
        <button type="button" class="px-4 py-2 rounded bg-hazardo-primary text-white text-sm font-medium" on:click={() => handleSaveCategory(false)}>{$t('vault.done')}</button>
      {:else}
        <button type="button" class="px-4 py-2 rounded bg-red-500 text-white text-sm font-medium hover:bg-red-600 transition-colors" on:click={handleDeleteCategory}>
          <Icon name="trash" size={14} />
        </button>
        <button type="button" class="px-4 py-2 rounded bg-hazardo-primary text-white text-sm font-medium" on:click={() => handleSaveCategory(false)}>{$t('settings.save')}</button>
      {/if}
    </div>
  </div>
</Modal>

<!-- Add/Edit Item Modal -->
<Modal bind:show={showItemModal} title={editingItem ? $t('vault.edit_item') : $t('vault.add_item')} width="w-96">
  <div class="flex flex-col gap-4">
    <div class="flex flex-col">
      <FormLabel label={$t('vault.item_name')} />
      <FormInput bind:value={itemName} placeholder={$t('vault.item_name_ph')} />
    </div>
    <div class="flex flex-col">
      <FormLabel label={$t('vault.category')} />
      <SelectDropdown options={itemCategoryOptions} bind:selected={itemCategoryId} placeholder={$t('vault.select_category')} />
    </div>
    <div>
      <FormLabel label={$t('vault.time_pref')} />
      <div class="mt-1">
        <OptionToggle options={timeOptions} bind:selected={itemTimePref} />
      </div>
    </div>
    <div>
      <FormLabel label={$t('vault.vibe_pref')} />
      <div class="mt-1">
        <OptionToggle options={vibeOptions} bind:selected={itemVibePref} />
      </div>
    </div>
    <div class="flex flex-col">
      <FormLabel label={$t('vault.notes_optional')} />
      <textarea
        bind:value={itemNotes}
        placeholder={$t('vault.add_notes_ph')}
        class="border rounded p-2 border-hazardo-lightGray focus:outline-hazardo-accent w-full text-sm resize-none"
        rows="2"
      ></textarea>
    </div>
    {#if editingItem}
      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" checked={itemIsPicked === 1} on:change={(e) => itemIsPicked = e.currentTarget.checked ? 1 : 0} class="w-4 h-4 accent-hazardo-accent" />
        {$t('vault.already_picked')}
      </label>
    {/if}
    <div class="flex justify-end gap-2">
      {#if !editingItem}
        <button type="button" class="px-4 py-2 rounded border border-hazardo-accent text-hazardo-accent text-sm font-medium hover:bg-hazardo-background transition-colors" on:click={() => handleSaveItem(true)}>{$t('vault.add')}</button>
        <button type="button" class="px-4 py-2 rounded bg-hazardo-primary text-white text-sm font-medium" on:click={() => handleSaveItem(false)}>{$t('vault.done')}</button>
      {:else}
        <button type="button" class="px-4 py-2 rounded bg-hazardo-primary text-white text-sm font-medium" on:click={() => handleSaveItem(false)}>{$t('settings.save')}</button>
      {/if}
    </div>
  </div>
</Modal>
