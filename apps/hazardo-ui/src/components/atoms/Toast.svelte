<script lang="ts">
  import { toasts } from '../../stores/toastStore';
</script>

{#if $toasts.length > 0}
  <div class="fixed top-4 left-1/2 -translate-x-1/2 z-80 flex flex-col gap-2 pointer-events-none">
    {#each $toasts as toast (toast.id)}
      <div class="px-4 py-2 rounded-lg shadow-lg text-sm font-medium pointer-events-auto animate-slide-down
        {toast.type === 'success' ? 'bg-green-500 text-white' : 'bg-red-500 text-white'}">
        {toast.message}
      </div>
    {/each}
  </div>
{/if}

<style>
  @keyframes slide-down {
    from { opacity: 0; transform: translateY(-10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .animate-slide-down {
    animation: slide-down 0.25s ease-out;
  }
</style>
