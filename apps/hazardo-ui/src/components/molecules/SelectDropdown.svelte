<script lang="ts">
  import Icon from '../atoms/Icon.svelte';
  import type { IconName } from '$lib/types';

  export let options: { value: string; label: string; icon?: string }[] = [];
  export let selected: string = '';
  export let placeholder: string = 'Select...';

  let open = false;

  $: selectedOption = options.find(o => o.value === selected);
  $: displayText = selectedOption?.label || placeholder;
  $: displayIcon = selectedOption?.icon;
  $: hasIcons = options.some(o => o.icon);

  function select(value: string) {
    selected = value;
    open = false;
  }
</script>

{#if hasIcons}
  <div class="relative">
    <button
      type="button"
      class="w-full flex items-center justify-between border rounded p-2 border-hazardo-lightGray bg-white text-sm text-left"
      on:click={() => open = !open}
    >
      <span class="flex items-center gap-2 truncate">
        {#if displayIcon}
          <Icon name={displayIcon} size={16} />
        {/if}
        {displayText}
      </span>
      <span class="text-hazardo-lightGray ml-1 shrink-0">
        <Icon name="chevron-down" size={16} />
      </span>
    </button>

    {#if open}
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div class="fixed inset-0 z-[55]" on:click={() => open = false}></div>
      <div class="absolute left-0 right-0 mt-1 bg-white border border-hazardo-lightGray rounded shadow-lg z-[60] max-h-80 overflow-y-auto">
        {#each options as opt}
          <button
            type="button"
            class="w-full flex items-center gap-2 px-3 py-2 text-sm text-left hover:bg-hazardo-background transition-colors {opt.value === selected ? 'text-hazardo-primary font-semibold bg-hazardo-background/50' : ''}"
            on:click={() => select(opt.value)}
          >
            {#if opt.icon}
              <Icon name={opt.icon} size={16} />
            {/if}
            {opt.label}
          </button>
        {/each}
      </div>
    {/if}
  </div>
{:else}
  <div class="relative">
    <select
      bind:value={selected}
      class="w-full appearance-none border rounded p-2 pr-8 border-hazardo-lightGray focus:outline-hazardo-accent bg-white text-sm"
    >
      {#if placeholder}
        <option value="" disabled>{placeholder}</option>
      {/if}
      {#each options as opt}
        <option value={opt.value}>{opt.label}</option>
      {/each}
    </select>
    <span class="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-hazardo-lightGray">
      <Icon name="chevron-down" size={16} />
    </span>
  </div>
{/if}
