<script lang="ts">
  import { onDestroy } from 'svelte';
  import Icon from '../atoms/Icon.svelte';

  export let show: boolean = false;
  export let title: string = '';
  export let width: string = 'w-80';

  let dismissable = false;
  $: if (show) {
    dismissable = false;
    setTimeout(() => dismissable = true, 300);
  }

  function backdropClick() {
    if (dismissable) show = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && show) show = false;
  }

  // Bind/unbind Escape listener when modal visibility changes
  $: if (typeof window !== 'undefined') {
    if (show) {
      window.addEventListener('keydown', handleKeydown);
    } else {
      window.removeEventListener('keydown', handleKeydown);
    }
  }

  onDestroy(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('keydown', handleKeydown);
    }
  });
</script>

{#if show}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40" on:click={backdropClick}>
    <div class="{width} bg-hazardo-surface rounded-lg p-6 shadow-xl max-h-[85vh] flex flex-col" role="dialog" aria-modal="true" aria-label={title || 'Dialog'} tabindex="-1" on:click|stopPropagation>
      {#if title}
        <div class="flex items-center justify-between mb-4">
          <h2 class="title-font text-lg">{title}</h2>
          <button class="text-hazardo-text hover:text-hazardo-primary" on:click={() => show = false}>
            <Icon name="close" size={20} />
          </button>
        </div>
      {/if}
      <div class="overflow-y-auto flex-1">
        <slot />
      </div>
    </div>
  </div>
{/if}
