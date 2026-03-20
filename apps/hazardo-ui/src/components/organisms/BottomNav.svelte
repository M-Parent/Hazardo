<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { t } from '../../stores/i18nStore';
  import Icon from '../atoms/Icon.svelte';

  $: currentPath = $page.url.pathname;

  const navItems = [
    { href: '/picked', icon: 'picked' as const, key: 'nav.picked' },
    { href: '/vault', icon: 'vault' as const, key: 'nav.vault' },
    { href: '/', icon: 'home' as const, key: 'nav.home' },
    { href: '/chatbot', icon: 'chatbot' as const, key: 'nav.chatbot' },
    { href: '/setting', icon: 'setting' as const, key: 'nav.setting' },
  ];

  function isActive(href: string, path: string): boolean {
    if (href === '/') return path === '/';
    return path.startsWith(href);
  }

  function navigate(href: string) {
    goto(href);
  }
</script>

<nav class="fixed bottom-0 left-0 right-0 bg-hazardo-surface border-t border-hazardo-lightGray flex justify-around py-3 z-40">
  {#each navItems as item}
    <button
      on:click={() => navigate(item.href)}
      class="relative flex flex-col items-center text-xs gap-1 transition-colors {isActive(item.href, currentPath) ? 'text-hazardo-primary' : 'text-hazardo-text'}"
    >
      {#if isActive(item.href, currentPath)}
        <span class="absolute -top-3 w-10 h-[3px] bg-hazardo-accent rounded-b"></span>
      {/if}
      <Icon name={item.icon} size={22} />
      {$t(item.key)}
    </button>
  {/each}
</nav>
