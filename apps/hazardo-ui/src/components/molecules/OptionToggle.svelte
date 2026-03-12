<script lang="ts">
  export let options: string[];
  export let selected: string;
  export let label: string = '';
</script>

{#if label}
  <label class="block mb-2">{label}</label>
{/if}
<div class="flex gap-0 border rounded border-hazardo-lightGray overflow-hidden">
  {#each options as opt}
    <button
      class="flex-1 py-2 text-sm transition-colors {selected === opt ? 'bg-hazardo-accent text-white' : ''}"
      on:click={() => selected = opt}
    >{opt}</button>
  {/each}
</div>
