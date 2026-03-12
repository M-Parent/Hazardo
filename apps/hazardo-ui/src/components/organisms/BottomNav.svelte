<script lang="ts">
  import { page } from '$app/stores';
  import Icon from '../atoms/Icon.svelte';

  $: currentPath = $page.url.pathname;

  const navItems = [
    { href: '/picked', icon: 'picked' as const, label: 'Picked' },
    { href: '/vault', icon: 'vault' as const, label: 'Vault' },
    { href: '/', icon: 'home' as const, label: 'Home' },
    { href: '/chatbot', icon: 'chatbot' as const, label: 'ChatBot' },
    { href: '/setting', icon: 'setting' as const, label: 'Setting' },
  ];

  function isActive(href: string, path: string): boolean {
    if (href === '/') return path === '/';
    return path.startsWith(href);
  }
</script>

<nav class="fixed bottom-0 left-0 right-0 bg-white border-t border-hazardo-lightGray flex justify-around py-3 z-40">
  {#each navItems as item}
    <a
      href={item.href}
      class="relative flex flex-col items-center text-xs gap-1 transition-colors {isActive(item.href, currentPath) ? 'text-hazardo-primary' : 'text-hazardo-text'}"
    >
      {#if isActive(item.href, currentPath)}
        <span class="absolute -top-3 w-10 h-[3px] bg-hazardo-accent rounded-b"></span>
      {/if}
      <Icon name={item.icon} size={22} />
      {item.label}
    </a>
  {/each}
</nav>
