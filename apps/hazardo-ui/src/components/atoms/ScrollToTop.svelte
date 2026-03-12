<script lang="ts">
  import Icon from '../atoms/Icon.svelte';

  export let container: HTMLElement | null = null;
  let visible = false;

  function handleScroll() {
    if (container) {
      visible = container.scrollTop > 200;
    }
  }

  function scrollToTop() {
    if (container) {
      container.scrollTo({ top: 0, behavior: 'smooth' });
    }
  }

  $: if (container) {
    container.addEventListener('scroll', handleScroll);
  }
</script>

{#if visible}
  <button
    class="fixed bottom-20 right-4 z-30 bg-hazardo-accent text-white rounded-full w-10 h-10 flex items-center justify-center shadow-lg hover:bg-hazardo-primary transition-colors"
    on:click={scrollToTop}
  >
    <Icon name="arrow-up" size={20} />
  </button>
{/if}
