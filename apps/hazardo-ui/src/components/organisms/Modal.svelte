<script lang="ts">
  import Icon from '../atoms/Icon.svelte';

  export let show: boolean = false;
  export let title: string = '';
  export let width: string = 'w-80';
</script>

{#if show}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40" on:click={() => show = false}>
    <div class="{width} bg-white rounded-lg p-6 shadow-xl max-h-[85vh] flex flex-col" on:click|stopPropagation>
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
