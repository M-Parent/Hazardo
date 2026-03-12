<script lang="ts">
  import Icon from '../atoms/Icon.svelte';

  export let value: string = '';
  export let placeholder: string = 'Search...';
</script>

<div class="relative">
  <input
    type="text"
    class="w-full border rounded p-2 pr-9 border-hazardo-lightGray focus:outline-hazardo-accent bg-white text-sm"
    {placeholder}
    bind:value
  />
  <span class="absolute right-2.5 top-1/2 -translate-y-1/2 text-hazardo-lightGray pointer-events-none">
    <Icon name="search" size={16} />
  </span>
</div>
