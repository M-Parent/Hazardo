<script lang="ts">
  export let id: string = '';
  export let name: string = '';
  export let value: string = '';
  export let placeholder: string = '';
  export let type: string = 'text';
</script>

<input
  {id}
  {name}
  {type}
  {placeholder}
  class="border rounded p-2 border-hazardo-lightGray focus:outline-hazardo-accent w-full"
  bind:value
/>
